use reqwest;
use serde_json::json;

#[tokio::test]
async fn get_leaderboard() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/api/leaderboard", app.base_url))
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());

    let body: serde_json::Value = resp.json().await.expect("failed to parse response");

    // Should return an array
    assert!(body.is_array(), "response should be an array");

    // If there are entries, verify the structure
    if let Some(first_entry) = body.as_array().and_then(|arr| arr.first()) {
        // Check that required fields are present
        assert!(first_entry.get("id").is_some(), "entry should have id");
        assert!(first_entry.get("seasonId").is_some(), "entry should have seasonId");
        assert!(first_entry.get("points").is_some(), "entry should have points");
        assert!(first_entry.get("userId").is_some(), "entry should have userId");
        assert!(first_entry.get("walletAddress").is_some(), "entry should have walletAddress");
        assert!(first_entry.get("email").is_some(), "entry should have email");
        assert!(first_entry.get("trustRating").is_some(), "entry should have trustRating");
        assert!(first_entry.get("createdAt").is_some(), "entry should have createdAt");
        assert!(first_entry.get("updatedAt").is_some(), "entry should have updatedAt");
    }
}

#[tokio::test]
async fn get_leaderboard_with_pagination() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/api/leaderboard?limit=5&offset=0", app.base_url))
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());

    let body: serde_json::Value = resp.json().await.expect("failed to parse response");

    // Should return an array
    assert!(body.is_array(), "response should be an array");

    // Should not exceed the limit
    if let Some(arr) = body.as_array() {
        assert!(arr.len() <= 5, "should not return more than 5 entries");
    }
}