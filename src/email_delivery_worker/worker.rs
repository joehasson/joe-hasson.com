use crate::{
    domain::{InvalidEmailError, SubscriberEmail},
    email_delivery_queue::{self, deprioritise_task},
    email_delivery_worker::email_client::{EmailClient, EmailClientError},
};
use lettre::AsyncTransport;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

pub async fn worker<T>(email_client: Arc<EmailClient<T>>, connection_pool: Arc<PgPool>)
where
    T: AsyncTransport + Sync + Send,
    T::Error: std::error::Error,
{
    loop {
        if let Err(e) = try_execute_task(&email_client, &connection_pool).await {
            match e {
                TryTaskError::CorruptedData(_) => {
                    log::error!("Error in email delivery worker: {}", e);
                    continue;
                }
                // Wait for tasks to become available
                TryTaskError::NoPendingTask => {
                    log::debug!("No pending tasks. Worker sleeping..");
                    tokio::time::sleep(Duration::from_secs(1)).await
                }

                // Sleep through (hopefully transient) db or email client errors.
                // Would be nice to implement exponential backoff, alerting, and
                // distinguish transient / fatal errors eventually
                TryTaskError::DatabaseError(_) | TryTaskError::EmailClientError(_) => {
                    log::error!("Error in email delivery worker: {}", e);
                    tokio::time::sleep(Duration::from_secs(10)).await
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum TryTaskError {
    #[error("{0}")]
    CorruptedData(#[from] InvalidEmailError),
    #[error("{0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("{0}")]
    EmailClientError(#[from] EmailClientError),
    #[error("No tasks ready to execute")]
    NoPendingTask,
}

async fn try_execute_task<T>(
    email_client: &EmailClient<T>,
    connection_pool: &PgPool,
) -> Result<(), TryTaskError>
where
    T: AsyncTransport + Sync + Send,
    T::Error: std::error::Error,
{
    let mut transaction = connection_pool.begin().await?;

    let task = email_delivery_queue::peek_task(&mut *transaction)
        .await?
        .ok_or(TryTaskError::NoPendingTask)?;

    let recipient = match SubscriberEmail::parse(task.recipient) {
        Ok(r) => r,
        // TODO: if email is unparseable, immediately move task to dead
        // letter queue once implemented instead of retrying + backing off
        // like we do here
        Err(e) => {
            deprioritise_task(&mut *transaction, task.id, task.n_retries).await?;
            transaction.commit().await?;
            return Err(TryTaskError::CorruptedData(e));
        }
    };

    if let Err(e) = email_client
        .send_email(
            &recipient,
            &task.subject,
            &task.email_html,
            &task.email_text,
        )
        .await
    {
        deprioritise_task(&mut *transaction, task.id, task.n_retries).await?;
        transaction.commit().await?;
        return Err(TryTaskError::EmailClientError(e));
    }

    email_delivery_queue::pop_task(&mut *transaction, task.id).await?;
    transaction.commit().await?;
    Ok(())
}
