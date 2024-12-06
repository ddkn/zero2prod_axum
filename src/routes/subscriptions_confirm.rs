use crate::routes::error_chain_fmt;
use anyhow::Context;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use sqlx::SqlitePool;
use sqlx::{Executor, Sqlite, Transaction};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[derive(thiserror::Error)]
pub enum SubscriptionsConfirmError {
    #[error("{0}")]
    InvalidTokenIdError(#[from] uuid::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for SubscriptionsConfirmError {
    fn into_response(self) -> Response {
        match self {
            SubscriptionsConfirmError::InvalidTokenIdError(_) => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
            SubscriptionsConfirmError::UnexpectedError(_) => {
                tracing::error!(error = ?self, "Subscription Confirmation Error",);
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        }
    }
}

impl std::fmt::Debug for SubscriptionsConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[tracing::instrument(
    name = "Confirm a pending subscriber",
    skip(pool, params)
)]
pub async fn confirm(
    Extension(pool): Extension<SqlitePool>,
    Query(params): Query<Parameters>,
) -> Result<impl IntoResponse, SubscriptionsConfirmError> {
    let mut transaction = pool
        .begin()
        .await
        .context("Unable to acqurie SQL connection to pool.")?;

    let id = get_subscriber_id_from_token(
        &mut transaction,
        &params.subscription_token,
    )
    .await
    .context("Unable to query `subscriber_id`.")?;

    match id {
        None => return Ok(StatusCode::UNAUTHORIZED),
        Some(id) => confirm_subscriber(&mut transaction, id)
            .await
            .context("Failed to confirm subscriber.")?,
    };
    transaction
        .commit()
        .await
        .context("Unable to to complete SQL transaction.")?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(
    name = "Mark subscriber as confirmed",
    skip(subscriber_id, transaction)
)]
pub async fn confirm_subscriber(
    transaction: &mut Transaction<'_, Sqlite>,
    subscriber_id: Uuid,
) -> Result<(), sqlx::Error> {
    let subscriber_id = subscriber_id.to_string();
    let query = sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id,
    );

    transaction.execute(query).await?;
    Ok(())
}

#[tracing::instrument(
    name = "Get subscriber id from token",
    skip(transaction, subscription_token)
)]
pub async fn get_subscriber_id_from_token(
    transaction: &mut Transaction<'_, Sqlite>,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let subscription_token = subscription_token.to_string();
    // &mut Transaction<'_, Sqlite> wraps the executor twice, hence the double dereference
    let result = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens \
        WHERE subscription_token = $1",
        subscription_token,
    )
    .fetch_optional(&mut **transaction)
    .await?;

    let id = result.map(|r| r.subscriber_id).unwrap();
    Ok(Uuid::parse_str(&id).ok())
}
