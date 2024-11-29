use axum::{
    extract::Query, http::StatusCode, response::IntoResponse, Extension,
};
use sqlx::{Connection, SqlitePool};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
    name = "Confirm a pending subscriber",
    skip(pool, params)
)]
pub async fn confirm(
    Extension(pool): Extension<SqlitePool>,
    Query(params): Query<Parameters>,
) -> impl IntoResponse {
    let id =
        match get_subscriber_id_from_token(&pool, &params.subscription_token)
            .await
        {
            Ok(id) => id,
            Err(_) => return StatusCode::UNAUTHORIZED,
        };

    match id {
        None => return StatusCode::UNAUTHORIZED,
        Some(id) => {
            if confirm_subscriber(&pool, id).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
    };
    StatusCode::OK
}

#[tracing::instrument(
    name = "Mark subscriber as confirmed",
    skip(subscriber_id, pool)
)]
pub async fn confirm_subscriber(
    pool: &SqlitePool,
    subscriber_id: Uuid,
) -> Result<(), sqlx::Error> {
    let subscriber_id = subscriber_id.to_string();
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Get subscriber id from token",
    skip(pool, subscription_token)
)]
pub async fn get_subscriber_id_from_token(
    pool: &SqlitePool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let subscription_token = subscription_token.to_string();
    let result = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens \
        WHERE subscription_token = $1",
        subscription_token,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    let id = result.map(|r| r.subscriber_id).unwrap();
    let id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(None),
    };

    Ok(Some(id))
}
