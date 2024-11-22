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

//! src/settings.rs

use crate::domain::SubscriberEmail;
use serde::Deserialize;
use std::fs;
// See p.122 in book for using Secret with postgres and passwords.
// As secrecy does not implement Display
use secrecy::Secret;
// use secrecy::ExposeSecret;

/// AppSettings
///
/// This controls the settings for the app itself.
#[derive(Deserialize, Debug)]
pub struct AppSettings {
    pub addr: String,
    pub port: u16,
    pub database: DatabaseSettings,
    pub email_client: EmailClientSettings,
}

/// DatabaseSettings
///
/// This controls the settings for the Databse. Since we are using
/// `sqlite` we only need the database name. Otherwise we would put all
/// the database connection information here.
#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
    pub name: String,
    // We need to use the `secrey::ExposeSecret`
    // for `password.expose_secret()`
    //pub password: Secret<String>,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!("sqlite://{}", self.name))
    }
}

#[derive(Deserialize, Debug)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: Secret<String>,
    pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. \
                Use either `local` or `production`.",
                other
            )),
        }
    }
}

pub fn read_settings_file() -> Result<AppSettings, toml::de::Error> {
    let env: Environment = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENV.");
    let filename = format!("./settings.{}.toml", env.as_str());
    let toml_str = fs::read_to_string(filename).unwrap();
    let settings: AppSettings = toml::from_str(&toml_str)?;
    Ok(settings)
}
