use actix_web::{web, http::header::LOCATION, HttpResponse, http::StatusCode, ResponseError};
use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction, Executor};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use uuid::Uuid;
use crate::domain::SubscriberEmail;
use chrono::Utc;


#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error)
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
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
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String
}

impl TryFrom<FormData> for SubscriberEmail {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        Ok(SubscriberEmail::parse(form.email)?)
    }
}

pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber: &SubscriberEmail
) -> Result<Uuid, sqlx::error::Error> {
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

    transaction.execute(query).await?;

    Ok(id)
}

pub struct StoreTokenError(sqlx::Error);

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A database error was encountered while trying to store a subscription token")
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
    subscription_token: &str
) -> Result<(), StoreTokenError>{
    let query = sqlx::query!(
        r#"
            INSERT INTO subscription_tokens (subscription_token, subscriber_id)
            VALUES ($1, $2)
        "#,
        subscription_token, subscriber_id
    );

    transaction.execute(query)
        .await
        .map_err(|e| { StoreTokenError(e)})?;

    Ok(())
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

pub async fn signup(
    form: web::Form<FormData>,
    connection_pool: web::Data<PgPool>
    ) -> Result<HttpResponse, SubscribeError> {
    let subscriber = form.0.try_into().map_err(SubscribeError::ValidationError)?;
    let mut transaction = connection_pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    let subscriber_id = insert_subscriber(&mut transaction, &subscriber)
        .await
        .context("Failed to insert a new subscriber in the database")?;
        
    let subscription_token = generate_subscription_token();
    store_token(&mut transaction, subscriber_id, &subscription_token)
        .await
        .context("Failed to insert subscription token in the database")?;

    transaction.commit()
        .await
        .context("Failed to commit SQL transaction to the database")?;

    Ok(HttpResponse::SeeOther()
       .insert_header((LOCATION, "/blog"))
       .finish()
    )
}
