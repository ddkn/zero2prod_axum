use crate::helpers::{cleanup_test_db, spawn_app};
use reqwest::Client;

#[tokio::test]
async fn health_check_success() {
    let app = spawn_app().await;

    let client = Client::new();
    let addr = app.addr;
    let resp = client
        .get(format!("http://{addr}/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(resp.status().is_success());
    assert_eq!(resp.content_length(), Some(0));

    cleanup_test_db(app.db_name.clone()).await.expect(&format!(
        "Failure to delete test database {}",
        app.db_name.as_str()
    ));
}
