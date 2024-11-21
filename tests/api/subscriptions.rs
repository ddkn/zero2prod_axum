use crate::helpers::{cleanup_test_db, spawn_app};
use axum::http::StatusCode;
use reqwest::Client;
use sqlx::{Connection, SqliteConnection};

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
