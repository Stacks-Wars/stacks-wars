// Lobby runtime state stored in Redis (status, participants, timing)

use crate::errors::AppError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;

/// Lobby lifecycle status enum
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "lobby_status", rename_all = "lowercase")]
pub enum LobbyStatus {
    Waiting,
    Starting,
    InProgress,
    Finished,
}

impl FromStr for LobbyStatus {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Waiting" => Ok(LobbyStatus::Waiting),
            "Starting" => Ok(LobbyStatus::Starting),
            "InProgress" => Ok(LobbyStatus::InProgress),
            "Finished" => Ok(LobbyStatus::Finished),
            other => Err(AppError::BadRequest(format!(
                "Unknown LobbyState: {}",
                other
            ))),
        }
    }
}

/// Runtime state of a lobby stored in Redis (dynamic runtime fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LobbyState {
    /// Lobby ID (for validation)
    pub lobby_id: Uuid,

    /// Current lobby status
    pub status: LobbyStatus, // Waiting, Starting, InProgress, Finished

    /// Number of players (for quick checks without querying sets)
    pub participant_count: usize,

    /// Unix timestamp when lobby was created in Redis
    pub created_at: i64,

    /// Unix timestamp of last state update
    pub updated_at: i64,

    /// Unix timestamp when game started (None if not started)
    pub started_at: Option<i64>,

    /// Unix timestamp when game finished (None if not finished)
    pub finished_at: Option<i64>,

    /// Last ping from lobby creator (for timeout detection)
    pub creator_last_ping: Option<u64>,

    /// Telegram message ID (for bot notifications)
    pub tg_msg_id: Option<i32>,
}

impl LobbyState {
    /// Create new lobby state with default values
    pub fn new(lobby_id: Uuid) -> Self {
        let now = Utc::now().timestamp();
        Self {
            lobby_id,
            status: LobbyStatus::Waiting,
            participant_count: 0,
            created_at: now,
            updated_at: now,
            started_at: None,
            finished_at: None,
            creator_last_ping: None,
            tg_msg_id: None,
        }
    }

    /// Convert to Redis hash map for storage
    ///
    /// Only serializes fields that are set (Some values).
    pub fn to_redis_hash(&self) -> Vec<(String, String)> {
        let mut fields = vec![
            ("lobby_id".into(), self.lobby_id.to_string()),
            ("status".into(), format!("{:?}", self.status)),
            (
                "participant_count".into(),
                self.participant_count.to_string(),
            ),
            ("created_at".into(), self.created_at.to_string()),
            ("updated_at".into(), self.updated_at.to_string()),
        ];

        if let Some(started) = self.started_at {
            fields.push(("started_at".into(), started.to_string()));
        }
        if let Some(finished) = self.finished_at {
            fields.push(("finished_at".into(), finished.to_string()));
        }
        if let Some(ping) = self.creator_last_ping {
            fields.push(("creator_last_ping".into(), ping.to_string()));
        }
        if let Some(msg_id) = self.tg_msg_id {
            fields.push(("tg_msg_id".into(), msg_id.to_string()));
        }

        fields
    }

    /// Parse from Redis hash map
    ///
    /// # Errors
    /// - `AppError::InvalidInput` if required fields are missing or invalid
    pub fn from_redis_hash(map: &HashMap<String, String>) -> Result<Self, AppError> {
        let lobby_id = map
            .get("lobby_id")
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| AppError::InvalidInput("Missing or invalid lobby_id".into()))?;

        let status = map
            .get("status")
            .and_then(|s| s.parse::<LobbyStatus>().ok())
            .ok_or_else(|| AppError::InvalidInput("Missing or invalid status".into()))?;

        let participant_count = map
            .get("participant_count")
            .and_then(|c| c.parse::<usize>().ok())
            .unwrap_or(0);

        let created_at = map
            .get("created_at")
            .and_then(|t| t.parse::<i64>().ok())
            .ok_or_else(|| AppError::InvalidInput("Missing or invalid created_at".into()))?;

        let updated_at = map
            .get("updated_at")
            .and_then(|t| t.parse::<i64>().ok())
            .ok_or_else(|| AppError::InvalidInput("Missing or invalid updated_at".into()))?;

        let started_at = map.get("started_at").and_then(|t| t.parse::<i64>().ok());

        let finished_at = map.get("finished_at").and_then(|t| t.parse::<i64>().ok());

        let creator_last_ping = map
            .get("creator_last_ping")
            .and_then(|p| p.parse::<u64>().ok());

        let tg_msg_id = map.get("tg_msg_id").and_then(|id| id.parse::<i32>().ok());

        Ok(Self {
            lobby_id,
            status,
            participant_count,
            created_at,
            updated_at,
            started_at,
            finished_at,
            creator_last_ping,
            tg_msg_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lobby_state_new() {
        let lobby_id = Uuid::new_v4();
        let state = LobbyState::new(lobby_id);

        assert_eq!(state.lobby_id, lobby_id);
        assert_eq!(state.status, LobbyStatus::Waiting);
        assert_eq!(state.participant_count, 0);
        assert!(state.started_at.is_none());
        assert!(state.finished_at.is_none());
    }

    #[test]
    fn test_to_redis_hash() {
        let lobby_id = Uuid::new_v4();
        let state = LobbyState::new(lobby_id);

        let hash = state.to_redis_hash();

        // Check required fields
        assert!(hash.iter().any(|(k, _)| k == "lobby_id"));
        assert!(hash.iter().any(|(k, _)| k == "status"));
        assert!(hash.iter().any(|(k, _)| k == "participant_count"));
        assert!(hash.iter().any(|(k, _)| k == "created_at"));
        assert!(hash.iter().any(|(k, _)| k == "updated_at"));

        // Optional fields should not be present if None
        assert!(!hash.iter().any(|(k, _)| k == "started_at"));
        assert!(!hash.iter().any(|(k, _)| k == "finished_at"));
    }

    #[test]
    fn test_from_redis_hash() {
        let lobby_id = Uuid::new_v4();
        let mut map = HashMap::new();
        map.insert("lobby_id".to_string(), lobby_id.to_string());
        map.insert("status".to_string(), "Waiting".to_string());
        map.insert("participant_count".to_string(), "5".to_string());
        map.insert("created_at".to_string(), "1000".to_string());
        map.insert("updated_at".to_string(), "2000".to_string());

        let state = LobbyState::from_redis_hash(&map).unwrap();

        assert_eq!(state.lobby_id, lobby_id);
        assert_eq!(state.status, LobbyStatus::Waiting);
        assert_eq!(state.participant_count, 5);
        assert_eq!(state.created_at, 1000);
        assert_eq!(state.updated_at, 2000);
    }
}
