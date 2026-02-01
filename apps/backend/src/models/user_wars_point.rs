use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::models::WalletAddress;

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
    pub total_matches: i32,
    pub total_wins: i32,
    pub total_pnl: f64,
    pub win_rate: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LeaderBoard {
    pub id: Uuid,
    pub season_id: i32,
    pub points: f64,
    pub rank_badge: Option<String>,
    pub user_id: Uuid,
    pub wallet_address: WalletAddress,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub profile_image: Option<String>,
    pub email: String,
    pub email_verified: bool,
    pub trust_rating: f64,
    pub total_matches: i32,
    pub total_wins: i32,
    pub total_pnl: f64,
    pub win_rate: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
