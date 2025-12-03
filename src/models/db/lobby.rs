use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::models::redis::{LobbyState, LobbyStatus};

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

/// Flattened Lobby payload combining Postgres metadata and Redis runtime fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LobbyExtended {
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

    // Postgres-backed status
    pub status: LobbyStatus,

    // Runtime fields from Redis
    pub participant_count: usize,
    pub creator_last_ping: Option<u64>,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl LobbyExtended {
    /// Build a flattened extended lobby payload from Postgres `Lobby` and Redis `LobbyState`.
    pub fn from_parts(db: Lobby, runtime: LobbyState) -> Self {
        Self {
            id: db.id,
            name: db.name,
            description: db.description,
            game_id: db.game_id,
            creator_id: db.creator_id,
            entry_amount: db.entry_amount,
            current_amount: db.current_amount,
            token_symbol: db.token_symbol,
            token_contract_id: db.token_contract_id,
            contract_address: db.contract_address,
            is_private: db.is_private,
            is_sponsored: db.is_sponsored,
            status: runtime.status,
            participant_count: runtime.participant_count,
            creator_last_ping: runtime.creator_last_ping,
            started_at: runtime.started_at,
            finished_at: runtime.finished_at,
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}
