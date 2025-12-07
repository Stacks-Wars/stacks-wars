// Lobby list message types (client -> server, server -> client)
use crate::models::{Lobby, LobbyStatus};
use serde::{Deserialize, Serialize};

/// Messages sent from clients to the lobby list websocket
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LobbyClientMessage {
    /// Subscribe to lobby list updates with filters
    Subscribe {
        #[serde(default)]
        status: Option<Vec<LobbyStatus>>,
        #[serde(default = "default_limit")]
        limit: usize,
    },
    /// Request next page of lobbies
    LoadMore { offset: usize },
}

fn default_limit() -> usize {
    12
}

/// Messages broadcast by the lobby list server to connected clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum LobbyServerMessage {
    /// Initial list of lobbies
    LobbyList {
        lobbies: Vec<Lobby>,
        total: usize,
    },

    /// New lobby created
    LobbyCreated {
        lobby: Lobby,
    },

    /// Lobby status changed
    LobbyUpdated {
        lobby: Lobby,
    },

    /// Lobby deleted/finished
    LobbyRemoved {
        lobby_id: uuid::Uuid,
    },

    Error {
        code: String,
        message: String,
    },
}
