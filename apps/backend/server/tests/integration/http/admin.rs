//! `/api/admin/*` — `http::admin`
use chrono::Utc;
use reqwest;
use serde_json::json;

use crate::common::TEST_ADMIN_WALLET;

#[tokio::test]
async fn post_create_season_admin_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (_admin_id, token) = factory
        .create_test_user(Some(TEST_ADMIN_WALLET))
        .await
        .expect("create admin user");

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
        .post(format!("{}/api/admin/season", app.base_url))
        .header("Cookie", factory.create_auth_cookie(&token))
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success(), "status={}", resp.status());
    let body: serde_json::Value = resp.json().await.expect("invalid json");
    let id = body.get("id").and_then(|v| v.as_i64()).expect("missing id");
    assert!(id > 0);

    app.stop().await;
}

#[tokio::test]
async fn put_update_season_admin_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();
    let factory = app.factory();

    let season_id = factory
        .create_test_season(Some("season-to-update"))
        .await
        .expect("create season");

    let (_admin_id, token) = factory
        .create_test_user(Some(TEST_ADMIN_WALLET))
        .await
        .expect("admin");

    let start = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let end = (Utc::now() + chrono::Duration::days(60))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let payload = json!({
        "name": "updated-season-name",
        "description": "updated via api",
        "startDate": start,
        "endDate": end,
    });

    let resp = client
        .put(format!(
            "{}/api/admin/season/{}",
            app.base_url, season_id
        ))
        .header("Cookie", factory.create_auth_cookie(&token))
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success(), "status={}", resp.status());
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(
        body.get("name").and_then(|v| v.as_str()),
        Some("updated-season-name")
    );

    app.stop().await;
}

#[tokio::test]
async fn patch_toggle_game_active_admin_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();
    let factory = app.factory();

    let (creator_id, _t) = factory.create_test_user(None).await.expect("user");
    let game_id = factory
        .create_test_game(creator_id, Some("toggle-active-game"))
        .await
        .expect("game");

    let (_admin_id, token) = factory
        .create_test_user(Some(TEST_ADMIN_WALLET))
        .await
        .expect("admin");

    let resp = client
        .patch(format!(
            "{}/api/admin/game/{}/active",
            app.base_url, game_id
        ))
        .header("Cookie", factory.create_auth_cookie(&token))
        .json(&json!({ "isActive": false }))
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success(), "status={}", resp.status());

    app.stop().await;
}
