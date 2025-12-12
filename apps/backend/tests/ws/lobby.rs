// Lobby WebSocket integration tests (/ws/lobbies)
// Tests for browsing available lobbies with status-based filtering
// Run with: `cargo test --test ws::lobby`

use crate::common;

use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_lobby_list_connection_and_initial_list() {
    let app = common::spawn_app_with_containers().await;

    // Ensure Coin Flip game exists
    let factory = app.factory();
    factory
        .ensure_coinflip_game()
        .await
        .expect("Failed to ensure Coin Flip game");

    // Create test user
    let (creator_id, creator_token) = factory
        .create_test_user(Some("list-viewer"))
        .await
        .expect("Failed to create user");

    // Create a few lobbies
    let (lobby1_id, _lobby1_path) = factory
        .create_test_lobby(creator_id, common::COINFLIP_GAME_ID, Some("Test Lobby 1"))
        .await
        .expect("Failed to create lobby 1");

    let (lobby2_id, _lobby2_path) = factory
        .create_test_lobby(creator_id, common::COINFLIP_GAME_ID, Some("Test Lobby 2"))
        .await
        .expect("Failed to create lobby 2");

    // Connect to lobby list without filter
    let mut lobby_list_ws =
        common::WsConnection::connect_to_lobby(&app.base_url, Some(&creator_token), None)
            .await
            .expect("Failed to connect to lobby list");

    // Should receive initial lobby list
    let initial_list = lobby_list_ws
        .recv_json_timeout(Duration::from_secs(2))
        .await
        .expect("Should receive initial lobby list");

    assert_eq!(
        initial_list.get("type").and_then(|v| v.as_str()),
        Some("lobbyList")
    );

    let lobbies = initial_list
        .get("lobbies")
        .and_then(|v| v.as_array())
        .expect("Should have lobbies array");

    // Should include our created lobbies
    assert!(
        lobbies.len() >= 2,
        "Should have at least 2 lobbies, got {}",
        lobbies.len()
    );

    // Verify our lobbies are in the list
    let lobby_ids: Vec<String> = lobbies
        .iter()
        .filter_map(|l| l.get("id").and_then(|id| id.as_str()))
        .map(String::from)
        .collect();

    assert!(
        lobby_ids.contains(&lobby1_id.to_string()),
        "List should contain lobby 1"
    );
    assert!(
        lobby_ids.contains(&lobby2_id.to_string()),
        "List should contain lobby 2"
    );

    // Clean up
    lobby_list_ws.close().await.ok();
    app.stop().await;
}

#[tokio::test]
async fn test_lobby_list_status_filter() {
    let app = common::spawn_app_with_containers().await;

    let factory = app.factory();
    factory
        .ensure_coinflip_game()
        .await
        .expect("Failed to ensure Coin Flip game");

    // Create test users
    let (creator_id, creator_token) = factory
        .create_test_user(Some("filter-tester"))
        .await
        .expect("Failed to create user");

    // Create a waiting lobby
    let (waiting_lobby_id, _waiting_lobby_path) = factory
        .create_test_lobby(creator_id, common::COINFLIP_GAME_ID, Some("Waiting Lobby"))
        .await
        .expect("Failed to create lobby");

    // Connect to lobby list with status filter for "Waiting" lobbies only
    let mut lobby_list_ws = common::WsConnection::connect_to_lobby(
        &app.base_url,
        Some(&creator_token),
        Some(&["Waiting"]),
    )
    .await
    .expect("Failed to connect to lobby list with filter");

    // Should receive filtered lobby list
    let filtered_list = lobby_list_ws
        .recv_json_timeout(Duration::from_secs(2))
        .await
        .expect("Should receive filtered lobby list");

    assert_eq!(
        filtered_list.get("type").and_then(|v| v.as_str()),
        Some("lobbyList")
    );

    let lobbies = filtered_list
        .get("lobbies")
        .and_then(|v| v.as_array())
        .expect("Should have lobbies array");

    // All lobbies in the list should have "Waiting" status
    for lobby in lobbies {
        let status = lobby
            .get("status")
            .and_then(|s| s.as_str())
            .expect("Lobby should have status");
        assert_eq!(status, "Waiting", "All lobbies should be in Waiting state");
    }

    // Verify our waiting lobby is in the list
    let lobby_ids: Vec<String> = lobbies
        .iter()
        .filter_map(|l| l.get("id").and_then(|id| id.as_str()))
        .map(String::from)
        .collect();

    assert!(
        lobby_ids.contains(&waiting_lobby_id.to_string()),
        "List should contain our waiting lobby"
    );

    // Clean up
    lobby_list_ws.close().await.ok();
    app.stop().await;
}

#[tokio::test]
async fn test_lobby_list_subscribe_update() {
    let app = common::spawn_app_with_containers().await;

    let factory = app.factory();
    factory
        .ensure_coinflip_game()
        .await
        .expect("Failed to ensure Coin Flip game");

    // Create test user
    let (creator_id, creator_token) = factory
        .create_test_user(Some("subscribe-tester"))
        .await
        .expect("Failed to create user");

    // Create a lobby
    let (_lobby_id, _lobby_path) = factory
        .create_test_lobby(creator_id, common::COINFLIP_GAME_ID, Some("Subscribe Test"))
        .await
        .expect("Failed to create lobby");

    // Connect to lobby list without filter
    let mut lobby_list_ws =
        common::WsConnection::connect_to_lobby(&app.base_url, Some(&creator_token), None)
            .await
            .expect("Failed to connect to lobby list");

    // Receive initial list
    let _initial = lobby_list_ws
        .recv_json_timeout(Duration::from_secs(2))
        .await
        .expect("Should receive initial list");

    // Send Subscribe message to update filter to "Waiting" only
    lobby_list_ws
        .send_json(&json!({
            "type": "subscribe",
            "status": ["Waiting"]
        }))
        .await
        .expect("Failed to send subscribe");

    // Should receive updated lobby list with new filter
    let updated_list = lobby_list_ws
        .recv_json_timeout(Duration::from_secs(2))
        .await
        .expect("Should receive updated lobby list");

    assert_eq!(
        updated_list.get("type").and_then(|v| v.as_str()),
        Some("lobbyList")
    );

    let lobbies = updated_list
        .get("lobbies")
        .and_then(|v| v.as_array())
        .expect("Should have lobbies array");

    // All lobbies should be in Waiting state
    for lobby in lobbies {
        let status = lobby
            .get("status")
            .and_then(|s| s.as_str())
            .expect("Lobby should have status");
        assert_eq!(
            status, "Waiting",
            "After subscribe, all lobbies should be in Waiting state"
        );
    }

    // Clean up
    lobby_list_ws.close().await.ok();
    app.stop().await;
}

#[tokio::test]
async fn test_lobby_list_load_more() {
    let app = common::spawn_app_with_containers().await;

    let factory = app.factory();
    factory
        .ensure_coinflip_game()
        .await
        .expect("Failed to ensure Coin Flip game");

    // Create test user
    let (creator_id, creator_token) = factory
        .create_test_user(Some("load-more-tester"))
        .await
        .expect("Failed to create user");

    // Create multiple lobbies (more than the initial page size)
    for i in 0..15 {
        factory
            .create_test_lobby(
                creator_id,
                common::COINFLIP_GAME_ID,
                Some(&format!("Lobby {}", i)),
            )
            .await
            .expect("Failed to create lobby");
    }

    // Connect to lobby list
    let mut lobby_list_ws =
        common::WsConnection::connect_to_lobby(&app.base_url, Some(&creator_token), None)
            .await
            .expect("Failed to connect to lobby list");

    // Receive initial list (default limit is 12)
    let initial_list = lobby_list_ws
        .recv_json_timeout(Duration::from_secs(2))
        .await
        .expect("Should receive initial list");

    let initial_lobbies = initial_list
        .get("lobbies")
        .and_then(|v| v.as_array())
        .expect("Should have lobbies array");

    let initial_count = initial_lobbies.len();

    // Send LoadMore message
    lobby_list_ws
        .send_json(&json!({
            "type": "loadMore",
            "offset": initial_count
        }))
        .await
        .expect("Failed to send loadMore");

    // Should receive more lobbies
    let more_lobbies = lobby_list_ws
        .recv_json_timeout(Duration::from_secs(2))
        .await
        .expect("Should receive more lobbies");

    assert_eq!(
        more_lobbies.get("type").and_then(|v| v.as_str()),
        Some("lobbyList")
    );

    let additional_lobbies = more_lobbies
        .get("lobbies")
        .and_then(|v| v.as_array())
        .expect("Should have lobbies array");

    // Should have received additional lobbies
    assert!(
        additional_lobbies.len() > 0,
        "Should receive additional lobbies"
    );

    // Clean up
    lobby_list_ws.close().await.ok();
    app.stop().await;
}

#[tokio::test]
async fn test_lobby_list_unauthenticated_connection() {
    let app = common::spawn_app_with_containers().await;

    let factory = app.factory();
    factory
        .ensure_coinflip_game()
        .await
        .expect("Failed to ensure Coin Flip game");

    // Connect to lobby list without authentication (public access)
    let mut lobby_list_ws = common::WsConnection::connect_to_lobby(&app.base_url, None, None)
        .await
        .expect("Failed to connect to lobby list without auth");

    // Should still receive initial lobby list (public access)
    let initial_list = lobby_list_ws
        .recv_json_timeout(Duration::from_secs(2))
        .await
        .expect("Should receive initial lobby list without auth");

    assert_eq!(
        initial_list.get("type").and_then(|v| v.as_str()),
        Some("lobbyList")
    );

    // Clean up
    lobby_list_ws.close().await.ok();
    app.stop().await;
}
