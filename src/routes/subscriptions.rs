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
use tracing::Instrument;
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

    let request_span = tracing::trace_span!(
        "add_new_subscriber",
        subscriber_email = %sign_up.email,
        subscriber_name = %sign_up.name,
    );

    let _request_span_guard = request_span.enter();

    let query_span =
        tracing::info_span!("Saving new subscriber details into database");

    match sqlx::query!(
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
    .instrument(query_span)
    .await
    // .expect("Unable to insert subscription.");
    {
        Ok(_) => {
            StatusCode::OK
        },
        Err(e) => {
            tracing::error!("Failed to execute query {:?}", e);
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}
