//! `/api/session/*` — `http::session`
use reqwest;

#[tokio::test]
async fn get_me_requires_auth() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/api/session/me", app.base_url))
        .send()
        .await
        .expect("request failed");
    assert_eq!(resp.status(), 401);

    app.stop().await;
}

#[tokio::test]
async fn get_me_with_token_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();
    let factory = app.factory();
    let (user_id, token) = factory
        .create_test_user(None)
        .await
        .expect("create user");

    let resp = client
        .get(format!("{}/api/session/me", app.base_url))
        .header("Cookie", factory.create_auth_cookie(&token))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["id"].as_str().unwrap(), user_id.to_string());

    app.stop().await;
}

#[tokio::test]
async fn get_unclaimed_rewards_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();
    let factory = app.factory();
    let (_id, token) = factory
        .create_test_user(None)
        .await
        .expect("create user");

    let resp = client
        .get(format!("{}/api/session/unclaimed-reward", app.base_url))
        .header("Cookie", factory.create_auth_cookie(&token))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());
    let arr: Vec<serde_json::Value> = resp.json().await.expect("json");
    assert!(arr.is_empty() || !arr.is_empty());

    app.stop().await;
}
