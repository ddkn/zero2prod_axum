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

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
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

pub fn parse_subscriber(sign_up: SignUp) -> Result<NewSubscriber, String> {
    let name = SubscriberName::parse(sign_up.name)?;
    let email = SubscriberEmail::parse(sign_up.email)?;
    Ok(NewSubscriber { email, name })
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
    let new_subscriber = match parse_subscriber(sign_up) {
        Ok(subscriber) => subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    match insert_subscriber(Extension(pool), &new_subscriber).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::BAD_REQUEST,
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    Extension(pool): Extension<SqlitePool>,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    let uuid = Uuid::new_v4().to_string();
    let current_time = get_current_utc_timestamp();

    let subscriber_name = new_subscriber.name.as_ref();
    let subscriber_email = new_subscriber.email.as_ref();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        uuid,
        subscriber_email,
        subscriber_name,
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
