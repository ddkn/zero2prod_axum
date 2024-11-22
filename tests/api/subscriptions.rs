use crate::helpers::{cleanup_test_db, spawn_app};
use axum::http::StatusCode;
use reqwest::Client;
use sqlx::{Connection, SqliteConnection};

/// Subscribe valid data
///
/// Checks for status code 200
#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;

    let mut connection = SqliteConnection::connect(&app.db_name)
        .await
        .expect("Failed to connect to database.");

    let body = "name=bird%20and%20boy&email=bnb@example.com";
    let resp = app.post_subscriptions(body.into()).await;

    assert_eq!(StatusCode::OK, resp.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "bnb@example.com");
    assert_eq!(saved.name, "bird and boy");

    cleanup_test_db(app.db_name.clone()).await.expect(&format!(
        "Failure to delete test database {}",
        app.db_name.as_str()
    ));
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
