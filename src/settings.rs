//! src/settings.rs

use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub database_name: String,
}
