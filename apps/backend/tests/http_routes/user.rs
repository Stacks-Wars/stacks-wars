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

    // Handler returns CreateUserResponse { user, token }
    let body: serde_json::Value = resp.json().await.expect("failed to parse response");
    let token = body
        .get("token")
        .and_then(|v| v.as_str())
        .expect("missing token");
    assert!(!token.is_empty());

    // Verify default email was created
    let user = body.get("user").expect("missing user");
    let email = user
        .get("email")
        .and_then(|v| v.as_str())
        .expect("missing email");
    assert_eq!(
        email,
        "SP2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKNRV9EJ7@stackswars.com"
    );
    let email_verified = user
        .get("emailVerified")
        .and_then(|v| v.as_bool())
        .expect("missing emailVerified");
    assert_eq!(email_verified, false);

    app.stop().await;
}

#[tokio::test]
async fn create_user_with_email() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let payload = json!({
        "walletAddress": "SP3K8BC0PPEVCV7NZ6QSRWPQ2JE9E5B6N3PA0KBR9",
        "emailAddress": "test@example.com"
    });

    let resp = client
        .post(format!("{}/api/user", app.base_url))
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());

    let body: serde_json::Value = resp.json().await.expect("failed to parse response");
    let user = body.get("user").expect("missing user");
    let email = user
        .get("email")
        .and_then(|v| v.as_str())
        .expect("missing email");
    assert_eq!(email, "test@example.com");
    let email_verified = user
        .get("emailVerified")
        .and_then(|v| v.as_bool())
        .expect("missing emailVerified");
    assert_eq!(email_verified, true);

    app.stop().await;
}

#[tokio::test]
async fn create_user_with_invalid_email() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let payload = json!({
        "walletAddress": "SP1HTBVD3JG9C05J7HBJTHGR0GGW7KXW28M5JS8QE",
        "emailAddress": "invalid-email"
    });

    let resp = client
        .post(format!("{}/api/user", app.base_url))
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    // Should return an error for invalid email
    assert!(!resp.status().is_success());
    assert_eq!(resp.status(), 400);

    app.stop().await;
}

#[tokio::test]
async fn update_or_set_username() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    // Use factory to create a user directly in DB and obtain token
    let factory = app.factory();
    let (user_id, token) = factory
        .create_test_user(Some("SP1AY6K3PQV5MRT6R4S671NWW2FRVPKM0BR162CT6"))
        .await
        .expect("create user failed");

    let payload = json!({ "username": "new_username" });

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
        "new_username"
    );

    app.stop().await;
}

#[tokio::test]
async fn update_or_set_displayname() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (user_id, token) = factory
        .create_test_user(Some("SP2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKNRV9EJ7"))
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
        .create_test_user(Some("SP3N2ZJX0KZR1D4YKN1ZVXMZJVN6H4JTVQPJK4Q6M"))
        .await
        .expect("create user failed");

    let payload = json!({ "username": "profile_user", "displayName": "Profile Player" });

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
        "profile_user"
    );
    assert_eq!(
        body.get("displayName")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
        "Profile Player"
    );

    app.stop().await;
}
