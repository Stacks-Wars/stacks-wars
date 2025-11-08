use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Player participation state in a lobby/game
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

/// Lobby lifecycle state
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

/// SQL ordering direction
#[derive(Debug, Clone, Copy)]
pub enum Order {
    Ascending,
    Descending,
}

impl Order {
    pub fn to_sql(&self) -> &'static str {
        match self {
            Order::Ascending => "ASC",
            Order::Descending => "DESC",
        }
    }
}

impl FromStr for Order {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "asc" | "ascending" => Ok(Order::Ascending),
            "desc" | "descending" => Ok(Order::Descending),
            _ => Err(AppError::BadRequest(format!("Unknown order: {}", s))),
        }
    }
}

/// Join request state for private lobbies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum JoinState {
    Pending,
    Allowed,
    Rejected,
}
