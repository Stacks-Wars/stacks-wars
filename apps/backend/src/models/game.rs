use std::str::FromStr;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::errors::AppError;

/// Game model mapping to the `games` table.
/// Represents a playable game mode with metadata and limits.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    #[serde(skip_deserializing)]
    pub(crate) id: Uuid,
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub min_players: i16,
    pub max_players: i16,
    pub category: Option<String>,
    pub creator_id: Uuid,
    pub is_active: bool,
    pub updated_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

impl Game {
    /// Get game ID.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Validate player count range.
    pub fn validate_player_count(
        min_players: i16,
        max_players: i16,
    ) -> Result<(i16, i16), PlayerCountError> {
        if min_players < 1 {
            return Err(PlayerCountError::MinTooLow { min: min_players });
        }
        if max_players < min_players {
            return Err(PlayerCountError::MaxLessThanMin {
                min: min_players,
                max: max_players,
            });
        }
        if max_players > 100 {
            return Err(PlayerCountError::MaxTooHigh { max: max_players });
        }
        Ok((min_players, max_players))
    }
}

/// Player count validation errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum PlayerCountError {
    #[error("Minimum players must be at least 1, got {min}")]
    MinTooLow { min: i16 },

    #[error("Maximum players ({max}) must be >= minimum players ({min})")]
    MaxLessThanMin { min: i16, max: i16 },

    #[error("Maximum players cannot exceed 100, got {max}")]
    MaxTooHigh { max: i16 },
}

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

#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub page: i64,
    pub limit: i64,
}

impl Pagination {
    pub fn offset(&self) -> i64 {
        (self.page.saturating_sub(1)) * self.limit
    }
}
