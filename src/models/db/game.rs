use std::str::FromStr;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::errors::AppError;

/// Defines each game type under Stacks Wars
/// Maps to `games` table in PostgreSQL
///
/// Represents a game mode available on the platform (e.g., "Lexi Wars").
/// Each game has its own rules, player limits, and can be toggled active/inactive.
///
/// # Database Schema
/// - Primary key: `id`
/// - Foreign key: `creator_id` (users)
/// - Unique constraint: `name`
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: Uuid,
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
