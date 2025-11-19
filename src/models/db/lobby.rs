use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::models::redis::LobbyStatus;

/// Lobby model mapping to the `lobbies` table (room metadata and status).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Lobby {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub game_id: Uuid,
    pub creator_id: Uuid,
    pub entry_amount: Option<f64>,
    pub current_amount: Option<f64>,
    pub token_symbol: Option<String>,
    pub token_contract_id: Option<String>,
    pub contract_address: Option<String>,
    pub is_private: bool,
    pub is_sponsored: bool,
    pub status: LobbyStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
