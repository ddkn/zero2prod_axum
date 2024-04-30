//! src/startup.rs

use crate::routes::{health_check, subscriptions};
use axum::{
    routing::{get, post},
    Extension, Router,
};
use clap::Parser;
use sqlx::SqlitePool;

#[derive(Parser)]
pub struct Cli {
    /// ip address
    #[clap(short, long, default_value = "127.0.0.1")]
    pub addr: String,
    /// ip port
    #[clap(short, long, default_value_t = 9000)]
    pub port: u16,
    /// settings file
    #[clap(short, long)]
    pub settings: Option<String>,
    /// override settings file
    #[clap(short, long, action = clap::ArgAction::SetTrue)]
    pub ignore_settings: bool,
}

pub fn app(pool: SqlitePool) -> Router {
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
}
