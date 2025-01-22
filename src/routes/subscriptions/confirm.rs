use crate::{
    domain::SubscriberEmail, email_client::EmailClient, flash_message::Flash, util::error_chain_fmt,
};
use actix_session::Session;
use actix_web::{http::header::LOCATION, http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use lettre::AsyncTransport;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[derive(thiserror::Error)]
pub enum SubscriptionConfirmError {
    InvalidSubscriptionToken,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SubscriptionConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for SubscriptionConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "An error was encountered while trying to confirm a subscription."
        )
    }
}
impl ResponseError for SubscriptionConfirmError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidSubscriptionToken => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn confirm<T>(
    pool: web::Data<PgPool>,
    parameters: web::Query<Parameters>,
    email_client: web::Data<EmailClient<T>>,
    session: Session,
) -> Result<HttpResponse, SubscriptionConfirmError>
where
    T: AsyncTransport + Sync + Send,
    T::Error: std::error::Error,
{
    let subscriber_id = get_subscriber_id_from_token(&pool, &parameters.subscription_token)
        .await
        .context("Failed to look up subscriber id in subscription_tokens table")?
        .ok_or(SubscriptionConfirmError::InvalidSubscriptionToken)?;

    let subscriber_email = confirm_subscriber(&pool, subscriber_id).await?;

    session
        .set_flash("Your subscription is confirmed!")
        .context("Failed to set session state")?;

    // TODO: enqueue send email-task with a separate
    // service once implemented; not great to block like this
    email_client
        .send_email_to_subscriber(
            subscriber_id,
            subscriber_email,
            "Welcome!",
            "<p>Your subscription to my blog is now confirmed. Welcome!</p>",
            "Your subscription to my blog is now confirmed. Welcome!",
        )
        .await
        .with_context(|| String::from("Failed to send email"))?;

    Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, "/blog"))
        .finish())
}

async fn get_subscriber_id_from_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens 
        WHERE subscription_token = $1
        "#,
        token
    )
    .fetch_optional(pool)
    .await?;
    Ok(result.map(|record| record.subscriber_id))
}

async fn confirm_subscriber(
    pool: &PgPool,
    subscriber_id: Uuid,
) -> Result<SubscriberEmail, anyhow::Error> {
    let subscriber_email = sqlx::query!(
        r#"
        UPDATE subscriptions
        SET confirmed = true 
        WHERE id = $1
        RETURNING email"#,
        subscriber_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to register subscriber confirmation in database")?
    .email;

    SubscriberEmail::parse(subscriber_email).with_context(|| {
        format!(
            "Malformed email in database for subscriber_id {}",
            subscriber_id
        )
    })
}
