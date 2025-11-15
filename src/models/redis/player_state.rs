//! # PlayerState - Generic Player State in Redis
//!
//! This model represents **platform-generic player state** in a lobby.
//! It contains NO game-specific fields, making it usable for ALL games.
//!
//! ## Storage
//! - **Redis Key**: `lobbies:{lobby_id}:players:{user_id}`
//! - **Type**: Hash
//!
//! ## Separation of Concerns
//!
//! ### PlayerState (Redis) - This file
//! - User/lobby identity
//! - Join status
//! - Payment tracking (tx_id)
//! - Post-game results (rank, prize, claim_state)
//! - Connection tracking (last_ping, joined_at)
//!
//! ### GameState (Redis - games/{game}/state.rs)
//! - Game-specific player data:
//!   - Lexi Wars: used_words, current turn
//!   - Chess: position, captured pieces
//!   - Poker: hand, chips, bet amount
//!
//! ## Why Separate?
//!
//! **Problem with old Player model:**
//! ```rust
//! pub struct Player {
//!     pub id: Uuid,
//!     pub used_words: Option<Vec<String>>, // ❌ Only for Lexi Wars!
//!     // Other games can't use this without carrying irrelevant fields
//! }
//! ```
//!
//! **Solution with new architecture:**
//! ```rust
//! // Platform-generic (ALL games)
//! pub struct PlayerState { ... } // This file
//!
//! // Game-specific (extensible)
//! pub trait GameState { ... }
//! pub struct LexiWarsGameState { pub used_words: Vec<String>, ... }
//! pub struct ChessGameState { pub position: Position, ... }
//! ```

use crate::errors::AppError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;

/// Player participation state in a lobby/game
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PlayerStatus {
    NotJoined,
    Joined,
}

impl FromStr for PlayerStatus {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "notjoined" | "notJoined" => Ok(PlayerStatus::NotJoined),
            "joined" => Ok(PlayerStatus::Joined),
            other => Err(AppError::BadRequest(format!(
                "Unknown PlayerStatus: {}",
                other
            ))),
        }
    }
}

/// Prize claim status for finished games
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", content = "data", rename_all = "camelCase")]
pub enum ClaimState {
    Claimed { tx_id: String },
    NotClaimed,
}

impl ClaimState {
    pub fn matches_filter(&self, filter: &ClaimState) -> bool {
        match (self, filter) {
            (ClaimState::NotClaimed, ClaimState::NotClaimed) => true,
            (ClaimState::Claimed { .. }, ClaimState::Claimed { .. }) => true,
            _ => false,
        }
    }

    pub fn is_claimed(&self) -> bool {
        matches!(self, ClaimState::Claimed { .. })
    }

    pub fn is_not_claimed(&self) -> bool {
        matches!(self, ClaimState::NotClaimed)
    }
}

/// Runtime state of a player in a lobby stored in Redis
///
/// This is GENERIC and works for ALL games.
/// Game-specific data (e.g., used_words, hand, position) goes in GameState.
///
/// # Redis Key
/// `lobbies:{lobby_id}:players:{user_id}`
///
/// # Example
/// ```rust,ignore
/// let player_state = PlayerState::new(user_id, lobby_id, Some(tx_id));
///
/// // Save to Redis
/// let hash = player_state.to_redis_hash();
/// redis.hset_multiple(&key, &hash).await?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerState {
    /// Player's user ID
    pub user_id: Uuid,

    /// Lobby ID (for validation)
    pub lobby_id: Uuid,

    /// Current player status (NotJoined, Joined)
    pub status: PlayerStatus,

    /// Transaction ID for entry payment
    pub tx_id: Option<String>,

    /// Rank in finished game (1st, 2nd, 3rd, etc.)
    pub rank: Option<usize>,

    /// Prize amount won
    pub prize: Option<f64>,

    /// Prize claim status
    pub claim_state: Option<ClaimState>,

    /// Last heartbeat timestamp (for disconnect detection)
    pub last_ping: Option<u64>,

    /// Unix timestamp when player joined
    pub joined_at: i64,

    /// Unix timestamp of last update
    pub updated_at: i64,
}

impl PlayerState {
    /// Create new player state when joining lobby
    ///
    /// Sets status to `Joined` and initializes timestamps.
    pub fn new(user_id: Uuid, lobby_id: Uuid, tx_id: Option<String>) -> Self {
        let now = Utc::now().timestamp();
        Self {
            user_id,
            lobby_id,
            status: PlayerStatus::Joined,
            tx_id,
            rank: None,
            prize: None,
            claim_state: None,
            last_ping: Some(Utc::now().timestamp_millis() as u64),
            joined_at: now,
            updated_at: now,
        }
    }

    /// Convert to Redis hash map for storage
    pub fn to_redis_hash(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();

        map.insert("user_id".to_string(), self.user_id.to_string());
        map.insert("lobby_id".to_string(), self.lobby_id.to_string());
        map.insert("status".to_string(), format!("{:?}", self.status));
        map.insert("joined_at".to_string(), self.joined_at.to_string());
        map.insert("updated_at".to_string(), self.updated_at.to_string());

        if let Some(ref tx_id) = self.tx_id {
            map.insert("tx_id".to_string(), tx_id.clone());
        }
        if let Some(rank) = self.rank {
            map.insert("rank".to_string(), rank.to_string());
        }
        if let Some(prize) = self.prize {
            map.insert("prize".to_string(), prize.to_string());
        }
        if let Some(ref claim_state) = self.claim_state {
            map.insert(
                "claim_state".to_string(),
                serde_json::to_string(claim_state).unwrap_or_default(),
            );
        }
        if let Some(last_ping) = self.last_ping {
            map.insert("last_ping".to_string(), last_ping.to_string());
        }

        map
    }

    /// Parse from Redis hash map
    ///
    /// # Errors
    /// - `AppError::InvalidInput` if required fields are missing or invalid
    pub fn from_redis_hash(data: &HashMap<String, String>) -> Result<Self, AppError> {
        let user_id = data
            .get("user_id")
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| AppError::InvalidInput("Missing or invalid user_id".into()))?;

        let lobby_id = data
            .get("lobby_id")
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| AppError::InvalidInput("Missing or invalid lobby_id".into()))?;

        let status = data
            .get("status")
            .and_then(|s| s.parse::<PlayerStatus>().ok())
            .ok_or_else(|| AppError::InvalidInput("Missing or invalid status".into()))?;

        let tx_id = data.get("tx_id").cloned();

        let rank = data.get("rank").and_then(|r| r.parse::<usize>().ok());

        let prize = data.get("prize").and_then(|p| p.parse::<f64>().ok());

        let claim_state = data
            .get("claim_state")
            .and_then(|s| serde_json::from_str(s).ok());

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
            status,
            tx_id,
            rank,
            prize,
            claim_state,
            last_ping,
            joined_at,
            updated_at,
        })
    }

    /// Update the updated_at timestamp to now
    pub fn touch(&mut self) {
        self.updated_at = Utc::now().timestamp();
    }

    /// Update last ping timestamp
    pub fn update_ping(&mut self) {
        self.last_ping = Some(Utc::now().timestamp_millis() as u64);
        self.touch();
    }

    /// Set player rank and prize after game finishes
    pub fn set_result(&mut self, rank: usize, prize: f64) {
        self.rank = Some(rank);
        self.prize = Some(prize);
        self.claim_state = Some(ClaimState::NotClaimed);
        self.touch();
    }

    /// Mark prize as claimed with transaction ID
    pub fn mark_claimed(&mut self, tx_id: String) {
        self.claim_state = Some(ClaimState::Claimed { tx_id });
        self.touch();
    }

    /// Check if player has claimed their prize
    pub fn has_claimed(&self) -> bool {
        matches!(self.claim_state, Some(ClaimState::Claimed { .. }))
    }

    /// Check if player has a prize to claim
    pub fn has_prize(&self) -> bool {
        self.prize.is_some() && self.prize.unwrap() > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_state_new() {
        let user_id = Uuid::new_v4();
        let lobby_id = Uuid::new_v4();
        let tx_id = Some("tx123".to_string());

        let state = PlayerState::new(user_id, lobby_id, tx_id.clone());

        assert_eq!(state.user_id, user_id);
        assert_eq!(state.lobby_id, lobby_id);
        assert_eq!(state.status, PlayerStatus::Joined);
        assert_eq!(state.tx_id, tx_id);
        assert!(state.rank.is_none());
        assert!(state.prize.is_none());
        assert!(state.last_ping.is_some());
    }

    #[test]
    fn test_to_redis_hash() {
        let user_id = Uuid::new_v4();
        let lobby_id = Uuid::new_v4();
        let state = PlayerState::new(user_id, lobby_id, None);

        let hash = state.to_redis_hash();

        assert!(hash.contains_key("user_id"));
        assert!(hash.contains_key("lobby_id"));
        assert!(hash.contains_key("status"));
        assert!(hash.contains_key("joined_at"));
        assert!(hash.contains_key("updated_at"));
    }

    #[test]
    fn test_from_redis_hash() {
        let user_id = Uuid::new_v4();
        let lobby_id = Uuid::new_v4();

        let mut map = HashMap::new();
        map.insert("user_id".to_string(), user_id.to_string());
        map.insert("lobby_id".to_string(), lobby_id.to_string());
        map.insert("status".to_string(), "Joined".to_string());
        map.insert("joined_at".to_string(), "1000".to_string());
        map.insert("updated_at".to_string(), "2000".to_string());

        let state = PlayerState::from_redis_hash(&map).unwrap();

        assert_eq!(state.user_id, user_id);
        assert_eq!(state.lobby_id, lobby_id);
        assert_eq!(state.status, PlayerStatus::Joined);
        assert_eq!(state.joined_at, 1000);
        assert_eq!(state.updated_at, 2000);
    }

    #[test]
    fn test_set_result() {
        let user_id = Uuid::new_v4();
        let lobby_id = Uuid::new_v4();
        let mut state = PlayerState::new(user_id, lobby_id, None);

        state.set_result(1, 100.0);

        assert_eq!(state.rank, Some(1));
        assert_eq!(state.prize, Some(100.0));
        assert!(matches!(state.claim_state, Some(ClaimState::NotClaimed)));
    }

    #[test]
    fn test_mark_claimed() {
        let user_id = Uuid::new_v4();
        let lobby_id = Uuid::new_v4();
        let mut state = PlayerState::new(user_id, lobby_id, None);

        state.set_result(1, 100.0);
        state.mark_claimed("claim_tx_123".to_string());

        assert!(state.has_claimed());
        assert!(matches!(
            state.claim_state,
            Some(ClaimState::Claimed { .. })
        ));
    }

    #[test]
    fn test_has_prize() {
        let user_id = Uuid::new_v4();
        let lobby_id = Uuid::new_v4();
        let mut state = PlayerState::new(user_id, lobby_id, None);

        assert!(!state.has_prize());

        state.set_result(1, 100.0);
        assert!(state.has_prize());

        state.prize = Some(0.0);
        assert!(!state.has_prize());
    }
}
