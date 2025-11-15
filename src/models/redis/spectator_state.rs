//! SpectatorState - runtime spectator state in Redis
//!
//! Similar to `PlayerState` but only contains spectator-specific runtime fields
//! (no prizes, ranks or tx info). Stored as a Redis hash at
//! `lobbies:{lobby_id}:spectators:{user_id}`.
use crate::errors::AppError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Runtime state of a spectator in a lobby stored in Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpectatorState {
    pub user_id: Uuid,
    pub lobby_id: Uuid,
    pub last_ping: Option<u64>,
    pub joined_at: i64,
    pub updated_at: i64,
}

impl SpectatorState {
    pub fn new(user_id: Uuid, lobby_id: Uuid) -> Self {
        let now = Utc::now().timestamp();
        Self {
            user_id,
            lobby_id,
            last_ping: Some(Utc::now().timestamp_millis() as u64),
            joined_at: now,
            updated_at: now,
        }
    }

    pub fn to_redis_hash(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("user_id".to_string(), self.user_id.to_string());
        map.insert("lobby_id".to_string(), self.lobby_id.to_string());
        map.insert("joined_at".to_string(), self.joined_at.to_string());
        map.insert("updated_at".to_string(), self.updated_at.to_string());
        if let Some(p) = self.last_ping {
            map.insert("last_ping".to_string(), p.to_string());
        }
        map
    }

    pub fn from_redis_hash(data: &HashMap<String, String>) -> Result<Self, AppError> {
        let user_id = data
            .get("user_id")
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| AppError::InvalidInput("Missing or invalid user_id".into()))?;

        let lobby_id = data
            .get("lobby_id")
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| AppError::InvalidInput("Missing or invalid lobby_id".into()))?;

        let last_ping = data.get("last_ping").and_then(|p| p.parse::<u64>().ok());

        let joined_at = data
            .get("joined_at")
            .and_then(|t| t.parse::<i64>().ok())
            .ok_or_else(|| AppError::InvalidInput("Missing or invalid joined_at".into()))?;

        let updated_at = data
            .get("updated_at")
            .and_then(|t| t.parse::<i64>().ok())
            .ok_or_else(|| AppError::InvalidInput("Missing or invalid updated_at".into()))?;

        Ok(Self {
            user_id,
            lobby_id,
            last_ping,
            joined_at,
            updated_at,
        })
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now().timestamp();
    }

    pub fn update_ping(&mut self) {
        self.last_ping = Some(Utc::now().timestamp_millis() as u64);
        self.touch();
    }
}
