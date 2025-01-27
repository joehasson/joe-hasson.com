use chrono::{DateTime, Utc};
use sqlx::{Executor, FromRow, Postgres};
use uuid::Uuid;

#[derive(FromRow)]
pub struct EmailDeliveryTask {
    pub id: Uuid,
    pub subscriber_id: Uuid,
    pub email: String,
    pub subject: String,
    pub email_html: String,
    pub email_text: String,
    pub created_at: DateTime<Utc>,
    pub n_retries: i32,
    pub send_after: DateTime<Utc>,
}

#[tracing::instrument(skip_all)]
pub async fn push_task<'a, T>(
    executor: T,
    subscriber_id: Uuid,
    subject: &str,
    html_content: &str,
    text_content: &str,
) -> Result<(), sqlx::Error>
where
    T: Executor<'a, Database = Postgres>,
{
    let id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
        INSERT INTO email_delivery_queue
            (id, subscriber_id, subject, email_html, email_text)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        id,
        subscriber_id,
        subject,
        html_content,
        text_content
    );
    query.execute(executor).await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn peek_task<'a, T>(executor: T) -> Result<Option<EmailDeliveryTask>, sqlx::Error>
where
    T: Executor<'a, Database = Postgres>,
{
    sqlx::query_as::<_, EmailDeliveryTask>(
        r#"
        SELECT email_delivery_queue.*, subscriptions.email
        FROM email_delivery_queue JOIN subscriptions
        ON email_delivery_queue.subscriber_id = subscriptions.id
        FOR UPDATE of email_delivery_queue
        SKIP LOCKED
        LIMIT 1
        "#, // FOR UPDATE locks the rows
    )
    .fetch_optional(executor)
    .await
}

#[tracing::instrument(skip_all)]
pub async fn pop_task<'a, T>(executor: T, task_id: Uuid) -> Result<(), sqlx::Error>
where
    T: Executor<'a, Database = Postgres>,
{
    sqlx::query!(
        r#" DELETE FROM email_delivery_queue WHERE id = $1"#,
        task_id
    )
    .execute(executor)
    .await?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn deprioritise_task<'a, T>(
    executor: T,
    task_id: Uuid,
    n_retries: i32,
) -> Result<(), sqlx::Error>
where
    T: Executor<'a, Database = Postgres>,
{
    const BASE_RETRY_INTERVAL_SECS: u64 = 60;
    let delay_secs = BASE_RETRY_INTERVAL_SECS * 2u64.pow(n_retries as u32);
    let send_after = Utc::now() + chrono::Duration::seconds(delay_secs as i64);

    sqlx::query!(
        r#"
        UPDATE email_delivery_queue 
        SET 
            n_retries = n_retries + 1,
            send_after = $1
        WHERE id = $2
    "#,
        send_after,
        task_id
    )
    .bind(task_id)
    .execute(executor)
    .await?;

    Ok(())
}
