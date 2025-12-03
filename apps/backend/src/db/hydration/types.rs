use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;

/// Legacy types for migration/hydration from Redis to Postgres

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub wallet_address: String,
    pub wars_point: f64,

    pub username: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PlayerState {
    NotJoined,
    Joined,
}

impl FromStr for PlayerState {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "notjoined" | "notJoined" => Ok(PlayerState::NotJoined),
            "joined" => Ok(PlayerState::Joined),
            other => Err(AppError::BadRequest(format!(
                "Unknown PlayerState: {}",
                other
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", content = "data", rename_all = "camelCase")]
pub enum ClaimState {
    Claimed { tx_id: String },
    NotClaimed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub id: Uuid,
    pub state: PlayerState,
    pub rank: Option<usize>,
    pub tx_id: Option<String>,
    pub claim: Option<ClaimState>,
    pub prize: Option<f64>,
    pub last_ping: Option<u64>,

    // Game-specific fields only
    pub used_words: Option<Vec<String>>,

    // Hydrated user data (not stored in Redis)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
}

impl Player {
    pub fn from_redis_hash(data: &HashMap<String, String>) -> Result<Self, AppError> {
        let id = data
            .get("id")
            .ok_or_else(|| AppError::Deserialization("Missing player id".into()))?
            .parse::<Uuid>()
            .map_err(|e| AppError::Deserialization(format!("Invalid player id: {}", e)))?;

        let state = data
            .get("state")
            .ok_or_else(|| AppError::Deserialization("Missing player state".into()))?
            .parse::<PlayerState>()
            .map_err(|e| AppError::Deserialization(format!("Invalid player state: {}", e)))?;

        let rank = data.get("rank").and_then(|v| v.parse::<usize>().ok());

        let used_words = data
            .get("used_words")
            .and_then(|v| serde_json::from_str::<Vec<String>>(v).ok());

        let tx_id = data.get("tx_id").cloned();

        let claim = data
            .get("claim")
            .and_then(|v| serde_json::from_str::<ClaimState>(v).ok());

        let prize = data.get("prize").and_then(|v| v.parse::<f64>().ok());

        let last_ping = data.get("last_ping").and_then(|v| v.parse::<u64>().ok());

        Ok(Player {
            id,
            state,
            rank,
            used_words,
            tx_id,
            claim,
            prize,
            last_ping,
            user: None, // Will be hydrated separately
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameType {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub min_players: u8,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum LobbyState {
    Waiting,
    Starting,
    InProgress,
    Finished,
}

impl FromStr for LobbyState {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Waiting" => Ok(LobbyState::Waiting),
            "Starting" => Ok(LobbyState::Starting),
            "InProgress" => Ok(LobbyState::InProgress),
            "Finished" => Ok(LobbyState::Finished),
            other => Err(AppError::BadRequest(format!(
                "Unknown LobbyState: {}",
                other
            ))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LobbyInfo {
    pub id: Uuid,
    pub name: String,

    pub creator: User,
    pub state: LobbyState,
    pub game: GameType,
    pub participants: usize,
    pub created_at: DateTime<Utc>,

    pub description: Option<String>,
    pub contract_address: Option<String>,
    pub entry_amount: Option<f64>,
    pub current_amount: Option<f64>,
    pub token_symbol: Option<String>,
    pub token_id: Option<String>,
    pub creator_last_ping: Option<u64>,
    pub tg_msg_id: Option<i32>,
}

impl LobbyInfo {
    pub fn from_redis_hash_partial(
        map: &HashMap<String, String>,
    ) -> Result<(Self, Uuid, Uuid), AppError> {
        let creator_id = map
            .get("creator_id")
            .ok_or_else(|| AppError::Deserialization("Missing creator_id".into()))?
            .parse()
            .map_err(|_| AppError::Deserialization("Invalid UUID for creator_id".into()))?;

        let game_id = map
            .get("game_id")
            .ok_or_else(|| AppError::Deserialization("Missing game_id".into()))?
            .parse()
            .map_err(|_| AppError::Deserialization("Invalid UUID for game_id".into()))?;

        // Create placeholder structs - will be replaced during hydration
        let placeholder_creator = User {
            id: creator_id,
            wallet_address: String::new(),
            wars_point: 0.0,
            username: None,
            display_name: None,
        };

        let placeholder_game = GameType {
            id: game_id,
            name: String::new(),
            description: String::new(),
            image_url: String::new(),
            min_players: 0,
            tags: None,
        };

        let lobby = Self {
            id: map
                .get("id")
                .ok_or_else(|| AppError::Deserialization("Missing id".into()))?
                .parse()
                .map_err(|_| AppError::Deserialization("Invalid UUID for id".into()))?,
            name: map
                .get("name")
                .ok_or_else(|| AppError::Deserialization("Missing name".into()))?
                .clone(),
            creator: placeholder_creator,
            state: map
                .get("state")
                .ok_or_else(|| AppError::Deserialization("Missing state".into()))?
                .parse::<LobbyState>()
                .map_err(|_| AppError::Deserialization("Invalid state".into()))?,
            game: placeholder_game,
            participants: map
                .get("participants")
                .ok_or_else(|| AppError::Deserialization("Missing participants".into()))?
                .parse()
                .map_err(|_| AppError::Deserialization("Invalid participants count".into()))?,
            created_at: map
                .get("created_at")
                .ok_or_else(|| AppError::Deserialization("Missing created_at".into()))?
                .parse()
                .map_err(|_| AppError::Deserialization("Invalid datetime format".into()))?,
            description: map.get("description").cloned(),
            contract_address: map.get("contract_address").cloned(),
            entry_amount: map.get("entry_amount").and_then(|s| s.parse().ok()),
            current_amount: map.get("current_amount").and_then(|s| s.parse().ok()),
            token_symbol: map.get("token_symbol").cloned(),
            token_id: map.get("token_id").cloned(),
            creator_last_ping: map.get("creator_last_ping").and_then(|s| s.parse().ok()),
            tg_msg_id: map.get("tg_msg_id").and_then(|s| s.parse().ok()),
        };

        Ok((lobby, creator_id, game_id))
    }
}
