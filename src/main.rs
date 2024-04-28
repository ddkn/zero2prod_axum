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

use axum::{
    extract::Path, http::StatusCode, response::IntoResponse, routing::get,
    Router,
};
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// ip address
    #[clap(short, long, default_value = "127.0.0.1")]
    addr: String,
    /// ip port
    #[clap(short, long, default_value_t = 9000)]
    port: u16,
}

/// Greet the listner
///
/// # Parameters
/// - `name`: Name to be greeted as
///
/// # Returns
/// A String greeting
async fn greet(Path(name): Path<String>) -> String {
    format!("Hello {}!\n", name)
}

/// Health check
///
/// Alive server check for external services.
///
/// # Returns
/// `(SuccessCode::OK, "")`
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "")
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let addr = cli.addr;
    let port = cli.port.to_string();
    // Naive way to create a binded address
    let bind_addr = format!("{}:{}", addr, port);

    // Define single routes for now
    let app = Router::new()
        .route(
            "/",
            get(|| async {
                "Welcome to an Axum Zero to Production implementation!\n"
            }),
        )
        .route("/:name", get(greet))
        .route("/health_check", get(health_check));

    // Run app using hyper while listening onto the configured port
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
