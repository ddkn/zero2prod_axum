// Copyright 2024 David Kalliecharan
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Copyright (c) 2024 David Kalliecharan
//
// SPDX-License-Identifier: BSD-2-Clause

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

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(sign_up, pool),
    fields(
        subscriber_email = %sign_up.email,
        subscriber_name = %sign_up.name
    )
)]
pub async fn subscriptions(
    Extension(pool): Extension<SqlitePool>,
    Form(sign_up): Form<SignUp>,
) -> impl IntoResponse {
    match insert_subscriber(Extension(pool), Form(sign_up)).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(sign_up, pool)
)]
pub async fn insert_subscriber(
    Extension(pool): Extension<SqlitePool>,
    Form(sign_up): Form<SignUp>,
) -> Result<(), sqlx::Error> {
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
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
        // Using the `?` to return early if fn failed, i.e., sqlx::Error
    })?;
    Ok(())
}
