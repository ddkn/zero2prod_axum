use once_cell::sync::Lazy;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::net::SocketAddr;
use std::{fs::remove_file, str::FromStr};
use uuid::Uuid;
use zero2prod_axum::email_client::EmailClient;
use zero2prod_axum::settings::read_settings_file;

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

/// spawn_app
///
/// Spawn's the app, which can be replaced with decoupled backend, for
/// example, Django. This allows us to change the backend implementation
/// but still use the testing pipline here as needed.
pub async fn spawn_app() -> (SocketAddr, String) {
    Lazy::force(&TRACING);

    let (pool, db_name) = create_connect_test_db()
        .await
        .expect("Unable to create test database");

    let app_settings =
        read_settings_file(None).expect("Failed to read settings file.");
    let addr = app_settings.addr;
    // NOTE: Must bind to any available port, set to 0.
    // Otherwise all bound to same port and tests complain about used
    // port number.
    let port = 0u16;

    let sender = app_settings
        .email_client
        .sender()
        .expect("Invalid sender email!");
    let timeout = app_settings.email_client.timeout();
    let email_client = EmailClient::new(
        app_settings.email_client.base_url,
        sender,
        app_settings.email_client.authorization_token,
        timeout,
    );
    // Tests require us to use port 0 for random ports otherwise all but one fail
    let bind_addr = format!("{}:{}", addr, port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    let addr = listener.local_addr().unwrap();
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

pub async fn cleanup_test_db(db_name: String) -> Result<(), sqlx::Error> {
    remove_file(&db_name)?;
    Ok(())
}
