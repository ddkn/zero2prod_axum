//! src/routes/subscriptions.rs

use axum::{http::StatusCode, response::IntoResponse, Form};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SignUp {
    name: String,
    email: String,
}

pub async fn subscriptions(Form(sign_up): Form<SignUp>) -> impl IntoResponse {
    (
        StatusCode::OK,
        format!(
            "Hello {}, subscriptions will be sent to {}",
            sign_up.name, sign_up.email
        ),
    )
}
