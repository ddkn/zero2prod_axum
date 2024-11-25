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

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
};
use axum::{http::StatusCode, response::IntoResponse, Extension, Form};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SignUp {
    name: String,
    email: String,
}

impl TryFrom<SignUp> for NewSubscriber {
    type Error = String;

    fn try_from(value: SignUp) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(NewSubscriber { email, name })
    }
}

fn get_current_utc_timestamp() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(sign_up, pool, email_client),
    fields(
        subscriber_email = %sign_up.email,
        subscriber_name = %sign_up.name
    )
)]
pub async fn subscriptions(
    Extension(pool): Extension<SqlitePool>,
    Extension(email_client): Extension<Arc<EmailClient>>,
    Form(sign_up): Form<SignUp>,
) -> impl IntoResponse {
    let new_subscriber = match sign_up.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    if insert_subscriber(Extension(pool), &new_subscriber)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    let confirmation_link =
        "https://there-is-no-such-domain.com/subscriptions/confirm";
    // Send a (useless) email to the new subscriber.
    //We are ignoring e-mail delivery errors for now.
    if email_client
        .send_email(
            new_subscriber.email,
            "Welcome!",
            &format!(
                "Welcome to our newsletter!<br />\
                Click <a href=\"{}\">here</a> to confirm your subscription.",
                confirmation_link
            ),
            &format!(
                "Welcome to our newsletter!\n Visit {} to confirm your subscription.",
                confirmation_link),
        )
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
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
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
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
