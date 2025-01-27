use crate::{flash_message::Flash, util::error_chain_fmt};
use actix_session::Session;
use actix_web::{http::header::LOCATION, http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use lettre::AsyncTransport;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(thiserror::Error)]
pub enum UnsubscribeError {
    #[error("{0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for UnsubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for UnsubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(serde::Deserialize)]
pub struct Parameters {
    id: Uuid,
}

#[tracing::instrument(skip_all)]
async fn remove_subscriber(
    connection_pool: &PgPool,
    subscriber_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        WITH deleted_subscriptions AS (
            DELETE FROM subscriptions 
            WHERE id = $1
            RETURNING id
        )
        DELETE FROM subscription_tokens 
        WHERE subscriber_id IN (
            SELECT id 
            FROM deleted_subscriptions
        ) "#,
        subscriber_id
    )
    .execute(connection_pool)
    .await?;

    Ok(())
}

pub async fn unsubscribe<T>(
    connection_pool: web::Data<PgPool>,
    parameters: web::Query<Parameters>,
    session: Session,
) -> Result<HttpResponse, UnsubscribeError>
where
    T: AsyncTransport + Sync + Send,
    T::Error: std::error::Error,
{
    remove_subscriber(&connection_pool, parameters.id).await?;
    session
        .set_flash("Successfully unsubscribed!")
        .context("Error setting session state")?;
    Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, "/blog"))
        .finish())
}
