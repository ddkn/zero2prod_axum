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

pub fn read_settings_file(
    path: Option<&str>,
) -> Result<AppSettings, toml::de::Error> {
    let filename = path.unwrap_or("./settings.toml");
    let toml_str = fs::read_to_string(filename).unwrap();
    let settings: AppSettings = toml::from_str(&toml_str)?;
    Ok(settings)
}
