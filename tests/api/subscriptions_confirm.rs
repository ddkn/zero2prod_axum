use crate::helpers::{cleanup_test_db, spawn_app};
use axum::http::StatusCode;
use sqlx::{Connection, SqliteConnection};
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    let app = spawn_app().await;

    let resp = reqwest::Client::new()
        .get(&format!("http://{}/subscriptions/confirm", app.addr))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(StatusCode::BAD_REQUEST, resp.status().as_u16());

    cleanup_test_db(app.db_name.clone()).await.expect(&format!(
        "Failure to delete test database {}",
        app.db_name.as_str()
    ));
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    let app = spawn_app().await;
    let body = "name=le%20_guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    assert_eq!(confirmation_links.html, confirmation_links.plain_text);

    cleanup_test_db(app.db_name.clone()).await.expect(&format!(
        "Failure to delete test database {}",
        app.db_name.as_str()
    ));
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    let app = spawn_app().await;
    let body = "name=le%20_guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    let resp = reqwest::get(confirmation_links.html).await.unwrap();

    assert_eq!(resp.status().as_u16(), 200);

    cleanup_test_db(app.db_name.clone()).await.expect(&format!(
        "Failure to delete test database {}",
        app.db_name.as_str()
    ));
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    let app = spawn_app().await;
    let mut connection = SqliteConnection::connect(&app.db_name)
        .await
        .expect("Failed to connect to database.");

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");

    cleanup_test_db(app.db_name.clone()).await.expect(&format!(
        "Failure to delete test database {}",
        app.db_name.as_str()
    ));
}
