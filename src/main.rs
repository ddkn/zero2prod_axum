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
use zero2prod_axum::{
    settings,
    startup::{app, Cli},
};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let addr = cli.addr;
    let port = cli.port.to_string();
    let settings_file = cli.settings;
    let ignore_settings = cli.ignore_settings;

    let bind_addr: String;
    let db_name: String;
    if ignore_settings {
        bind_addr = format!("{}:{}", addr, port);
        // Naive way to create a binded address
        db_name = "./demo.db".to_string();
    } else {
        let app_settings =
            settings::read_settings_file(settings_file.as_deref())
                .expect("Failed to read settings file.");
        let addr = app_settings.addr;
        let port = app_settings.port;
        db_name = app_settings.database.name;
        // Naive way to create a binded address
        bind_addr = format!("{}:{}", addr, port);
    }

    // Run app using hyper while listening onto the configured port
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(listener, app()).await.unwrap();
}
