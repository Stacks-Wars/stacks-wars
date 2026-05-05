//! `/api/lobbies/*` — `http::lobbies`
use redis::AsyncCommands;
use reqwest;
use serde_json::json;

#[tokio::test]
async fn post_create_lobby_created() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (user_id, token) = factory
        .create_test_user(None)
        .await
        .expect("create user failed");

    let (creator_id, _t) = factory
        .create_test_user(None)
        .await
        .expect("create creator failed");
    let game_id = factory
        .create_test_game(creator_id, Some("lobby-game"))
        .await
        .expect("create game failed")
        .to_string();

    let lobby_payload = json!({
        "name": "test lobby",
        "description": "desc",
        "entryAmount": 0.0,
        "tokenSymbol": "STX",
        "tokenContractId": null,
        "contractAddress": null,
        "isPrivate": false,
        "isSponsored": false,
        "gameId": game_id,
        "gamePath": "lobby-game"
    });

    let resp = client
        .post(format!("{}/api/lobbies", app.base_url))
        .header("Cookie", factory.create_auth_cookie(&token))
        .json(&lobby_payload)
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status().as_u16(), 201);
    let body: serde_json::Value = resp.json().await.expect("invalid json");
    let lobby_id = body.get("id").and_then(|v| v.as_str()).expect("missing id");

    let lobby_path = body
        .get("path")
        .and_then(|v| v.as_str())
        .expect("missing path");
    assert!(!lobby_path.is_empty(), "path should be auto-generated");
    assert_eq!(lobby_path.len(), 8, "path should be 8 characters");

    {
        let mut conn = app.state.redis.get().await.expect("redis conn");
        let lobby_key = stacks_wars_server::models::RedisKey::lobby_state(lobby_id);
        let exists: bool = conn.exists(&lobby_key).await.expect("redis exists");
        assert!(exists, "lobby state missing in redis");

        let player_key = stacks_wars_server::models::RedisKey::lobby_player(lobby_id, user_id);
        let pexists: bool = conn.exists(&player_key).await.expect("redis exists");
        assert!(pexists, "creator player state missing in redis");
    }

    app.stop().await;
}

#[tokio::test]
async fn get_lobby_by_id_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (creator_id, _t) = factory
        .create_test_user(None)
        .await
        .expect("create user failed");
    let game_id = factory
        .create_test_game(creator_id, Some("get-lobby-game"))
        .await
        .expect("create game failed");

    let (lobby_id, _lobby_path) = factory
        .create_test_lobby(creator_id, game_id, Some("factory-lobby"))
        .await
        .expect("create lobby failed");

    let resp = client
        .get(format!("{}/api/lobbies/{}", app.base_url, lobby_id))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());

    app.stop().await;
}

#[tokio::test]
async fn get_lobbies_by_game_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (creator_id, _t) = factory
        .create_test_user(None)
        .await
        .expect("create user failed");
    let game_id = factory
        .create_test_game(creator_id, Some("list-lobby-game"))
        .await
        .expect("create game failed");

    let _ = factory
        .create_test_lobby(creator_id, game_id, Some("lobby-1"))
        .await
        .expect("create lobby failed");
    let _ = factory
        .create_test_lobby(creator_id, game_id, Some("lobby-2"))
        .await
        .expect("create lobby failed");

    let resp = client
        .get(format!(
            "{}/api/lobbies/game/{}/lobbies",
            app.base_url, game_id
        ))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.expect("invalid json");
    let arr = body
        .get("data")
        .and_then(|v| v.as_array())
        .expect("paginated data");
    assert!(arr.len() >= 2);

    app.stop().await;
}

#[tokio::test]
async fn get_list_all_lobbies_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/api/lobbies", app.base_url))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.expect("json");
    assert!(body.get("data").and_then(|v| v.as_array()).is_some());

    app.stop().await;
}

#[tokio::test]
async fn get_my_lobbies_ok() {
    let app = crate::common::spawn_app_with_containers().await;
    let client = reqwest::Client::new();

    let factory = app.factory();
    let (_creator_id, token) = factory
        .create_test_user(None)
        .await
        .expect("create user failed");

    let game_creator = factory
        .create_test_user(Some("owner-list-game-creator"))
        .await
        .expect("create user failed");
    let game_id = factory
        .create_test_game(game_creator.0, Some("list-game"))
        .await
        .expect("create game failed");

    let lobby_payload = json!({
        "name": "my lobby",
        "description": "owned lobby",
        "entryAmount": 1.0,
        "tokenSymbol": "STX",
        "isSponsored": false,
        "gameId": game_id.to_string(),
        "gamePath": "list-game"
    });

    let resp = client
        .post(format!("{}/api/lobbies", app.base_url))
        .header("Cookie", factory.create_auth_cookie(&token))
        .json(&lobby_payload)
        .send()
        .await
        .expect("request failed");
    assert_eq!(resp.status().as_u16(), 201);
    let body: serde_json::Value = resp.json().await.expect("invalid json");
    let lobby_id = body.get("id").and_then(|v| v.as_str()).expect("missing id");

    let resp = client
        .get(format!("{}/api/lobbies/my", app.base_url))
        .header("Cookie", factory.create_auth_cookie(&token))
        .send()
        .await
        .expect("request failed");
    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.expect("invalid json");
    let arr = body
        .get("data")
        .and_then(|v| v.as_array())
        .expect("paginated data");
    let found = arr
        .iter()
        .any(|v| v.get("id").and_then(|id| id.as_str()) == Some(lobby_id));
    assert!(found, "created lobby not found in my lobbies");

    app.stop().await;
}
