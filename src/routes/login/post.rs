use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};

pub async fn login() -> impl IntoResponse {
    Redirect::to("/")
}
