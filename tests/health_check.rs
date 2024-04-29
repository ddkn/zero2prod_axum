//! health_check.rs

use std::io::Result;

const ADDR: &str = "127.0.0.1";
const PORT: u16 = 9000;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    spawn_app().await.expect("Failed to spaw our app.");

    // Use reqwest to do HTTP requests
    let client = reqwest::Client::new();

    // Act
    let bind_addr = format!("{}:{}", ADDR, PORT);
    let resp = client
        .get(format!("http://{}/health_check", bind_addr))
        .send()
        .await
        .expect("Failed to execut request.");

    assert!(resp.status().is_success());
    assert_eq!(resp.content_length(), Some(0));
}

/// spawn_app
///
/// Spawn's the app, which can be replaced with decoupled backend, for
/// example, Django. This allows us to change the backend implementation
/// but still use the testing pipline here as needed.
async fn spawn_app() -> Result<()> {
    let cli = zero2prod_axum::Cli {
        addr: ADDR.to_string(),
        port: PORT,
    };
    zero2prod_axum::run(cli).await
}
