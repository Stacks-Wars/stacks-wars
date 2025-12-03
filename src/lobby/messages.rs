// Lobby message types (client -> server, server -> client)
use crate::db::join_request::JoinRequestDTO;
use crate::lobby::error::LobbyError;
use crate::models::db::LobbyExtended;
use crate::models::redis::PlayerState;
use crate::models::redis::lobby_state::LobbyStatus;
use uuid::Uuid;

/// Messages sent from clients to the lobby websocket.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LobbyClientMessage {
    Join,
    Leave,
    UpdateLobbyStatus {
        status: LobbyStatus,
    },
    /// Request to join a private lobby
    JoinRequest,
    /// Creator accepts a join request
    ApproveJoin {
        player_id: Uuid,
    },
    /// Creator rejects a join request
    RejectJoin {
        player_id: Uuid,
    },
    /// Creator kicks a participant
    Kick {
        player_id: Uuid,
    },
    /// Heartbeat from client; `ts` is client's timestamp in milliseconds
    Ping {
        ts: u64,
    },
}

/// Messages broadcast by the lobby server to connected clients.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum LobbyServerMessage {
    LobbyBootstrap {
        lobby: LobbyExtended,
        players: Vec<PlayerState>,
        join_requests: Vec<JoinRequestDTO>,
    },

    /// Generic lobby state change
    LobbyStateChanged {
        state: LobbyStatus,
    },

    /// Countdown updates
    StartCountdown {
        seconds_remaining: u8,
    },

    PlayerJoined {
        player_id: Uuid,
    },
    PlayerLeft {
        player_id: Uuid,
    },
    PlayerKicked {
        player_id: Uuid,
    },

    /// Broadcasted list of join requests (visible to lobby); only creator may accept/reject
    JoinRequestsUpdated {
        join_requests: Vec<JoinRequestDTO>,
    },

    /// Personal status for a join request
    JoinRequestStatus {
        player_id: Uuid,
        accepted: bool,
    },

    /// Personal pong response; elapsed_ms = now.saturating_sub(client_ts)
    Pong {
        elapsed_ms: u64,
    },

    PlayerUpdated {
        players: Vec<PlayerState>,
    },

    Error {
        code: String,
        message: String,
    },
}

impl From<LobbyError> for LobbyServerMessage {
    fn from(err: LobbyError) -> Self {
        LobbyServerMessage::Error {
            code: err.code().to_string(),
            message: err.to_string(),
        }
    }
}
