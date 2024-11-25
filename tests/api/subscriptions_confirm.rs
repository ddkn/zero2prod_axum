use crate::helpers::spawn_app;
use axum::http::StatusCode;

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
