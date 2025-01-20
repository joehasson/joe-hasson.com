use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String
}

pub async fn confirm(
    pool: web::Data<PgPool>,
    parameters: web::Query<Parameters>
) -> HttpResponse {
    let maybe_id = match
        get_subscriber_id_from_token(
            &pool,
            &parameters.subscription_token
        )
        .await {
            Ok(maybe_id) => maybe_id,
            Err(_) => {
                return HttpResponse::InternalServerError().finish()
            }
        };

    match maybe_id {
        None => HttpResponse::Unauthorized().finish(),
        Some(id) => {
            if confirm_subscriber(&pool, id).await.is_err() {
                return HttpResponse::InternalServerError().finish()
            }
            HttpResponse::Ok().finish()
        }
    }
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
