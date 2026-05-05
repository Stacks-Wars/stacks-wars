//! `/api/users/*` — `http::users`
use reqwest;
use serde_json::json;

#[tokio::test]
async fn post_register_creates_user() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let payload = json!({ "walletAddress": "SP2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKNRV9EJ7" });

    let resp = client
        .post(format!("{}/api/users/register", app.base_url))
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());

    let set_cookie_header = resp
        .headers()
        .get("set-cookie")
        .expect("Set-Cookie header should be present")
        .to_str()
        .expect("header should be valid string");

    assert!(set_cookie_header.contains("auth_token="));
    assert!(set_cookie_header.contains("HttpOnly"));
    assert!(set_cookie_header.contains("Path=/"));

    let body: serde_json::Value = resp.json().await.expect("failed to parse response");

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
async fn post_register_with_email() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let payload = json!({
        "walletAddress": "SP3K8BC0PPEVCV7NZ6QSRWPQ2JE9E5B6N3PA0KBR9",
        "emailAddress": "test@example.com"
    });

    let resp = client
        .post(format!("{}/api/users/register", app.base_url))
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());

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
async fn post_register_rejects_invalid_email() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let payload = json!({
        "walletAddress": "SP1HTBVD3JG9C05J7HBJTHGR0GGW7KXW28M5JS8QE",
        "emailAddress": "invalid-email"
    });

    let resp = client
        .post(format!("{}/api/users/register", app.base_url))
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert!(!resp.status().is_success());
    assert_eq!(resp.status(), 400);

    app.stop().await;
}

#[tokio::test]
async fn patch_username_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (user_id, token) = factory
        .create_test_user(Some("SP1AY6K3PQV5MRT6R4S671NWW2FRVPKM0BR162CT6"))
        .await
        .expect("create user failed");

    let payload = json!({ "username": "new_username" });

    let resp = client
        .patch(format!("{}/api/users/user/username", app.base_url))
        .header("Cookie", factory.create_auth_cookie(&token))
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());

    let resp2 = client
        .get(format!("{}/api/users/user/{}", app.base_url, user_id))
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
async fn patch_display_name_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (user_id, token) = factory
        .create_test_user(Some("SP2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKNRV9EJ7"))
        .await
        .expect("create user failed");

    let payload = json!({ "displayName": "Cool Player" });

    let resp = client
        .patch(format!("{}/api/users/user/display-name", app.base_url))
        .header("Cookie", factory.create_auth_cookie(&token))
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());

    let resp2 = client
        .get(format!("{}/api/users/user/{}", app.base_url, user_id))
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
async fn patch_profile_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (user_id, token) = factory
        .create_test_user(Some("SP3N2ZJX0KZR1D4YKN1ZVXMZJVN6H4JTVQPJK4Q6M"))
        .await
        .expect("create user failed");

    let payload = json!({ "username": "profile_user", "displayName": "Profile Player" });

    let resp = client
        .patch(format!("{}/api/users/user/profile", app.base_url))
        .header("Cookie", factory.create_auth_cookie(&token))
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());

    let resp2 = client
        .get(format!("{}/api/users/user/{}", app.base_url, user_id))
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
async fn get_user_by_id_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();
    let factory = app.factory();
    let (user_id, _token) = factory.create_test_user(None).await.expect("create user");

    let resp = client
        .get(format!("{}/api/users/user/{}", app.base_url, user_id))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());

    app.stop().await;
}

#[tokio::test]
async fn get_player_lobbies_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();
    let factory = app.factory();
    let (user_id, _token) = factory.create_test_user(None).await.expect("create user");

    let resp = client
        .get(format!(
            "{}/api/users/player-lobby/{}?status=waiting,starting,inProgress&limit=6&offset=0",
            app.base_url, user_id
        ))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());

    app.stop().await;
}

#[tokio::test]
async fn post_logout_revokes_session() {
    let app = crate::common::spawn_app_with_containers().await;
    let factory = app.factory();

    let client = reqwest::Client::new();

    let (_user, token) = factory
        .create_test_user(None)
        .await
        .expect("create user failed");

    let resp = client
        .get(format!("{}/api/lobbies/my", app.base_url))
        .header("Cookie", factory.create_auth_cookie(&token))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());

    let logout_resp = client
        .post(format!("{}/api/session/logout", app.base_url))
        .header("Cookie", factory.create_auth_cookie(&token))
        .send()
        .await
        .expect("logout request failed");

    assert_eq!(logout_resp.status(), reqwest::StatusCode::NO_CONTENT);

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

    let resp2 = client
        .get(format!("{}/api/lobbies/my", app.base_url))
        .header("Cookie", factory.create_auth_cookie(&token))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp2.status(), reqwest::StatusCode::UNAUTHORIZED);

    app.stop().await;
}
