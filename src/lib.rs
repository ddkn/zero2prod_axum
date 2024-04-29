use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Form, Router,
};
use clap::Parser;
use serde::Deserialize;

#[derive(Parser)]
pub struct Cli {
    /// ip address
    #[clap(short, long, default_value = "127.0.0.1")]
    pub addr: String,
    /// ip port
    #[clap(short, long, default_value_t = 9000)]
    pub port: u16,
}

#[derive(Deserialize)]
pub struct SignUp {
    name: String,
    email: String,
}

/// Greet the listner
///
/// # Parameters
/// - `name`: Name to be greeted as
///
/// # Returns
/// A String greeting
pub async fn greet(Path(name): Path<String>) -> String {
    format!("Hello {}!\n", name)
}

/// Health check
///
/// Alive server check for external services.
///
/// # Returns
/// `(SuccessCode::OK, "")`
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "")
}

pub async fn subscriptions(Form(sign_up): Form<SignUp>) -> impl IntoResponse {
    (
        StatusCode::OK,
        format!(
            "Hello {}, subscriptions will be sent to {}",
            sign_up.name, sign_up.email
        ),
    )
}

pub fn app() -> Router {
    // Define single routes for now
    Router::new()
        .route(
            "/",
            get(|| async {
                "Welcome to an Axum Zero to Production implementation!\n"
            }),
        )
        .route("/:name", get(greet))
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscriptions))
}
