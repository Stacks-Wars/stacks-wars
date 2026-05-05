//! `GET /health`, `GET /` — `http::public`
use reqwest;

#[tokio::test]
async fn get_health_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/health", app.base_url))
        .send()
        .await
        .expect("request failed");
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.expect("body");
    assert_eq!(body, "OK");

    app.stop().await;
}

#[tokio::test]
async fn get_root_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/", app.base_url))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());
    let v: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(v.get("name").and_then(|x| x.as_str()), Some("Stacks Wars API"));
    assert_eq!(v.get("status").and_then(|x| x.as_str()), Some("running"));

    app.stop().await;
}
