use reqwest;
use serde_json::json;

#[tokio::test]
async fn create_user() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let payload = json!({ "walletAddress": "SP2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKNRV9EJ7" });

    let resp = client
        .post(format!("{}/api/user", app.base_url))
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());

    // Handler returns a JWT token string in the body
    let token: String = resp.json().await.expect("failed to parse token");
    assert!(!token.is_empty());
    app.stop().await;
}

#[tokio::test]
async fn update_or_set_username() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    // Use factory to create a user directly in DB and obtain token
    let factory = app.factory();
    let (user_id, token) = factory
        .create_test_user(Some("update-username-wallet"))
        .await
        .expect("create user failed");

    let payload = json!({ "username": "new-username" });

    let resp = client
        .patch(format!("{}/api/user/username", app.base_url))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());

    // Verify username was updated via GET
    let resp2 = client
        .get(format!("{}/api/user/{}", app.base_url, user_id))
        .send()
        .await
        .expect("request failed");
    assert!(resp2.status().is_success());
    let body: serde_json::Value = resp2.json().await.expect("invalid json");
    assert_eq!(
        body.get("username").and_then(|v| v.as_str()).unwrap_or(""),
        "new-username"
    );

    app.stop().await;
}

#[tokio::test]
async fn update_or_set_displayname() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (user_id, token) = factory
        .create_test_user(Some("update-display-wallet"))
        .await
        .expect("create user failed");

    let payload = json!({ "displayName": "Cool Player" });

    let resp = client
        .patch(format!("{}/api/user/display-name", app.base_url))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());

    let resp2 = client
        .get(format!("{}/api/user/{}", app.base_url, user_id))
        .send()
        .await
        .expect("request failed");
    assert!(resp2.status().is_success());
    let body: serde_json::Value = resp2.json().await.expect("invalid json");
    assert_eq!(
        body.get("displayName")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
        "Cool Player"
    );

    app.stop().await;
}

#[tokio::test]
async fn update_user_profile() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (user_id, token) = factory
        .create_test_user(Some("update-profile-wallet"))
        .await
        .expect("create user failed");

    let payload = json!({ "username": "profile-user", "displayName": "Profile Player" });

    let resp = client
        .patch(format!("{}/api/user/profile", app.base_url))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());

    let resp2 = client
        .get(format!("{}/api/user/{}", app.base_url, user_id))
        .send()
        .await
        .expect("request failed");
    assert!(resp2.status().is_success());
    let body: serde_json::Value = resp2.json().await.expect("invalid json");
    assert_eq!(
        body.get("username").and_then(|v| v.as_str()).unwrap_or(""),
        "profile-user"
    );
    assert_eq!(
        body.get("displayName")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
        "Profile Player"
    );

    app.stop().await;
}
