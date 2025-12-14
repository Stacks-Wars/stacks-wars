use chrono::Utc;
use reqwest;
use serde_json::json;

#[tokio::test]
async fn create_season() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let name = "integration-season-create";
    let start = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let end = (Utc::now() + chrono::Duration::days(30))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let payload = json!({
        "name": name,
        "description": "created via api",
        "startDate": start,
        "endDate": end,
    });

    let resp = client
        .post(format!("{}/api/season", app.base_url))
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.expect("invalid json");
    let id = body.get("id").and_then(|v| v.as_i64()).expect("missing id");
    assert!(id > 0);

    app.stop().await;
}

#[tokio::test]
async fn get_list_seasons() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    // Ensure there is at least one season via factory
    let factory = app.factory();
    let name = "integration-season-list";
    let _ = factory
        .create_test_season(Some(name))
        .await
        .expect("create season failed");

    let resp = client
        .get(format!("{}/api/season", app.base_url))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.expect("invalid json");
    let found = body
        .as_array()
        .map(|arr| {
            arr.iter()
                .any(|v| v.get("name").and_then(|n| n.as_str()) == Some(name))
        })
        .unwrap_or(false);
    assert!(found, "created season not found in list");

    app.stop().await;
}

#[tokio::test]
async fn get_current_season() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/api/season/current", app.base_url))
        .send()
        .await
        .expect("request failed");
    // Some setups may return 200 with empty object; ensure we don't 500
    assert!(resp.status().is_success());

    app.stop().await;
}
