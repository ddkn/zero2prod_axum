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

//! src/startup.rs

use crate::email_client::EmailClient;
use crate::routes::{health_check, subscriptions};
use axum::{
    http::Request,
    routing::{get, post},
    Extension, Router,
};
use sqlx::SqlitePool;
use std::sync::Arc;
// use tower::{Service, ServiceBuilder, ServiceExt};
use tower_http::trace::TraceLayer;
use tracing::Level;

pub fn app(pool: SqlitePool, email_client: EmailClient) -> Router {
    // wrap client in Arc for multiple handlers
    let shared_client = Arc::new(email_client);
    // Define single routes for now
    Router::new()
        .route(
            "/",
            get(|| async {
                "Welcome to an Axum Zero to Production implementation!\n"
            }),
        )
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscriptions))
        .layer(Extension(pool))
        // Use Extension to add the Arc<Reqwest::Client>
        // if using multiple Reqwest::Client, then order matters
        // or wrap in a unique struct, e.g., struct ClientA(Client)
        .layer(Extension(shared_client))
        .layer(TraceLayer::new_for_http().make_span_with(
            |request: &Request<_>| {
                let request_id = uuid::Uuid::new_v4().to_string();

                tracing::span!(
                    Level::DEBUG,
                    "request",
                    %request_id,
                    method = ?request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                )
            },
        ))
}
