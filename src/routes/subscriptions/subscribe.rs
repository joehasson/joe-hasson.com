use crate::{
    domain::{InvalidEmailError, SubscriberEmail},
    email_delivery_queue,
    flash_message::Flash,
    util::{error_chain_fmt, read_env_or_panic},
};
use actix_session::Session;
use actix_web::{http::header::LOCATION, http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use chrono::Utc;
use log;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(#[from] InvalidEmailError),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
}

#[derive(thiserror::Error, Debug)]
pub enum InsertSubscriberError {
    #[error("Email address already exists")]
    DuplicateEmail,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber: &SubscriberEmail,
) -> Result<Uuid, InsertSubscriberError> {
    let id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, subscribed_at, confirmed)
            VALUES ($1, $2, $3, $4)
            "#,
        id,
        subscriber.as_ref(),
        Utc::now(),
        false
    );

    match transaction.execute(query).await {
        Ok(_) => Ok(id),
        Err(e) => {
            if let Some(db_error) = e.as_database_error() {
                if db_error.code() == Some("23505".into()) {
                    return Err(InsertSubscriberError::DuplicateEmail);
                }
            }

            Err(e.into())
        }
    }
}

pub struct StoreTokenError(sqlx::Error);

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while trying to store a subscription token"
        )
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), StoreTokenError> {
    let query = sqlx::query!(
        r#"
            INSERT INTO subscription_tokens (subscription_token, subscriber_id)
            VALUES ($1, $2)
        "#,
        subscription_token,
        subscriber_id
    );

    transaction.execute(query).await.map_err(StoreTokenError)?;

    Ok(())
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

async fn get_subscription_token_from_subscriber_email(
    connection_pool: &PgPool,
    subscriber_email: &SubscriberEmail,
) -> Result<String, anyhow::Error> {
    sqlx::query!(
        r#"
        SELECT subscription_token
        FROM subscriptions JOIN subscription_tokens
        ON subscriptions.id = subscription_tokens.subscriber_id
        WHERE subscriptions.email = $1
        "#,
        subscriber_email.as_ref()
    )
    .fetch_optional(connection_pool)
    .await?
    .map(|record| record.subscription_token)
    .ok_or(anyhow::anyhow!(format!(
        "Unable to find subscription token for email {}",
        subscriber_email
    )))
}

async fn enqueue_confirmation_email<'a, T>(
    executor: T,
    subscriber_email: &SubscriberEmail,
    subscription_token: &str,
) -> Result<(), sqlx::Error>
where
    T: Executor<'a, Database = Postgres>,
{
    let app_base_url = read_env_or_panic("APP_BASE_URL");

    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        app_base_url, subscription_token
    );

    let html_content = &format!(
        "Thanks for signing up to my blog!
         <br /> Click <a href=\"{}\">here</a>
         to confirm your subscription.",
        confirmation_link
    );

    let text_content = &format!(
        "Thanks for signing up to my blog!\nVisit {} to confirm your subscription.",
        confirmation_link
    );

    email_delivery_queue::push_task(
        executor,
        subscriber_email,
        "Please confirm your subscription.",
        html_content,
        text_content,
    )
    .await?;

    Ok(())
}

pub async fn subscribe<T>(
    form: web::Form<FormData>,
    connection_pool: web::Data<PgPool>,
    session: Session,
) -> Result<HttpResponse, SubscribeError> {
    let subscriber_email = SubscriberEmail::parse(form.0.email)?;

    let mut transaction = connection_pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    log::info!("Attempting to insert subscriber...");
    match insert_subscriber(&mut transaction, &subscriber_email).await {
        Ok(subscriber_id) => {
            log::info!("Succeeded!");
            let subscription_token = generate_subscription_token();
            log::info!("Storing token...");
            store_token(&mut transaction, subscriber_id, &subscription_token)
                .await
                .context("Failed to insert subscription token in the database")?;

            log::info!("Committing transaction...");
            log::info!("Pushing confirmation email task onto queue...");

            enqueue_confirmation_email(&mut *transaction, &subscriber_email, &subscription_token)
                .await
                .context("Error sending confirmation")?;

            transaction
                .commit()
                .await
                .context("Failed to commit SQL transaction to the database")?;

            session.set_flash(
                "Check your inbox for a confirmation email and follow the link to complete your registration."
            ).context("Error setting session state")?;
        }
        // Resend existing confirmation token
        Err(InsertSubscriberError::DuplicateEmail) => {
            log::info!("Duplicate email! Rolling back...");
            transaction.rollback().await.context("Transaction failed")?;
            let subscription_token =
                get_subscription_token_from_subscriber_email(&connection_pool, &subscriber_email)
                    .await?;

            log::info!("Resending confirmation email...");

            enqueue_confirmation_email(&**connection_pool, &subscriber_email, &subscription_token)
                .await
                .context("Error sending confirmation")?;

            session
                .set_flash("Email already registered. A new confirmation email has been sent to your inbox in case you haven't confirmed already.")
                .context("Error setting session state")?;
        }
        Err(e) => {
            log::info!("Unknown error! Rolling transaction back...");
            log::error!("{}", e);
            transaction.rollback().await.context("Transaction failed")?;
            session.set_flash(
                "Sorry - an internal error occurred while processing your sign-up. Please try again later."
            ).context("Error setting session state")?;
        }
    };

    Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, "/blog"))
        .finish())
}
