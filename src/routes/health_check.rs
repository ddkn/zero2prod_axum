//! src/routes/health_check.rs

use axum::{http::StatusCode, response::IntoResponse};

/// Health check
///
/// Alive server check for external services.
///
/// # Returns
/// `(SuccessCode::OK, "")`
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "")
}
