use crate::{email_delivery_queue, flash_message::Flash, util::error_chain_fmt};
use actix_session::Session;
use actix_web::{http::header::LOCATION, http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use lettre::AsyncTransport;
use sqlx::{PgPool, Postgres, Transaction};
use tracing;
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

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    confirm_subscriber(&mut transaction, subscriber_id).await?;

    email_delivery_queue::push_task(
        &mut *transaction,
        subscriber_id,
        "Welcome!",
        "<p>Your subscription to my blog is now confirmed. Welcome!</p>",
        "Your subscription to my blog is now confirmed. Welcome!",
    )
    .await
    .with_context(|| String::from("Failed to send email"))?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to the database")?;

    session
        .set_flash("Your subscription is confirmed!")
        .context("Failed to set session state")?;

    Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, "/blog"))
        .finish())
}

#[tracing::instrument(skip_all)]
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

#[tracing::instrument(skip_all)]
async fn confirm_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        UPDATE subscriptions
        SET confirmed = true 
        WHERE id = $1"#,
        subscriber_id
    )
    .execute(&mut **transaction) // Rust :)
    .await
    .context("Failed to register subscriber confirmation in database")?;

    Ok(())
}
