//! src/routes/subscriptions.rs

use axum::{http::StatusCode, response::IntoResponse, Extension, Form};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SignUp {
    name: String,
    email: String,
}

fn get_current_utc_timestamp() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub async fn subscriptions(
    Extension(pool): Extension<SqlitePool>,
    Form(sign_up): Form<SignUp>,
) -> impl IntoResponse {
    let uuid = Uuid::new_v4().to_string();
    let current_time = get_current_utc_timestamp();

    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        uuid,
        sign_up.email,
        sign_up.name,
        current_time
    )
    .execute(&pool)
    .await
    .expect("Unable to insert subscription.");

    StatusCode::OK
}
