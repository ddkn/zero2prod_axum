use once_cell::sync::Lazy;
use reqwest::Url;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::net::SocketAddr;
use std::{fs::remove_file, str::FromStr};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod_axum::settings::read_settings_file;
use zero2prod_axum::startup::Application;

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

pub struct TestDatabaseConnection {
    pub db_name: String,
    pub pool: SqlitePool,
}

pub struct TestApp {
    pub addr: SocketAddr,
    pub port: u16,
    pub db_name: String,
    pub email_server: MockServer,
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        let addr = self.addr;
        reqwest::Client::new()
            .post(&format!("http://{addr}/subscriptions"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_links(
        &self,
        email_request: &wiremock::Request,
    ) -> ConfirmationLinks {
        let body: serde_json::Value =
            serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = Url::parse(&raw_link).unwrap();
            // Let's make sure we don't call Random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            // Rewrite URL to include port
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());

        ConfirmationLinks { html, plain_text }
    }

    pub async fn post_newsletter(
        &self,
        body: serde_json::Value,
    ) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("http://{}/newsletters", &self.addr))
            .basic_auth(
                Uuid::new_v4().to_string(),
                Some(Uuid::new_v4().to_string()),
            )
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

/// spawn_app
///
/// Spawn's the app, which can be replaced with decoupled backend, for
/// example, Django. This allows us to change the backend implementation
/// but still use the testing pipline here as needed.
pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let db_conn = create_connect_test_db()
        .await
        .expect("Unable to create test database");

    let email_server = MockServer::start().await;

    // Randomise configuration to ensure test isolation
    // Neat, mut struct inside scope to a read-only variable
    let app_settings = {
        let mut settings =
            read_settings_file().expect("Failed to read settings file.");
        settings.database.name = db_conn.db_name.clone();
        settings.email_client.base_url = email_server.uri();
        // NOTE: Must bind to any available port, set to 0.
        // Otherwise all bound to same port and tests complain about used
        // port number.
        settings.port = 0u16;
        settings
    };

    let app = Application::build(app_settings.clone()).await.unwrap();
    let addr = app.address();
    let port = app.port();
    let _ = tokio::spawn(async move { app.run_until_stopped().await });

    TestApp {
        addr,
        port,
        db_name: db_conn.db_name,
        email_server,
    }
}

async fn create_connect_test_db() -> Result<TestDatabaseConnection, sqlx::Error>
{
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

    Ok(TestDatabaseConnection { db_name, pool })
}

pub async fn cleanup_test_db(db_name: String) -> Result<(), sqlx::Error> {
    remove_file(&db_name)?;
    Ok(())
}
