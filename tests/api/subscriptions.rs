use crate::helpers::{cleanup_test_db, spawn_app};
use axum::http::StatusCode;
use sqlx::{Connection, SqliteConnection};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

/// Subscribe valid data
///
/// Checks for status code 200
#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;

    let body = "name=bird%20and%20boy&email=bnb@example.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let resp = app.post_subscriptions(body.into()).await;

    assert_eq!(StatusCode::OK, resp.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    let app = spawn_app().await;

    let mut connection = SqliteConnection::connect(&app.db_name)
        .await
        .expect("Failed to connect to database.");

    let body = "name=bird%20and%20boy&email=bnb@example.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "bnb@example.com");
    assert_eq!(saved.name, "bird and boy");
    assert_eq!(saved.status, "pending_confirmation");

    // cleanup_test_db(app.db_name.clone()).await.expect(&format!(
    //     "Failure to delete test database {}",
    //     app.db_name.as_str()
    // ));
}

/// Subscribe missing data
///
/// `axum::Form`, unlike `actix_web::Form`, returns status code 400
/// when there is incomplete data. `actix_web::web::form` instead
/// returns a status code of 400.
#[tokio::test]
async fn subscribe_returns_400_for_missing_form_data() {
    let app = spawn_app().await;

    let test_cases = vec![
        ("name=bird%20and%20boy&email=", "missing the e-mail"),
        ("name=&email=bb%40example.com", "missing the name"),
        ("name=birb&email=tyranosaurusrex", "invalid email format"),
    ];

    for (invalid_body, error_mesg) in test_cases {
        let resp = app.post_subscriptions(invalid_body.into()).await;

        assert_eq!(
            StatusCode::BAD_REQUEST,
            resp.status().as_u16(),
            "API did not fail with 400 Bad Request when the payload was {}: {}.",
            error_mesg, invalid_body
        );
    }

    cleanup_test_db(app.db_name.clone()).await.expect(&format!(
        "Failure to delete test database {}",
        app.db_name.as_str()
    ));
}

/// Subscribe missing fields
///
/// `axum::Form`, unlike `actix_web::Form`, returns status code 422
/// when there is missing fields. `actix_web::web::form` instead
/// returns a status code of 400.
#[tokio::test]
async fn subscribe_returns_422_for_missing_form_fields() {
    let app = spawn_app().await;

    let test_cases = vec![
        ("name=birb2", "No email provided"),
        ("email=birb@example.com", "No name provided"),
        ("", "missing both name and e-mail"),
    ];

    for (invalid_body, error_mesg) in test_cases {
        let resp = app.post_subscriptions(invalid_body.into()).await;

        assert_eq!(
            StatusCode::UNPROCESSABLE_ENTITY,
            resp.status().as_u16(),
            "API did not fail with 400 Bad Request when the payload was {}: {}.",
            error_mesg, invalid_body
        );
    }

    cleanup_test_db(app.db_name.clone()).await.expect(&format!(
        "Failure to delete test database {}",
        app.db_name.as_str()
    ));
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le@20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;

    // Assert
    // Mock asserts on drop
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        // We are not setting an expectation here anymore
        // The test is focust on another aspect of the app behaviour
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    // Assert
    // Get first intercepted request
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    // parse the body as JSON, starting from raw bytes
    let body: serde_json::Value =
        serde_json::from_slice(&email_request.body).unwrap();

    // Extract the link from one of the request fields using a .
    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let html_link = get_link(&body["HtmlBody"].as_str().unwrap());
    let text_link = get_link(&body["TextBody"].as_str().unwrap());
    // The two links should be identical
    assert_eq!(html_link, text_link);
}
