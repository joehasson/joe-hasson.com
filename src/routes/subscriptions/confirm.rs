use actix_web::{web, http::header::LOCATION, HttpResponse, http::StatusCode, ResponseError};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;
use actix_session::Session;
use crate::{util::error_chain_fmt, flash_message::Flash};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String
}

#[derive(thiserror::Error)]
pub enum SubscriptionConfirmError {
    InvalidSubscriptionToken,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error)
}

impl std::fmt::Debug for SubscriptionConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for SubscriptionConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An error was encountered while trying to confirm a subscription.")
    }
}
impl ResponseError for SubscriptionConfirmError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidSubscriptionToken => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn confirm(
    pool: web::Data<PgPool>,
    parameters: web::Query<Parameters>,
    session: Session
) -> Result<HttpResponse, SubscriptionConfirmError> {
    let subscriber_id = get_subscriber_id_from_token(
            &pool,
            &parameters.subscription_token
        )
        .await
        .context("Failed to look up subscriber id in subscription_tokens table")?
        .ok_or(SubscriptionConfirmError::InvalidSubscriptionToken)?;

    confirm_subscriber(&pool, subscriber_id)
        .await
        .context("Failed to register subscriber confirmation in database")?;

    session.set_flash("Subscription confirmed!")
        .context("Failed to set session state")?;

    Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, "/blog"))
        .finish())
}

async fn get_subscriber_id_from_token(
    pool: &PgPool,
    token: &str
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens 
        WHERE subscription_token = $1
        "#, token)
    .fetch_optional(pool)
    .await?;
    Ok(result.map(|record| record.subscriber_id))
}

async fn confirm_subscriber(
    pool: &PgPool,
    subscriber_id: Uuid
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET confirmed = true WHERE id = $1"#,
        subscriber_id
        ).execute(pool)
    .await?;
    Ok(())
}
