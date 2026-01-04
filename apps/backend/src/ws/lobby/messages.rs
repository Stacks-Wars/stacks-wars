// Lobby list message types (client -> server, server -> client)
use crate::models::{LobbyInfo, LobbyStatus};
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
    #[serde(rename_all = "camelCase")]
    LobbyList {
        lobby_info: Vec<LobbyInfo>,
        total: usize,
    },

    /// New lobby created
    #[serde(rename_all = "camelCase")]
    LobbyCreated {
        lobby_info: LobbyInfo,
    },

    /// Lobby status changed
    LobbyUpdated {
        lobby: LobbyInfo,
    },

    /// Lobby deleted/finished
    #[serde(rename_all = "camelCase")]
    LobbyRemoved {
        lobby_id: uuid::Uuid,
    },

    Error {
        code: String,
        message: String,
    },
}
