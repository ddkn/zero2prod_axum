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

//! zero2prod_axum is an implementation of Zero to Production in Axum.
//!
//! # Table of contents
//!
//! - [Introduction](#introduction)
//!
//! # Introduction
//!
//! For those interested, this project is building a web server to serve
//! a E-mail Newsletter. It is very bare bones and has the capabilities,
//! * Send newsletter to subscribers
//! * Allow authors to send emails to subscribers

use clap::Parser;
use secrecy::{ExposeSecret, Secret};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
use zero2prod_axum::{
    domain::SubscriberEmail,
    email_client::EmailClient,
    settings,
    startup::{app, Cli},
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() {
    let subscriber =
        get_subscriber("zero2prod_axum".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let cli = Cli::parse();

    let addr = cli.addr;
    let port = cli.port.to_string();
    let settings_file = cli.settings;
    let ignore_settings = cli.ignore_settings;

    let bind_addr: String;
    let connection_str: String;
    let email_client: EmailClient;
    if ignore_settings {
        bind_addr = format!("{}:{}", addr, port);
        // Naive way to create a binded address
        connection_str = "sqlite:demo.db".to_string();
        let base_url = "http://localhost/api".to_string();
        let subscriber_email =
            SubscriberEmail::parse("test@gmail.com".to_string())
                .expect("Invalid sender email!");
        let timeout = std::time::Duration::from_millis(10000);
        email_client = EmailClient::new(
            base_url,
            subscriber_email,
            Secret::new("my-secret-token".to_string()),
            timeout,
        );
    } else {
        let app_settings =
            settings::read_settings_file(settings_file.as_deref())
                .expect("Failed to read settings file.");
        let addr = app_settings.addr;
        let port = app_settings.port;
        connection_str = app_settings
            .database
            .connection_string()
            .expose_secret()
            .to_string();
        // Naive way to create a binded address
        bind_addr = format!("{}:{}", addr, port);

        let sender = app_settings
            .email_client
            .sender()
            .expect("Invalid sender email!");
        let timeout = app_settings.email_client.timeout();
        email_client = EmailClient::new(
            app_settings.email_client.base_url,
            sender,
            app_settings.email_client.authorization_token,
            timeout,
        );
    }

    let conn_opt = SqliteConnectOptions::from_str(&connection_str)
        .expect("Failed to create sqlite connection.")
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(conn_opt)
        .await
        .expect("Failed to create database pool.");

    tracing::info!("Listening on {}", port);
    // Run app using hyper while listening onto the configured port
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(listener, app(pool, email_client))
        .await
        .unwrap();
}
