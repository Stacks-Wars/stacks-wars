use reqwest;
use serde_json::json;

#[tokio::test]
async fn create_platform_rating() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (user_id, token) = factory
        .create_test_user(None)
        .await
        .expect("create user failed");

    let payload = json!({ "rating": 4, "comment": "Great platform" });

    let resp = client
        .post(format!("{}/api/platform-rating", app.base_url))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());

    // Verify via GET
    let resp2 = client
        .get(format!("{}/api/platform-rating/{}", app.base_url, user_id))
        .send()
        .await
        .expect("request failed");

    assert!(resp2.status().is_success());
    let body: serde_json::Value = resp2.json().await.expect("invalid json");
    assert_eq!(body.get("rating").and_then(|v| v.as_i64()).unwrap_or(0), 4);

    app.stop().await;
}

#[tokio::test]
async fn get_platform_rating() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (user_id, token) = factory
        .create_test_user(None)
        .await
        .expect("create user failed");

    // Insert rating directly
    let _id = factory
        .create_platform_rating(user_id, 3)
        .await
        .expect("create platform rating failed");

    let resp = client
        .get(format!("{}/api/platform-rating/{}", app.base_url, user_id))
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.expect("invalid json");
    assert_eq!(body.get("rating").and_then(|v| v.as_i64()).unwrap_or(0), 3);

    app.stop().await;
}

#[tokio::test]
async fn list_platform_ratings_and_filter() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();

    let (user1, _t1) = factory
        .create_test_user(None)
        .await
        .expect("create user failed");
    let (user2, _t2) = factory
        .create_test_user(None)
        .await
        .expect("create user failed");

    let _ = factory
        .create_platform_rating(user1, 5)
        .await
        .expect("create platform rating failed");
    let _ = factory
        .create_platform_rating(user2, 3)
        .await
        .expect("create platform rating failed");

    // List all
    let resp = client
        .get(format!("{}/api/platform-rating", app.base_url))
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());
    let list: Vec<serde_json::Value> = resp.json().await.expect("invalid json");
    // At least the two we inserted should be present
    assert!(list.len() >= 2);

    // Filter by rating=3
    let resp2 = client
        .get(format!("{}/api/platform-rating?rating=3", app.base_url))
        .send()
        .await
        .expect("request failed");
    assert!(resp2.status().is_success());
    let list2: Vec<serde_json::Value> = resp2.json().await.expect("invalid json");
    // All returned ratings should equal 3
    for item in list2.iter() {
        assert_eq!(item.get("rating").and_then(|v| v.as_i64()).unwrap_or(0), 3);
    }

    app.stop().await;
}

#[tokio::test]
async fn update_platform_rating() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (user_id, token) = factory
        .create_test_user(None)
        .await
        .expect("create user failed");

    // Create via API
    let payload = json!({ "rating": 2, "comment": "initial" });
    let resp = client
        .post(format!("{}/api/platform-rating", app.base_url))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());

    // Update via PATCH
    let payload2 = json!({ "rating": 1, "comment": "updated" });
    let resp2 = client
        .patch(format!("{}/api/platform-rating", app.base_url))
        .bearer_auth(&token)
        .json(&payload2)
        .send()
        .await
        .expect("request failed");
    assert!(resp2.status().is_success());
    let body: serde_json::Value = resp2.json().await.expect("invalid json");
    assert_eq!(body.get("rating").and_then(|v| v.as_i64()).unwrap_or(0), 1);

    app.stop().await;
}

#[tokio::test]
async fn delete_platform_rating() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (user_id, token) = factory
        .create_test_user(None)
        .await
        .expect("create user failed");

    // Create via API
    let payload = json!({ "rating": 2, "comment": "to-delete" });
    let resp = client
        .post(format!("{}/api/platform-rating", app.base_url))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());

    // Delete via API
    let resp3 = client
        .delete(format!("{}/api/platform-rating", app.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    assert!(resp3.status().is_success());

    // GET should now return 404
    let resp4 = client
        .get(format!("{}/api/platform-rating/{}", app.base_url, user_id))
        .send()
        .await
        .expect("request failed");
    assert_eq!(resp4.status().as_u16(), 404);

    app.stop().await;
}
