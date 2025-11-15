//! Lobby message types (client -> server, server -> client)
use crate::models::redis::PlayerState;
use crate::models::redis::lobby_state::LobbyStatus;

/// Messages sent from clients to the lobby websocket.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LobbyClientMessage {
    Join,
    Leave,
    ToggleStart,
    Ping { ts: u64 },
}

/// Messages broadcast by the lobby server to connected clients.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum LobbyServerMessage {
    LobbyState {
        state: LobbyStatus,
        joined_players: Option<Vec<PlayerState>>,
        started: bool,
    },
    PlayerUpdated {
        players: Vec<PlayerState>,
    },
    Error {
        message: String,
    },
}
