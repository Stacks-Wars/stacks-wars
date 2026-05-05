//! `/api/stats/*` — `http::stats`
use reqwest;

#[tokio::test]
async fn get_platform_stats_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/api/stats", app.base_url))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.expect("json");
    assert!(body.get("totalUsers").is_some());

    app.stop().await;
}
