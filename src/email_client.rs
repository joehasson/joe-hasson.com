use crate::domain::SubscriberEmail;
use crate::util::error_chain_fmt;
use lettre::message::Mailbox;
use lettre::message::{Message, MultiPart};
use lettre::AsyncTransport;
use log;
use std::error::Error as StdError;
use std::sync::Arc;
use uuid::Uuid;

pub struct EmailClient<T: AsyncTransport + Send + Sync> {
    smtp_client: Arc<T>,
    sender: Mailbox,
    app_base_url: String,
}

// TODO: nice recursive Debug trait like in routes/subscriptions.rs
pub enum EmailClientError {
    AddressError(lettre::address::AddressError),
    EmailError(lettre::error::Error),
    // TODO: Would be nice if I could wrap the various AsyncTransport::Error's for
    // the various different AsyncTransport types better. Can't even use trait bounds
    // on std::error::Error bc AsyncTransport::Error might not implement that
    TransportError(&'static str),
}

impl From<lettre::address::AddressError> for EmailClientError {
    fn from(value: lettre::address::AddressError) -> Self {
        Self::AddressError(value)
    }
}

impl From<lettre::error::Error> for EmailClientError {
    fn from(value: lettre::error::Error) -> Self {
        Self::EmailError(value)
    }
}
impl std::fmt::Debug for EmailClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for EmailClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
        //write!(f, "Email Client Error")
    }
}

impl std::error::Error for EmailClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::AddressError(e) => Some(e),
            Self::EmailError(e) => Some(e),
            Self::TransportError(_) => None,
        }
    }
}

impl<T> EmailClient<T>
where
    T: AsyncTransport + Send + Sync,
    T::Error: std::error::Error,
{
    pub fn new(
        smtp_client: Arc<T>,
        from: &str,
        app_base_url: String,
    ) -> Result<EmailClient<T>, String> {
        let sender: Mailbox = match format!("Joe Hasson Blog <{}>", from).parse() {
            Ok(mailbox) => mailbox,
            Err(_) => return Err("Invalid sender".into()),
        };
        Ok(EmailClient {
            smtp_client,
            sender,
            app_base_url,
        })
    }

    async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), EmailClientError> {
        let mailbox: Mailbox = recipient.as_ref().parse()?;

        let message = Message::builder()
            .from(self.sender.clone())
            .to(mailbox)
            .subject(subject)
            .multipart(MultiPart::alternative_plain_html(
                String::from(text_content),
                String::from(html_content),
            ))?;

        log::info!("About to send email...");
        match self.smtp_client.send(message).await {
            Ok(_) => Ok(()),
            // TODO: This is  terrible error handling fix it up
            Err(e) => {
                log::error!("Email error: {:?}", e);
                let mut current = e.source();
                while let Some(cause) = current {
                    log::error!("Caused by: {}", cause);
                    current = cause.source();
                }
                Err(EmailClientError::TransportError("Failed to send email"))
            }
        }
    }

    pub async fn send_email_to_subscriber(
        &self,
        subscriber_id: Uuid,
        subscriber_email: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), EmailClientError> {
        let unsubscribe_link = format!(
            "{}/subscriptions/unsubscribe?id={}",
            self.app_base_url, subscriber_id
        );

        let html_content = &format!(
            r#"
            <div style="font-family: sans-serif; max-width: 600px; margin: 0 auto;">
                {html_content}
        
                <hr style="border: none; border-top: 1px solid #ddd; margin: 20px 0;"/>
        
                <footer style="font-size: 12px; color: #666;">
                    <p>
                        You're receiving this because you subscribed to Joe Hasson's Blog.<br/>
                        <a href="{unsubscribe_link}" style="color: #666;">Unsubscribe</a>
                    </p>
                </footer>
            </div>"#
        );

        let text_content = &format!(
            "{text_content}\n\n\
            -------------------------------------------\n\
            You're receiving this because you subscribed to Joe Hasson's Blog.\n\
            Unsubscribe: {unsubscribe_link}\n"
        );

        self.send_email(&subscriber_email, subject, html_content, text_content)
            .await?;
        Ok(())
    }

    pub async fn send_confirmation_email(
        &self,
        subscription_token: &str,
        subscriber_email: SubscriberEmail,
    ) -> Result<(), EmailClientError> {
        let confirmation_link = format!(
            "{}/subscriptions/confirm?subscription_token={}",
            self.app_base_url, subscription_token
        );
        log::info!("Sending confirmation email with link {}", confirmation_link);

        let html_content = &format!(
            "
            Thanks for signing up to my blog! 
            <br /> Click <a href=\"{}\">here</a>
            to confirm your subscription.
            ",
            confirmation_link
        );
        let text_content = &format!(
            "
            Thanks for signing up to my blog!
            Visit {} to confirm your subscription.
            ",
            confirmation_link
        );
        self.send_email(&subscriber_email, "Welcome!", html_content, text_content)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::EmailClient;
    use crate::domain::SubscriberEmail;
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::Fake;
    use lettre::transport::stub::AsyncStubTransport;
    use std::sync::Arc;

    #[tokio::test]
    async fn err_if_server_errors() {
        let stub_client = Arc::new(AsyncStubTransport::new_error());
        let client =
            EmailClient::new(stub_client, "test@tld.com", "localhost:8000".into()).unwrap();

        let recipient = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        assert_err!(
            client
                .send_email(&recipient, &subject, &content, &content)
                .await
        );
    }

    #[tokio::test]
    async fn ok_if_server_ok() {
        let stub_client = Arc::new(AsyncStubTransport::new_ok());
        let client =
            EmailClient::new(stub_client, "test@tld.com", "localhost:8000".into()).unwrap();

        let recipient = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        assert_ok!(
            client
                .send_email(&recipient, &subject, &content, &content)
                .await
        );
    }
}
