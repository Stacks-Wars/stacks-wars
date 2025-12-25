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

    // Check that Set-Cookie header is present
    let set_cookie_header = resp
        .headers()
        .get("set-cookie")
        .expect("Set-Cookie header should be present")
        .to_str()
        .expect("header should be valid string");

    assert!(
        set_cookie_header.contains("auth_token="),
        "auth_token cookie should be set"
    );
    assert!(
        set_cookie_header.contains("HttpOnly"),
        "cookie should be httpOnly"
    );
    assert!(
        set_cookie_header.contains("Path=/"),
        "cookie path should be /"
    );

    let body: serde_json::Value = resp.json().await.expect("failed to parse response");

    // Verify default email was created
    let email = body
        .get("email")
        .and_then(|v| v.as_str())
        .expect("missing email");
    assert_eq!(
        email,
        "SP2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKNRV9EJ7@stackswars.com"
    );
    let email_verified = body
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

    // Verify cookie is set
    let set_cookie_header = resp
        .headers()
        .get("set-cookie")
        .expect("Set-Cookie header should be present");
    assert!(set_cookie_header.to_str().unwrap().contains("auth_token="));

    let body: serde_json::Value = resp.json().await.expect("failed to parse response");
    let email = body
        .get("email")
        .and_then(|v| v.as_str())
        .expect("missing email");
    assert_eq!(email, "test@example.com");
    let email_verified = body
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
        .header("Cookie", factory.create_auth_cookie(&token))
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
        .header("Cookie", factory.create_auth_cookie(&token))
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
        .header("Cookie", factory.create_auth_cookie(&token))
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

#[tokio::test]
async fn logout_user() {
    let app = crate::common::spawn_app_with_containers().await;
    let factory = app.factory();

    let client = reqwest::Client::new();

    // Create a user and get auth token
    let (_user, token) = factory
        .create_test_user(None)
        .await
        .expect("create user failed");

    // First verify we can access authenticated endpoint
    let resp = client
        .get(format!("{}/api/lobby/my", app.base_url))
        .header("Cookie", factory.create_auth_cookie(&token))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());

    // Now logout
    let logout_resp = client
        .post(format!("{}/api/logout", app.base_url))
        .header("Cookie", factory.create_auth_cookie(&token))
        .send()
        .await
        .expect("logout request failed");

    assert_eq!(logout_resp.status(), reqwest::StatusCode::NO_CONTENT);

    // Check that the cookie was cleared
    let set_cookie_headers: Vec<_> = logout_resp
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|h| h.to_str().unwrap_or(""))
        .collect();

    let has_cleared_cookie = set_cookie_headers
        .iter()
        .any(|h| h.contains("auth_token=") && (h.contains("Max-Age=0") || h.contains("max-age=0")));

    assert!(has_cleared_cookie, "auth_token cookie should be cleared");

    // Now try to access authenticated endpoint with the revoked token
    let resp2 = client
        .get(format!("{}/api/lobby/my", app.base_url))
        .header("Cookie", factory.create_auth_cookie(&token))
        .send()
        .await
        .expect("request failed");

    // Should be unauthorized since token is revoked
    assert_eq!(resp2.status(), reqwest::StatusCode::UNAUTHORIZED);

    app.stop().await;
}
