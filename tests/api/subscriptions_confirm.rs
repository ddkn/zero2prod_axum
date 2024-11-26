use crate::helpers::spawn_app;
use axum::http::StatusCode;
use reqwest::Url;
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
    let body: serde_json::Value =
        serde_json::from_slice(&email_request.body).unwrap();
    // Extract the link from one of the request fields.
    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let raw_confirmation_link = &get_link(&body["HtmlBody"].as_str().unwrap());
    let mut confirmation_link = Url::parse(raw_confirmation_link).unwrap();
    // Let's make sure we don't call Random APIs on the web
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
    // Rewrite URL to include port
    confirmation_link.set_port(Some(app.port)).unwrap();

    let resp = reqwest::get(confirmation_link.clone()).await.unwrap();

    assert_eq!(resp.status().as_u16(), 200);
}
