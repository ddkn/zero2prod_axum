//! health_check.rs

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use reqwest::Client;
use sqlx::{sqlite::SqlitePoolOptions, Connection, SqliteConnection};
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
    let settings =
        zero2prod_axum::settings::read_settings_file(Some("settings.toml"))
            .expect("Failed to read settings file.");

    let connection_str = settings.database.connection_string();
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&connection_str)
        .await
        .expect("Failed to create database pool.");

    let app = zero2prod_axum::startup::app(pool);

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
async fn health_check_success() {
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

/// Subscribe valid data
///
/// Checks for status code 200
#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let addr = spawn_app().await;
    let settings =
        zero2prod_axum::settings::read_settings_file(Some("settings.toml"))
            .expect("Failed to read settings file.");
    let connection_str = settings.database.connection_string();
    let mut connection = SqliteConnection::connect(&connection_str)
        .await
        .expect("Failed to connect to database.");

    let client = Client::new();

    let body = "name=bird%20and%20boy&email=bnb@example.com";

    let resp = client
        .post(&format!("http://{addr}/subscriptions"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(StatusCode::OK, resp.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "bnb@example.com");
    assert_eq!(saved.name, "bird and boy");
}

/// Subscribe missing data
///
/// `axum::Form`, unlike `actix_web::Form`, returns status code 422
/// when there is incomplete data. `actix_web::web::form` instead
/// returns a status code of 400.
#[tokio::test]
async fn subscribe_returns_400_for_missing_form_data() {
    let addr = spawn_app().await;
    let client = Client::new();

    let test_cases = vec![
        ("name=bird%20and%20boy", "missing the e-mail"),
        ("email=bb%40example.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_mesg) in test_cases {
        let resp = client
            .post(&format!("http://{addr}/subscriptions"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            StatusCode::UNPROCESSABLE_ENTITY,
            resp.status().as_u16(),
            "API did not fail with 400 Bad Request when the payload was {}.",
            error_mesg
        );
    }
}

/// spawn_app
///
/// Spawn's the app, which can be replaced with decoupled backend, for
/// example, Django. This allows us to change the backend implementation
/// but still use the testing pipline here as needed.
async fn spawn_app() -> SocketAddr {
    let settings =
        zero2prod_axum::settings::read_settings_file(Some("settings.toml"))
            .expect("Failed to read settings file.");

    let connection_str = settings.database.connection_string();
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&connection_str)
        .await
        .expect("Failed to create database pool.");

    // Tests require us to use port 0 for random ports otherwise all but one fail
    let port = PORT;
    let addr = ADDR;
    let bind_addr = format!("{}:{}", addr, port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    let addr = listener.local_addr().unwrap();

    let _ = tokio::spawn(async move {
        axum::serve(listener, zero2prod_axum::startup::app(pool))
            .await
            .unwrap();
    });

    addr
}
