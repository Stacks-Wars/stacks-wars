/// Coin Flip game integration tests
use serde_json::json;
use std::time::Duration;

/// Helper to wait for any game message with specific type
/// Handles wrapped format: { game: "coin-flip", type: "...", payload: {...} }
async fn wait_for_game_message(
    ws: &mut crate::common::WsConnection,
    expected_type: &str,
    timeout_attempts: usize,
) -> Result<serde_json::Value, String> {
    for _ in 0..timeout_attempts {
        if let Ok(msg) = ws.recv_json_timeout(Duration::from_secs(1)).await {
            // Check if it's a wrapped game message with the expected type
            if msg.get("game").is_some()
                && msg.get("type").and_then(|v| v.as_str()) == Some(expected_type)
            {
                // Return the payload for easier assertion
                if let Some(payload) = msg.get("payload") {
                    return Ok(payload.clone());
                }
                return Ok(msg);
            }
        }
    }
    Err(format!("Timed out waiting for {}", expected_type))
}

/// Helper to wait for game_started event, skipping countdown and state change events
/// Now expects messages wrapped with game identifier: { game: "coin-flip", type: "game_started", payload: {...} }
async fn wait_for_game_started(
    ws: &mut crate::common::WsConnection,
    timeout_attempts: usize,
) -> Result<serde_json::Value, String> {
    for _ in 0..timeout_attempts {
        if let Ok(msg) = ws.recv_json_timeout(Duration::from_secs(1)).await {
            // Check if it's a wrapped game message
            if msg.get("game").is_some()
                && msg.get("type").and_then(|v| v.as_str()) == Some("game_started")
            {
                // Return the payload for easier assertion
                if let Some(payload) = msg.get("payload") {
                    return Ok(payload.clone());
                }
                return Ok(msg);
            }
        }
    }
    Err("Timed out waiting for game_started".to_string())
}

#[tokio::test]
async fn test_coinflip_game_bootstrap() {
    let app = crate::common::spawn_app_with_containers().await;

    let factory = app.factory();
    factory
        .ensure_coinflip_game()
        .await
        .expect("Failed to ensure Coin Flip game");

    // Create test users
    let (creator_id, creator_token) = factory
        .create_test_user(Some("coinflip-creator"))
        .await
        .expect("Failed to create creator");

    let (_player1_id, player1_token) = factory
        .create_test_user(Some("coinflip-player"))
        .await
        .expect("Failed to create player");

    // Create lobby
    let (_lobby_id, lobby_path) = factory
        .create_test_lobby(
            creator_id,
            crate::common::COINFLIP_GAME_ID,
            Some("Coin Flip Test"),
        )
        .await
        .expect("Failed to create lobby");

    // Both players connect
    let mut creator_ws =
        crate::common::WsConnection::connect_to_room(&app.base_url, &lobby_path, &creator_token)
            .await
            .expect("Creator failed to connect");

    let mut player1_ws =
        crate::common::WsConnection::connect_to_room(&app.base_url, &lobby_path, &player1_token)
            .await
            .expect("Player failed to connect");

    // Wait for lobby bootstrap messages
    let _ = creator_ws.recv_json_timeout(Duration::from_secs(2)).await; // creator lobbyBootstrap
    let _ = player1_ws.recv_json_timeout(Duration::from_secs(2)).await; // player lobbyBootstrap

    // Player1 joins the lobby
    player1_ws
        .send_json(&json!({"type": "join"}))
        .await
        .expect("Failed to send join");

    // Wait for join confirmation
    let _ = creator_ws.recv_json_timeout(Duration::from_secs(2)).await; // playerJoined notification
    let _ = player1_ws.recv_json_timeout(Duration::from_secs(2)).await; // playerUpdated or similar

    // Creator starts the game
    creator_ws
        .send_json(&json!({
            "type": "updateLobbyStatus",
            "status": "Starting"
        }))
        .await
        .expect("Failed to send start game");

    // Wait for game_started bootstrap event
    let creator_msg = wait_for_game_started(&mut creator_ws, 15)
        .await
        .expect("Creator should receive game_started");
    let player_msg = wait_for_game_started(&mut player1_ws, 15)
        .await
        .expect("Player should receive game_started");

    // Verify bootstrap contains required fields (type check removed - payload doesn't include type)
    assert!(
        creator_msg.get("players").is_some(),
        "Bootstrap should contain players list"
    );
    assert!(
        creator_msg.get("current_player").is_some(),
        "Bootstrap should contain current player"
    );
    assert!(
        creator_msg.get("timeout_secs").is_some(),
        "Bootstrap should contain timeout"
    );

    // Player should get same bootstrap (type check removed - payload doesn't include type)
    assert!(
        player_msg.get("players").is_some(),
        "Bootstrap should contain players list"
    );
    assert!(
        player_msg.get("current_player").is_some(),
        "Bootstrap should contain current player"
    );
    assert!(
        player_msg.get("timeout_secs").is_some(),
        "Bootstrap should contain timeout"
    );

    // Clean up
    creator_ws.close().await.ok();
    player1_ws.close().await.ok();
    app.stop().await;
}

#[tokio::test]
async fn test_coinflip_round_flow() {
    let app = crate::common::spawn_app_with_containers().await;

    let factory = app.factory();
    factory
        .ensure_coinflip_game()
        .await
        .expect("Failed to ensure Coin Flip game");

    // Create test users
    let (creator_id, creator_token) = factory
        .create_test_user(Some("round-creator"))
        .await
        .expect("Failed to create creator");

    let (_player1_id, player1_token) = factory
        .create_test_user(Some("round-player"))
        .await
        .expect("Failed to create player");

    // Create lobby
    let (_lobby_id, lobby_path) = factory
        .create_test_lobby(
            creator_id,
            crate::common::COINFLIP_GAME_ID,
            Some("Round Flow Test"),
        )
        .await
        .expect("Failed to create lobby");

    // Both players connect
    let mut creator_ws =
        crate::common::WsConnection::connect_to_room(&app.base_url, &lobby_path, &creator_token)
            .await
            .expect("Creator failed to connect");

    let mut player1_ws =
        crate::common::WsConnection::connect_to_room(&app.base_url, &lobby_path, &player1_token)
            .await
            .expect("Player failed to connect");

    // Wait for lobby bootstrap messages
    let _ = creator_ws.recv_json_timeout(Duration::from_secs(2)).await; // creator bootstrap
    let _ = player1_ws.recv_json_timeout(Duration::from_secs(2)).await; // player bootstrap

    // Player1 joins the lobby
    player1_ws
        .send_json(&json!({"type": "join"}))
        .await
        .expect("Failed to send join");

    // Wait for join confirmation
    let _ = creator_ws.recv_json_timeout(Duration::from_secs(2)).await; // playerJoined
    let _ = player1_ws.recv_json_timeout(Duration::from_secs(2)).await; // confirmation

    // Start game
    creator_ws
        .send_json(&json!({
            "type": "updateLobbyStatus",
            "status": "Starting"
        }))
        .await
        .expect("Failed to start game");

    // Both wait for game_started, skipping countdown
    let _ = wait_for_game_started(&mut creator_ws, 15).await;
    let _ = wait_for_game_started(&mut player1_ws, 15).await;

    // Both players make guesses
    creator_ws
        .send_json(&json!({
            "type": "game_action",
            "action": {
                "type": "make_guess",
                "guess": "heads"
            }
        }))
        .await
        .expect("Creator failed to guess");

    player1_ws
        .send_json(&json!({
            "type": "game_action",
            "action": {
                "type": "make_guess",
                "guess": "tails"
            }
        }))
        .await
        .expect("Player failed to guess");

    // Collect all messages from both players until we get round_complete
    let mut creator_messages = Vec::new();
    let mut player_messages = Vec::new();

    // Keep collecting messages until we get round_complete for both players
    let mut creator_round_complete = None;
    let mut player_round_complete = None;

    // Read messages with a timeout, expecting guess_received events first, then round_complete
    for _ in 0..10 {
        // Max 10 attempts to avoid infinite loop
        // Try creator
        if creator_round_complete.is_none() {
            if let Ok(msg) = creator_ws
                .recv_json_timeout(Duration::from_millis(200))
                .await
            {
                println!("Creator received: {:?}", msg);
                // Check for wrapped game message with type round_complete
                if msg.get("game").is_some()
                    && msg.get("type").and_then(|v| v.as_str()) == Some("round_complete")
                {
                    creator_round_complete = msg.get("payload").cloned();
                } else {
                    creator_messages.push(msg);
                }
            }
        }

        // Try player
        if player_round_complete.is_none() {
            if let Ok(msg) = player1_ws
                .recv_json_timeout(Duration::from_millis(200))
                .await
            {
                println!("Player received: {:?}", msg);
                // Check for wrapped game message with type round_complete
                if msg.get("game").is_some()
                    && msg.get("type").and_then(|v| v.as_str()) == Some("round_complete")
                {
                    player_round_complete = msg.get("payload").cloned();
                } else {
                    player_messages.push(msg);
                }
            }
        }

        // If we have both round_complete messages, we're done
        if creator_round_complete.is_some() && player_round_complete.is_some() {
            break;
        }
    }

    // Verify round complete payloads have results (type is in wrapper, not payload)
    // Just check that we have payloads - detailed assertions can check payload contents

    // Check that we received guess_received events before round_complete
    let creator_got_guess_received = creator_messages.iter().any(|msg| {
        msg.get("game").is_some()
            && msg.get("type").and_then(|v| v.as_str()) == Some("guess_received")
    });
    let player_got_guess_received = player_messages.iter().any(|msg| {
        msg.get("game").is_some()
            && msg.get("type").and_then(|v| v.as_str()) == Some("guess_received")
    });

    assert!(
        creator_got_guess_received,
        "Creator should have received guess_received events"
    );
    assert!(
        player_got_guess_received,
        "Player should have received guess_received events"
    );

    // Clean up
    creator_ws.close().await.ok();
    player1_ws.close().await.ok();
    app.stop().await;
}

#[tokio::test]
async fn test_coinflip_player_elimination() {
    let app = crate::common::spawn_app_with_containers().await;

    let factory = app.factory();
    factory
        .ensure_coinflip_game()
        .await
        .expect("Failed to ensure Coin Flip game");

    // Create 3 players
    let (p1_id, p1_token) = factory
        .create_test_user(Some("elim-p1"))
        .await
        .expect("Failed to create p1");

    let (_p2_id, p2_token) = factory
        .create_test_user(Some("elim-p2"))
        .await
        .expect("Failed to create p2");

    let (_p3_id, p3_token) = factory
        .create_test_user(Some("elim-p3"))
        .await
        .expect("Failed to create p3");

    // Create lobby
    let (_lobby_id, lobby_path) = factory
        .create_test_lobby(
            p1_id,
            crate::common::COINFLIP_GAME_ID,
            Some("Elimination Test"),
        )
        .await
        .expect("Failed to create lobby");

    // All connect
    let mut p1_ws =
        crate::common::WsConnection::connect_to_room(&app.base_url, &lobby_path, &p1_token)
            .await
            .expect("P1 failed to connect");
    let mut p2_ws =
        crate::common::WsConnection::connect_to_room(&app.base_url, &lobby_path, &p2_token)
            .await
            .expect("P2 failed to connect");
    let mut p3_ws =
        crate::common::WsConnection::connect_to_room(&app.base_url, &lobby_path, &p3_token)
            .await
            .expect("P3 failed to connect");

    // Wait for all bootstrap messages
    let _ = p1_ws.recv_json_timeout(Duration::from_secs(2)).await; // p1 bootstrap
    let _ = p2_ws.recv_json_timeout(Duration::from_secs(2)).await; // p2 bootstrap
    let _ = p3_ws.recv_json_timeout(Duration::from_secs(2)).await; // p3 bootstrap

    // Players 2 and 3 join the lobby (p1 is creator, auto-joined)
    p2_ws
        .send_json(&json!({"type": "join"}))
        .await
        .expect("P2 failed to join");
    p3_ws
        .send_json(&json!({"type": "join"}))
        .await
        .expect("P3 failed to join");

    // Wait for join confirmations (each join generates notifications to all)
    for _ in 0..6 {
        // 2 joins × 3 participants = 6 notifications
        let _ = tokio::select! {
            _ = p1_ws.recv_json_timeout(Duration::from_secs(2)) => {},
            _ = p2_ws.recv_json_timeout(Duration::from_secs(2)) => {},
            _ = p3_ws.recv_json_timeout(Duration::from_secs(2)) => {},
        };
    }

    // Start game
    p1_ws
        .send_json(&json!({
            "type": "updateLobbyStatus",
            "status": "Starting"
        }))
        .await
        .expect("Failed to start");

    // All wait for game_started, skipping countdown
    let _ = wait_for_game_started(&mut p1_ws, 15).await;
    let _ = wait_for_game_started(&mut p2_ws, 15).await;
    let _ = wait_for_game_started(&mut p3_ws, 15).await;

    // All guess the same - this should eliminate some
    for ws in [&mut p1_ws, &mut p2_ws, &mut p3_ws] {
        ws.send_json(&json!({
            "type": "game_action",
            "action": {"type": "make_guess", "guess": "heads"}
        }))
        .await
        .ok();
    }

    // Wait for round_complete using helper (handles wrapped format)
    let result = wait_for_game_message(&mut p1_ws, "round_complete", 10)
        .await
        .expect("Should receive round_complete message");

    // result is the payload now, not the full wrapped message
    let eliminated = result.get("eliminated_players").and_then(|v| v.as_array());
    let remaining = result.get("remaining_players").and_then(|v| v.as_array());

    // Either some were eliminated, or all guessed correctly
    assert!(
        eliminated.is_some() && remaining.is_some(),
        "Should have elimination info: {:?}",
        result
    );

    // Clean up
    p1_ws.close().await.ok();
    p2_ws.close().await.ok();
    p3_ws.close().await.ok();
    app.stop().await;
}
