use crate::models::redis::PlayerState as RedisPlayerState;
use crate::models::redis::lobby_state::LobbyStatus;
use serde::{Deserialize, Serialize};

/// Messages sent from clients to the lobby websocket.
///
/// Placed under `crate::models::lobby` so contributors can find all lobby
/// message shapes in one place. The handler re-exports this as
/// `crate::ws::handlers::lobby::types::ClientMessage` for backwards
/// compatibility with existing code in the `ws` layer.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LobbyClientMessage {
    Join,
    Leave,
    ToggleStart,
    Ping { ts: u64 },
}

/// Messages broadcast by the lobby server to connected clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum LobbyServerMessage {
    LobbyState {
        state: LobbyStatus,
        joined_players: Option<Vec<RedisPlayerState>>,
        started: bool,
    },
    PlayerUpdated {
        players: Vec<RedisPlayerState>,
    },
    Error {
        message: String,
    },
}
