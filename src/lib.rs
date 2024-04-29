use axum::{
    extract::Path, http::StatusCode, response::IntoResponse, routing::get,
    Router,
};
use clap::Parser;
use std::io::Result;

#[derive(Parser)]
pub struct Cli {
    /// ip address
    #[clap(short, long, default_value = "127.0.0.1")]
    pub addr: String,
    /// ip port
    #[clap(short, long, default_value_t = 9000)]
    pub port: u16,
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

pub async fn run(cli: Cli) -> Result<()> {
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
    axum::serve(listener, app).await
}
