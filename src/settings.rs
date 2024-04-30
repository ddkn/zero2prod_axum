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

use serde::Deserialize;
use std::fs;

/// AppSettings
///
/// This controls the settings for the app itself.
#[derive(Deserialize, Debug)]
pub struct AppSettings {
    pub addr: String,
    pub port: u16,
    pub database: DatabaseSettings,
}

/// DatabaseSettings
///
/// This controls the settings for the Databse. Since we are using
/// `sqlite` we only need the database name. Otherwise we would put all
/// the database connection information here.
#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
    pub name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!("sqlite://{}", self.name)
    }
}

pub fn read_settings_file(
    path: Option<&str>,
) -> Result<AppSettings, toml::de::Error> {
    let filename = path.unwrap_or("./settings.toml");
    let toml_str = fs::read_to_string(filename).unwrap();
    let settings: AppSettings = toml::from_str(&toml_str)?;
    Ok(settings)
}
