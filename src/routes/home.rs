use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};

pub async fn home() -> impl IntoResponse {
    (StatusCode::OK, Html::from(include_str!("home/home.html")))
}
