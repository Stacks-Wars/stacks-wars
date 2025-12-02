use crate::models::{
    game::{LobbyState, Player, PlayerState},
    user::User,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum JoinState {
    Pending,
    Allowed,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRequest {
    pub user: User,
    pub state: JoinState,
}

//#[derive(Serialize)]
//pub struct PaginatedResponse<T> {
//    pub data: Vec<T>,
//    pub pagination: PaginationMeta,
//}

//#[derive(Serialize)]
//pub struct PaginationMeta {
//    pub page: u32,
//    pub limit: u32,
//    pub total_count: u32,
//    pub total_pages: u32,
//    pub has_next: bool,
//    pub has_previous: bool,
//}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LobbyClientMessage {
    #[serde(rename_all = "camelCase")]
    UpdatePlayerState {
        new_state: PlayerState,
    },

    #[serde(rename_all = "camelCase")]
    UpdateLobbyState {
        new_state: LobbyState,
    },

    LeaveLobby,

    #[serde(rename_all = "camelCase")]
    KickPlayer {
        player_id: Uuid,
    },

    RequestJoin,

    #[serde(rename_all = "camelCase")]
    PermitJoin {
        user_id: Uuid,
        allow: bool,
    },

    #[serde(rename_all = "camelCase")]
    JoinLobby {
        tx_id: Option<String>,
    },

    Ping {
        ts: u64,
    },

    LastPing {
        ts: u64,
    },

    RequestLeave,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PendingJoin {
    pub user: User,
    pub state: JoinState,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LobbyServerMessage {
    PlayerUpdated {
        players: Vec<Player>,
    },
    PlayerKicked {
        player: User,
    },
    NotifyKicked,
    Left,
    Countdown {
        time: u32,
    },

    #[serde(rename_all = "camelCase")]
    LobbyState {
        state: LobbyState,
        joined_players: Option<Vec<Uuid>>,
        started: bool,
    },

    #[serde(rename_all = "camelCase")]
    PendingPlayers {
        pending_players: Vec<PendingJoin>,
    },
    PlayersNotJoined {
        players: Vec<Player>,
    },
    Allowed,
    Rejected,
    Pending,
    Error {
        message: String,
    },
    Pong {
        ts: u64,
        pong: u64,
    },

    #[serde(rename_all = "camelCase")]
    WarsPointDeduction {
        amount: f64,
        new_total: f64,
        reason: String,
    },

    #[serde(rename_all = "camelCase")]
    IsConnectedPlayer {
        response: bool,
    },
}

impl LobbyServerMessage {
    /// Determines if this message should be queued for offline players
    pub fn should_queue(&self) -> bool {
        match self {
            // Time-sensitive messages that should NOT be queued
            LobbyServerMessage::Countdown { .. } => false,
            LobbyServerMessage::Pong { .. } => false,

            // Important messages that SHOULD be queued
            LobbyServerMessage::Error { .. } => true,
            LobbyServerMessage::Allowed { .. } => true,
            LobbyServerMessage::LobbyState { .. } => true,
            LobbyServerMessage::PlayersNotJoined { .. } => true,
            LobbyServerMessage::PlayerKicked { .. } => true,
            LobbyServerMessage::Rejected { .. } => true,
            LobbyServerMessage::PendingPlayers { .. } => true,
            LobbyServerMessage::NotifyKicked => true,
            LobbyServerMessage::Left => true,
            LobbyServerMessage::PlayerUpdated { .. } => true,
            LobbyServerMessage::Pending { .. } => true,
            LobbyServerMessage::WarsPointDeduction { .. } => true,
            LobbyServerMessage::IsConnectedPlayer { .. } => true,
        }
    }
}
