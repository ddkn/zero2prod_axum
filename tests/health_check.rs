//! health_check.rs

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use reqwest::Client;
use std::net::SocketAddr;
use tower::ServiceExt;

const ADDR: &str = "127.0.0.1";
/// Bind to port 0 which causes the OS to hunt for an available port.
const PORT: u16 = 0;

/// Oneshot test
///
/// This requires tower to provide `.oneshot` as wel as http_body_util
/// to be able to `.collect()` the body.
#[tokio::test]
async fn health_check_oneshot() {
    let app = zero2prod_axum::app();

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/health_check")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"");
}

/// Client test
///
/// Tests proper http client requests.
#[tokio::test]
async fn health_check_works() {
    let addr = spawn_app().await;

    let client = Client::new();
    let resp = client
        .get(format!("http://{addr}/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(resp.status().is_success());
    assert_eq!(resp.content_length(), Some(0));
}

/// spawn_app
///
/// Spawn's the app, which can be replaced with decoupled backend, for
/// example, Django. This allows us to change the backend implementation
/// but still use the testing pipline here as needed.
async fn spawn_app() -> SocketAddr {
    let cli = zero2prod_axum::Cli {
        addr: ADDR.to_string(),
        port: PORT,
    };
    let bind_addr = format!("{}:{}", cli.addr.to_string(), cli.port);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    let addr = listener.local_addr().unwrap();

    let _ = tokio::spawn(async move {
        axum::serve(listener, zero2prod_axum::app()).await.unwrap();
    });

    addr
}
