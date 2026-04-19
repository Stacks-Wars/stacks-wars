//! `/api/seasons/*` read routes — `http::seasons`
use reqwest;

#[tokio::test]
async fn get_list_seasons_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let name = "integration-season-list";
    let _ = factory
        .create_test_season(Some(name))
        .await
        .expect("create season failed");

    let resp = client
        .get(format!("{}/api/seasons", app.base_url))
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
async fn get_current_season_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/api/seasons/current", app.base_url))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());

    app.stop().await;
}
