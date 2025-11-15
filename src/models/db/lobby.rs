use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::models::redis::LobbyStatus;

/// Core structure representing a multiplayer lobby
/// Maps to `lobbies` table in PostgreSQL
///
/// A lobby is a game room where players gather before starting a match.
/// Supports both free and paid entry with STX or custom tokens.
///
/// # Database Schema
/// - Primary key: `id`
/// - Foreign keys: `game_id` (games), `creator_id` (users)
/// - Status values: "waiting", "starting", "in_progress", "finished"
///
/// # State Flow
/// waiting → starting → in_progress → finished
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
