use reqwest;
use serde_json::json;

#[tokio::test]
async fn create_game() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    // Create a user to act as the creator
    let factory = app.factory();
    let (_user_id, token) = factory
        .create_test_user(Some("create-game-wallet"))
        .await
        .expect("create user failed");

    let payload = json!({
        "name": "Integration Game",
        "path": "integration-game",
        "description": "A test game",
        "imageUrl": "https://example.com/img.png",
        "minPlayers": 1,
        "maxPlayers": 4,
        "category": "Word Games"
    });

    let resp = client
        .post(format!("{}/api/game", app.base_url))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("request failed");

    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.expect("invalid json");
    assert_eq!(
        body.get("name").and_then(|v| v.as_str()).unwrap_or(""),
        "Integration Game"
    );

    app.stop().await;
}

#[tokio::test]
async fn get_game() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    // Use factory to insert a game directly
    let factory = app.factory();
    // We need a creator user id
    let (creator_id, _token) = factory
        .create_test_user(Some("game-owner-wallet"))
        .await
        .expect("create user failed");

    let game_id = factory
        .create_test_game(creator_id, Some("factory-game"))
        .await
        .expect("create game failed");

    // GET single game
    let resp = client
        .get(format!("{}/api/game/{}", app.base_url, game_id))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.expect("invalid json");
    assert_eq!(
        body.get("id").and_then(|v| v.as_str()).unwrap_or(""),
        game_id.to_string()
    );

    app.stop().await;
}

#[tokio::test]
async fn list_games() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    // Ensure there is at least one game in DB (factory)
    let factory = app.factory();
    let (creator_id, _t) = factory
        .create_test_user(Some("list-game-owner"))
        .await
        .expect("create user failed");
    let _ = factory
        .create_test_game(creator_id, Some("list-game-1"))
        .await
        .expect("create game failed");

    let resp = client
        .get(format!("{}/api/game", app.base_url))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());
    let arr: Vec<serde_json::Value> = resp.json().await.expect("invalid json");
    assert!(arr.len() >= 1);

    app.stop().await;
}
