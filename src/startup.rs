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
use crate::routes::{confirm, health_check, publish_newsletter, subscriptions};
use crate::settings::AppSettings;
use axum::{
    http::Request,
    routing::{get, post},
    Extension, Router,
};
use secrecy::ExposeSecret;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
// use tower::{Service, ServiceBuilder, ServiceExt};
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::Level;

pub struct Application {
    port: u16,
    router: Router,
    listener: TcpListener,
}

// Need to wrap base url to prevent raw `String` conflicts on access.
#[derive(Clone)]
pub struct ApplicationBaseUrl(pub String);

pub fn app(
    pool: SqlitePool,
    email_client: EmailClient,
    base_url: String,
) -> Router {
    // wrap client in Arc for multiple handlers
    let shared_client = Arc::new(email_client);
    let base_url = ApplicationBaseUrl(base_url);
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
        .route("/subscriptions/confirm", get(confirm))
        .route("/newsletters", post(publish_newsletter))
        .layer(Extension(pool))
        // Use Extension to add the Arc<Reqwest::Client>
        // if using multiple Reqwest::Client, then order matters
        // or wrap in a unique struct, e.g., struct ClientA(Client)
        .layer(Extension(shared_client))
        .layer(Extension(base_url))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let request_id = uuid::Uuid::new_v4().to_string();

                    tracing::span!(
                        Level::DEBUG,
                        "request",
                        %request_id,
                        method = ?request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_response(DefaultOnResponse::new().include_headers(true)),
        )
}

impl Application {
    pub async fn build(settings: AppSettings) -> Result<Self, std::io::Error> {
        let addr = &settings.addr;
        let port = settings.port;
        let connection_str = settings
            .database
            .connection_string()
            .expose_secret()
            .to_string();
        // Naive way to create a binded address
        let bind_addr = format!("{}:{}", addr, port);

        let sender = settings
            .email_client
            .sender()
            .expect("Invalid sender email!");
        let timeout = settings.email_client.timeout();
        let email_client = EmailClient::new(
            settings.email_client.base_url.clone(),
            sender,
            settings.email_client.authorization_token.clone(),
            timeout,
        );

        let conn_opt = SqliteConnectOptions::from_str(&connection_str)
            .expect("Failed to create sqlite connection.")
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect_with(conn_opt)
            .await
            .expect("Failed to create database pool.");

        // Run app using hyper while listening onto the configured port
        tracing::info!("Listening on {}", port);
        let listener = tokio::net::TcpListener::bind(bind_addr).await?;
        let base_url = settings.normalized_base_url().unwrap();
        Ok(Self {
            port,
            router: app(pool, email_client, base_url.into()),
            listener,
        })
    }

    pub fn port(&self) -> u16 {
        let addr = self.listener.local_addr().unwrap();
        addr.port()
    }

    pub fn address(&self) -> SocketAddr {
        self.listener.local_addr().unwrap()
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        axum::serve(self.listener, self.router).await
    }
}
