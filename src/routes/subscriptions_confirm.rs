use axum::{extract::Query, http::StatusCode, response::IntoResponse};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_params))]
pub async fn confirm(Query(_params): Query<Parameters>) -> impl IntoResponse {
    StatusCode::OK
}
