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
    routes::error_chain_fmt,
    startup::ApplicationBaseUrl,
};
use anyhow::Context;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Form,
};
use chrono::{DateTime, Utc};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Deserialize;
use sqlx::SqlitePool;
use sqlx::{Executor, Sqlite, Transaction};
use std::{char, sync::Arc};
use uuid::Uuid;

#[derive(thiserror::Error)]
pub enum SubscriptionsError {
    // String or &String cannot use #[from] or #[source], requires `.map_err(...)`
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for SubscriptionsError {
    fn into_response(self) -> Response {
        // Can match here to give specific a `StatusCode`
        match self {
            SubscriptionsError::ValidationError(_) => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
            SubscriptionsError::UnexpectedError(_) => {
                // Avoid passing internal details to the user only use `tracing::error`
                tracing::error!(error = ?self, "Subscriptions error");
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        }
    }
}

// Implement this to utilize the error chaining printing
impl std::fmt::Debug for SubscriptionsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

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
    skip(sign_up, pool, email_client, base_url),
    fields(
        subscriber_email = %sign_up.email,
        subscriber_name = %sign_up.name
    )
)]
#[axum_macros::debug_handler]
pub async fn subscriptions(
    Extension(pool): Extension<SqlitePool>,
    Extension(email_client): Extension<Arc<EmailClient>>,
    Extension(base_url): Extension<ApplicationBaseUrl>,
    Form(sign_up): Form<SignUp>,
) -> Result<impl IntoResponse, SubscriptionsError> {
    let new_subscriber = sign_up
        .try_into()
        .map_err(SubscriptionsError::ValidationError)?;

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Sqlite connection from the pool.")?;
    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber)
        .await
        .context("Failed to insert new subscriber into the database")?;
    let subscription_token = generate_subscription_token();
    let _ = store_token(&mut transaction, subscriber_id, &subscription_token)
        .await
        .context(
            "Failed to store the confirmation token for a new subscriber.",
        )?;
    transaction.commit().await.context(
        "Failed to commit SQL transaction to store a new subscriber.",
    )?;
    send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        &subscription_token,
    )
    .await
    .context("Failed to send a confirmation email.")?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    // pool: &SqlitePool,
    transaction: &mut Transaction<'_, Sqlite>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    let subscriber_id_string = subscriber_id.to_string();
    let query = sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id_string,
    );
    // Can define `impl From<sqlx::Error> for StoreTokenError` and
    // propogate errors early with `?`
    transaction.execute(query).await?;

    Ok(())
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}subscriptions/confirm?subscription_token={subscription_token}",
        base_url
    );
    // Send a (useless) email to the new subscriber.
    //We are ignoring e-mail delivery errors for now.
    email_client
      .send_email(
          &new_subscriber.email,
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
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction)
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Sqlite>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    let current_time = get_current_utc_timestamp();

    let subscriber_id_string = subscriber_id.to_string();
    let subscriber_name = new_subscriber.name.as_ref();
    let subscriber_email = new_subscriber.email.as_ref();
    let query = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id_string,
        subscriber_email,
        subscriber_name,
        current_time
    );
    transaction.execute(query).await?;
    Ok(subscriber_id)
}
