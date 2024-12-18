use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};

pub async fn login_form() -> impl IntoResponse {
    (StatusCode::OK, Html::from(include_str!("login.html")))
}
