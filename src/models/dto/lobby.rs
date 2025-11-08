use serde::{Deserialize, Serialize};

use crate::models::{
    User,
    enums::JoinState,
    redis_key::{LobbyInfo, Player},
};

/// Lobby entry pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyPoolInput {
    pub entry_amount: f64,
    pub current_amount: f64,
    pub contract_address: String,
    #[serde(default = "default_token_symbol")]
    pub token_symbol: Option<String>,
    pub token_id: Option<String>,
}

fn default_token_symbol() -> Option<String> {
    Some("STX".to_string())
}

/// Join request for private lobbies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRequest {
    pub user: User,
    pub state: JoinState,
}

/// Lobby with player list (extended view)
#[derive(Serialize, Debug)]
pub struct LobbyExtended {
    pub lobby: LobbyInfo,
    pub players: Vec<Player>,
}

/// Player-specific lobby view including their performance
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerLobbyInfo {
    #[serde(flatten)]
    pub lobby: LobbyInfo,
    pub prize_amount: Option<f64>,
    pub rank: Option<usize>,
    pub claim_state: Option<crate::models::enums::ClaimState>,
}
