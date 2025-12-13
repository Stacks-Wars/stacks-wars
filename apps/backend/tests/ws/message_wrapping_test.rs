/// Unit tests for message wrapping functionality
use serde_json::json;

#[test]
fn test_game_message_structure() {
    // Simulate the wrapper format that our backend creates
    let game_message = json!({
        "game": "coin-flip",
        "type": "round_complete",
        "payload": {
            "winner": "player1",
            "result": "heads",
            "eliminated_players": [],
            "remaining_players": ["player1", "player2"]
        }
    });

    // Frontend will check for game field to route to correct plugin
    assert_eq!(
        game_message.get("game").and_then(|v| v.as_str()),
        Some("coin-flip")
    );
    assert_eq!(
        game_message.get("type").and_then(|v| v.as_str()),
        Some("round_complete")
    );

    // Frontend extracts payload for game-specific handling
    let payload = game_message.get("payload").expect("Should have payload");
    assert_eq!(
        payload.get("winner").and_then(|v| v.as_str()),
        Some("player1")
    );
}

#[test]
fn test_lobby_message_structure() {
    // Lobby messages remain unwrapped (no game field)
    let lobby_message = json!({
        "type": "playerJoined",
        "payload": {
            "userId": "123",
            "username": "test_user"
        }
    });

    // No game field means it's a lobby message
    assert!(lobby_message.get("game").is_none());
    assert_eq!(
        lobby_message.get("type").and_then(|v| v.as_str()),
        Some("playerJoined")
    );
}

#[test]
fn test_wrapped_vs_unwrapped_detection() {
    let game_msg = json!({
        "game": "lexi-wars",
        "type": "word_submitted",
        "payload": { "word": "test" }
    });

    let lobby_msg = json!({
        "type": "lobbyBootstrap",
        "payload": { "lobbyId": "123" }
    });

    // Detection logic that frontend uses
    let is_game_message = |msg: &serde_json::Value| msg.get("game").is_some();

    assert!(is_game_message(&game_msg), "Should detect game message");
    assert!(!is_game_message(&lobby_msg), "Should detect lobby message");
}
