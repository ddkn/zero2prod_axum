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
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};
use zero2prod_axum::{
    settings,
    startup::{app, Cli},
};

#[tokio::main]
async fn main() {
    // Logging: add tracing subscriber with log options from RUST_LOG or fallback
    // This defaults to the following packages levels:
    // * zero2prod       = debug
    // * tower_http      = debug
    // * axum::rejection = trace
    tracing_subscriber::registry()
        .with(fmt::layer().pretty())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "zero2prod_axum=debug,tower_http=debug,axum::rejection=trace".into()
        }))
        .init();

    let cli = Cli::parse();

    let addr = cli.addr;
    let port = cli.port.to_string();
    let settings_file = cli.settings;
    let ignore_settings = cli.ignore_settings;

    let bind_addr: String;
    let connection_str: String;
    if ignore_settings {
        bind_addr = format!("{}:{}", addr, port);
        // Naive way to create a binded address
        connection_str = "sqlite:demo.db".to_string();
    } else {
        let app_settings =
            settings::read_settings_file(settings_file.as_deref())
                .expect("Failed to read settings file.");
        let addr = app_settings.addr;
        let port = app_settings.port;
        connection_str = app_settings.database.connection_string();
        // Naive way to create a binded address
        bind_addr = format!("{}:{}", addr, port);
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
    axum::serve(listener, app(pool)).await.unwrap();
}
