use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

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
pub struct GameV2 {
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
