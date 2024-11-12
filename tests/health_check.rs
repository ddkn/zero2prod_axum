//! health_check.rs

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use once_cell::sync::Lazy;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions},
    Connection, SqliteConnection,
};
use std::net::SocketAddr;
use std::{fs::remove_file, str::FromStr};
use tower::ServiceExt;
use uuid::Uuid;
use zero2prod_axum::{domain::SubscriberEmail, email_client::EmailClient};

const ADDR: &str = "127.0.0.1";
/// Bind to port 0 which causes the OS to hunt for an available port.
const PORT: u16 = 0;
const BASE_URL: &str = "localhost";
const SENDER_EMAIL: &str = "test@example.com";

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // Output of `get_subscriber` cannot be assigned to a variable since
    // the Sink is part of the return type, therefore,
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = zero2prod_axum::telemetry::get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::stdout,
        );
        zero2prod_axum::telemetry::init_subscriber(subscriber);
    } else {
        let subscriber = zero2prod_axum::telemetry::get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::sink,
        );
        zero2prod_axum::telemetry::init_subscriber(subscriber);
    }
});

/// Oneshot test
///
/// This requires tower to provide `.oneshot` as wel as http_body_util
/// to be able to `.collect()` the body.
#[tokio::test]
async fn health_check_oneshot() {
    let settings = zero2prod_axum::settings::read_settings_file(None)
        .expect("Failed to read settings file.");

    let connection_str = settings
        .database
        .connection_string()
        .expose_secret()
        .to_string();
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&connection_str)
        .await
        .expect("Failed to create database pool.");
    let sender = settings
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let email_client = EmailClient::new(
        settings.email_client.base_url,
        sender,
        settings.email_client.authorization_token,
    );

    let app = zero2prod_axum::startup::app(pool, email_client);

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
    let (addr, db_name) = spawn_app().await;

    let client = Client::new();
    let resp = client
        .get(format!("http://{addr}/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(resp.status().is_success());
    assert_eq!(resp.content_length(), Some(0));

    cleanup_test_db(db_name.clone()).await.expect(&format!(
        "Failure to delete test database {}",
        db_name.as_str()
    ));
}

/// Subscribe valid data
///
/// Checks for status code 200
#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let (addr, db_name) = spawn_app().await;

    let mut connection = SqliteConnection::connect(&db_name)
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

    cleanup_test_db(db_name.clone()).await.expect(&format!(
        "Failure to delete test database {}",
        db_name.as_str()
    ));
}

/// Subscribe missing data
///
/// `axum::Form`, unlike `actix_web::Form`, returns status code 422
/// when there is incomplete data. `actix_web::web::form` instead
/// returns a status code of 400.
#[tokio::test]
async fn subscribe_returns_400_for_missing_form_data() {
    let (addr, db_name) = spawn_app().await;
    let client = Client::new();

    let test_cases = vec![
        ("name=bird%20and%20boy&email=", "missing the e-mail"),
        ("name=&email=bb%40example.com", "missing the name"),
        ("name=birb&email=tyranosaurusrex", "invalid email format"),
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
            StatusCode::BAD_REQUEST,
            resp.status().as_u16(),
            "API did not fail with 400 Bad Request when the payload was {}: {}.",
            error_mesg, invalid_body
        );
    }

    cleanup_test_db(db_name.clone()).await.expect(&format!(
        "Failure to delete test database {}",
        db_name.as_str()
    ));
}

/// spawn_app
///
/// Spawn's the app, which can be replaced with decoupled backend, for
/// example, Django. This allows us to change the backend implementation
/// but still use the testing pipline here as needed.
async fn spawn_app() -> (SocketAddr, String) {
    Lazy::force(&TRACING);

    let (pool, db_name) = create_connect_test_db()
        .await
        .expect("Unable to create test database");

    // Tests require us to use port 0 for random ports otherwise all but one fail
    let port = PORT;
    let addr = ADDR;
    let bind_addr = format!("{}:{}", addr, port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    let addr = listener.local_addr().unwrap();
    let sender_email = SubscriberEmail::parse(SENDER_EMAIL.to_string())
        .expect("Invalid sender email!");
    let authorization_token = Secret::new("my-secret-token".to_string());
    let email_client = EmailClient::new(
        BASE_URL.to_string(),
        sender_email,
        authorization_token,
    );

    let _ = tokio::spawn(async move {
        axum::serve(listener, zero2prod_axum::startup::app(pool, email_client))
            .await
            .unwrap();
    });

    (addr, db_name)
}

async fn create_connect_test_db() -> Result<(SqlitePool, String), sqlx::Error> {
    let uuid_name = Uuid::new_v4();
    let db_name = format!("{}.db", uuid_name);
    let db_path = format!("sqlite://{}", db_name);

    let conn_opt =
        SqliteConnectOptions::from_str(&db_path)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(conn_opt)
        .await
        .expect("Failed to create database pool.");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate the database");

    Ok((pool, db_name))
}

async fn cleanup_test_db(db_name: String) -> Result<(), sqlx::Error> {
    remove_file(&db_name)?;
    Ok(())
}
