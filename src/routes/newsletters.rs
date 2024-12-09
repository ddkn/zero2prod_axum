use crate::routes::error_chain_fmt;
use anyhow::Context;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use sqlx::SqlitePool;

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexepectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl IntoResponse for PublishError {
    fn into_response(self) -> Response {
        tracing::error!(error = ?self, "Publish error.");
        match self {
            PublishError::UnexepectedError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        }
    }
}

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

struct ConfirmedSubscriber {
    email: String,
}

pub async fn publish_newsletter(
    Extension(pool): Extension<SqlitePool>,
    _body: Json<BodyData>,
) -> Result<impl IntoResponse, PublishError> {
    let _subscribers = get_confirmed_subscribers(&pool)
        .await
        .context("Unable to query confirmed subscribers")?;
    Ok(StatusCode::OK)
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &SqlitePool,
) -> Result<Vec<ConfirmedSubscriber>, anyhow::Error> {
    let rows = sqlx::query_as!(
        ConfirmedSubscriber,
        r#"SELECT email FROM subscriptions WHERE status = 'confirmed'"#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
