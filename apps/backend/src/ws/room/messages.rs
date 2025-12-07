// Room message types (client -> server, server -> client)
use crate::db::join_request::JoinRequestDTO;
use crate::models::lobby_state::LobbyStatus;
use crate::models::{ChatMessage, LobbyExtended, PlayerState};
use crate::ws::room::error::RoomError;
use uuid::Uuid;

/// Messages sent from clients to the lobby websocket.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RoomClientMessage {
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
    /// Send a chat message
    SendMessage {
        content: String,
        reply_to: Option<Uuid>,
    },
    /// Add a reaction to a message
    AddReaction {
        message_id: Uuid,
        emoji: String,
    },
    /// Remove a reaction from a message
    RemoveReaction {
        message_id: Uuid,
        emoji: String,
    },
    /// Heartbeat from client; `ts` is client's timestamp in milliseconds
    Ping {
        ts: u64,
    },
}

/// Messages broadcast by the lobby server to connected clients.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum RoomServerMessage {
    LobbyBootstrap {
        lobby: LobbyExtended,
        players: Vec<PlayerState>,
        join_requests: Vec<JoinRequestDTO>,
        chat_history: Vec<ChatMessage>,
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

    /// Chat message received
    MessageReceived {
        message: ChatMessage,
    },

    /// Reaction added to a message
    ReactionAdded {
        message_id: Uuid,
        user_id: Uuid,
        emoji: String,
    },

    /// Reaction removed from a message
    ReactionRemoved {
        message_id: Uuid,
        user_id: Uuid,
        emoji: String,
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

impl From<RoomError> for RoomServerMessage {
    fn from(err: RoomError) -> Self {
        RoomServerMessage::Error {
            code: err.code().to_string(),
            message: err.to_string(),
        }
    }
}
