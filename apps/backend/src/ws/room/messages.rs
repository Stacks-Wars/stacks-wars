// Room message types (client -> server, server -> client)
use crate::db::join_request::JoinRequest;
use crate::models::lobby_state::LobbyStatus;
use crate::models::{ChatMessage, LobbyInfo, PlayerState};
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
    #[serde(rename_all = "camelCase")]
    ApproveJoin {
        user_id: Uuid,
    },
    /// Creator rejects a join request
    #[serde(rename_all = "camelCase")]
    RejectJoin {
        user_id: Uuid,
    },
    /// Creator kicks a participant
    #[serde(rename_all = "camelCase")]
    Kick {
        user_id: Uuid,
    },
    /// Send a chat message
    #[serde(rename_all = "camelCase")]
    SendMessage {
        content: String,
        reply_to: Option<Uuid>,
    },
    /// Add a reaction to a message
    #[serde(rename_all = "camelCase")]
    AddReaction {
        message_id: Uuid,
        emoji: String,
    },
    /// Remove a reaction from a message
    #[serde(rename_all = "camelCase")]
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
///
/// Lobby-level messages are sent without wrapper.
/// Game-specific messages should use GameMessage wrapper.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum RoomServerMessage {
    #[serde(rename_all = "camelCase")]
    LobbyBootstrap {
        lobby_info: LobbyInfo,
        players: Vec<PlayerState>,
        join_requests: Vec<JoinRequest>,
        chat_history: Vec<ChatMessage>,
    },

    /// Generic lobby state change
    #[serde(rename_all = "camelCase")]
    LobbyStatusChanged {
        status: LobbyStatus,
        participant_count: usize,
        current_amount: Option<f64>,
    },

    /// Countdown updates
    #[serde(rename_all = "camelCase")]
    StartCountdown {
        seconds_remaining: u8,
    },

    #[serde(rename_all = "camelCase")]
    PlayerJoined {
        user_id: Uuid,
    },

    #[serde(rename_all = "camelCase")]
    PlayerLeft {
        user_id: Uuid,
    },

    #[serde(rename_all = "camelCase")]
    PlayerKicked {
        user_id: Uuid,
    },

    /// Broadcasted list of join requests (visible to lobby); only creator may accept/reject
    #[serde(rename_all = "camelCase")]
    JoinRequestsUpdated {
        join_requests: Vec<JoinRequest>,
    },

    /// Personal status for a join request
    #[serde(rename_all = "camelCase")]
    JoinRequestStatus {
        user_id: Uuid,
        accepted: bool,
    },

    /// Chat message received
    MessageReceived {
        message: ChatMessage,
    },

    /// Reaction added to a message
    #[serde(rename_all = "camelCase")]
    ReactionAdded {
        message_id: Uuid,
        user_id: Uuid,
        emoji: String,
    },

    /// Reaction removed from a message
    #[serde(rename_all = "camelCase")]
    ReactionRemoved {
        message_id: Uuid,
        user_id: Uuid,
        emoji: String,
    },

    /// Personal pong response; elapsed_ms = now.saturating_sub(client_ts)
    #[serde(rename_all = "camelCase")]
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

/// Wrapper for game-specific messages.
///
/// Frontend router uses the `game` field to route messages to correct game plugin.
///
/// Example:
/// ```json
/// {
///   "game": "lexi-wars",
///   "type": "wordSubmitted",
///   "payload": { "word": "hello", "points": 5, "valid": true }
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameMessage {
    /// Game identifier (e.g., "lexi-wars", "coin-flip")
    pub game: String,
    /// Message type specific to the game
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Game-specific payload
    pub payload: serde_json::Value,
}

impl GameMessage {
    /// Create a new game message
    pub fn new(game: String, msg_type: String, payload: serde_json::Value) -> Self {
        Self {
            game,
            msg_type,
            payload,
        }
    }

    /// Create a game message from a serializable payload
    pub fn with_payload<T: serde::Serialize>(
        game: String,
        msg_type: String,
        payload: T,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            game,
            msg_type,
            payload: serde_json::to_value(payload)?,
        })
    }
}
