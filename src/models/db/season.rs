use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents gameplay seasons for leaderboard resets and rewards
/// Maps to `seasons` table in PostgreSQL
///
/// Seasons define time-bound competitive periods where players compete
/// for rankings and rewards. Each season has its own leaderboard tracked
/// via `user_wars_points`.
///
/// # Database Schema
/// - Primary key: `id` (auto-increment)
/// - Constraints: end_date must be after start_date
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Season {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub created_at: NaiveDateTime,
}
