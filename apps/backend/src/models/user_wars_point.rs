use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

/// Tracks player progression, ranks, and rewards across seasons
/// Maps to `user_wars_points` table in PostgreSQL
///
/// # Database Schema
/// - Primary key: `id`
/// - Foreign keys: `user_id` (users), `season_id` (seasons)
/// - Unique constraint: `(user_id, season_id)` - one entry per user per season
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserWarsPoints {
    pub id: Uuid,
    pub user_id: Uuid,
    pub season_id: i32,
    pub points: f64,
    pub rank_badge: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
