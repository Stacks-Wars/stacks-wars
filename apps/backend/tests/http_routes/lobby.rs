use redis::AsyncCommands;
use reqwest;
use serde_json::json;

#[tokio::test]
async fn create_lobby() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    // create user and token using factory
    let factory = app.factory();
    let (user_id, token) = factory
        .create_test_user(Some("lobby-test-wallet"))
        .await
        .expect("create user failed");

    // create a game to attach the lobby to via factory
    let (creator_id, _t) = factory
        .create_test_user(Some("lobby-game-creator"))
        .await
        .expect("create creator failed");
    let game_id = factory
        .create_test_game(creator_id, Some("lobby-game"))
        .await
        .expect("create game failed")
        .to_string();

    // create lobby via API
    let lobby_payload = json!({
        "name": "test lobby",
        "description": "desc",
        "entryAmount": 0.0,
        "tokenSymbol": "STX",
        "tokenContractId": null,
        "contractAddress": null,
        "isPrivate": false,
        "isSponsored": false,
        "gameId": game_id
    });

    let resp = client
        .post(format!("{}/api/lobby", app.base_url))
        .bearer_auth(&token)
        .json(&lobby_payload)
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 201);
    let body: serde_json::Value = resp.json().await.expect("invalid json");
    let lobby_id = body
        .get("lobbyId")
        .and_then(|v| v.as_str())
        .expect("missing lobbyId");

    // verify Redis runtime state exists for the lobby and creator player
    {
        let mut conn = app.state.redis.get().await.expect("redis conn");
        let lobby_key = stacks_wars_be::models::redis::RedisKey::lobby_state(lobby_id);
        let exists: bool = conn.exists(&lobby_key).await.expect("redis exists");
        assert!(exists, "lobby state missing in redis");

        let player_key = stacks_wars_be::models::redis::RedisKey::lobby_player(lobby_id, user_id);
        let pexists: bool = conn.exists(&player_key).await.expect("redis exists");
        assert!(pexists, "creator player state missing in redis");
    }

    app.stop().await;
}

#[tokio::test]
async fn get_lobby() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (creator_id, _t) = factory
        .create_test_user(Some("get-lobby-creator"))
        .await
        .expect("create user failed");
    let game_id = factory
        .create_test_game(creator_id, Some("get-lobby-game"))
        .await
        .expect("create game failed");

    let lobby_id = factory
        .create_test_lobby(creator_id, game_id, Some("factory-lobby"))
        .await
        .expect("create lobby failed");

    let resp = client
        .get(format!("{}/api/lobby/{}", app.base_url, lobby_id))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());

    app.stop().await;
}

#[tokio::test]
async fn list_lobbies_by_game() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (creator_id, _t) = factory
        .create_test_user(Some("list-lobby-creator"))
        .await
        .expect("create user failed");
    let game_id = factory
        .create_test_game(creator_id, Some("list-lobby-game"))
        .await
        .expect("create game failed");

    // create a couple lobbies for the game
    let _ = factory
        .create_test_lobby(creator_id, game_id, Some("lobby-1"))
        .await
        .expect("create lobby failed");
    let _ = factory
        .create_test_lobby(creator_id, game_id, Some("lobby-2"))
        .await
        .expect("create lobby failed");

    let resp = client
        .get(format!("{}/api/game/{}/lobbies", app.base_url, game_id))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());
    let arr: Vec<serde_json::Value> = resp.json().await.expect("invalid json");
    assert!(arr.len() >= 2);

    app.stop().await;
}

#[tokio::test]
async fn list_my_lobbies() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    // create user and token using factory
    let factory = app.factory();
    let (_user_id, token) = factory
        .create_test_user(Some("owner-list-wallet"))
        .await
        .expect("create user failed");

    // create a game
    let game_creator = factory
        .create_test_user(Some("owner-list-game-creator"))
        .await
        .expect("create user failed");
    let game_id = factory
        .create_test_game(game_creator.0, Some("list-game"))
        .await
        .expect("create game failed");

    // create lobby via API as this user
    let lobby_payload = json!({
        "name": "my lobby",
        "description": "owned lobby",
        "entryAmount": 1.0,
        "tokenSymbol": "STX",
        "isSponsored": false,
        "gameId": game_id.to_string()
    });

    let resp = client
        .post(format!("{}/api/lobby", app.base_url))
        .bearer_auth(&token)
        .json(&lobby_payload)
        .send()
        .await
        .expect("request failed");
    assert_eq!(resp.status().as_u16(), 201);
    let body: serde_json::Value = resp.json().await.expect("invalid json");
    let lobby_id = body
        .get("lobbyId")
        .and_then(|v| v.as_str())
        .expect("missing lobbyId");

    // list my lobbies and assert created one is present
    let resp = client
        .get(format!("{}/api/lobby/my", app.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());
    let arr: Vec<serde_json::Value> = resp.json().await.expect("invalid json");
    let found = arr
        .iter()
        .any(|v| v.get("id").and_then(|id| id.as_str()) == Some(lobby_id));
    assert!(found, "created lobby not found in my lobbies");

    app.stop().await;
}

#[tokio::test]
async fn delete_lobby() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    // create user and token using factory
    let factory = app.factory();
    let (_user_id, token) = factory
        .create_test_user(Some("owner-delete-wallet"))
        .await
        .expect("create user failed");

    // create a game
    let game_creator = factory
        .create_test_user(Some("owner-delete-game-creator"))
        .await
        .expect("create user failed");
    let game_id = factory
        .create_test_game(game_creator.0, Some("delete-game"))
        .await
        .expect("create game failed");

    // create lobby via API as this user
    let lobby_payload = json!({
        "name": "deletable lobby",
        "description": "to be deleted",
        "entryAmount": 1.0,
        "tokenSymbol": "STX",
        "isSponsored": false,
        "gameId": game_id.to_string()
    });

    let resp = client
        .post(format!("{}/api/lobby", app.base_url))
        .bearer_auth(&token)
        .json(&lobby_payload)
        .send()
        .await
        .expect("request failed");
    assert_eq!(resp.status().as_u16(), 201);
    let body: serde_json::Value = resp.json().await.expect("invalid json");
    let lobby_id = body
        .get("lobbyId")
        .and_then(|v| v.as_str())
        .expect("missing lobbyId");

    // delete the lobby
    let resp = client
        .delete(format!("{}/api/lobby/{}", app.base_url, lobby_id))
        .bearer_auth(&token)
        .send()
        .await
        .expect("request failed");
    assert_eq!(resp.status().as_u16(), 204);

    // ensure GET now fails (404)
    let resp = client
        .get(format!("{}/api/lobby/{}", app.base_url, lobby_id))
        .send()
        .await
        .expect("request failed");
    assert!(!resp.status().is_success());

    app.stop().await;
}
