use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::models::game::Player;

/// Legacy user model for Redis-based operations
/// TODO: Migrate all usage to UserV2 for PostgreSQL persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub wallet_address: String,
    pub wars_point: f64,

    pub username: Option<String>,
    pub display_name: Option<String>,
}

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

/// Refined user model that maps directly to PostgreSQL
/// Maps to `users` table with embedded seasonal points
/// 
/// This is the primary user model for all new features and endpoints.
/// Contains wallet authentication, profile data, and trust metrics.
/// 
/// # Database Schema
/// - Primary key: `id`
/// - Unique constraint: `wallet_address`
/// - Default trust_rating: 10.0
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserV2 {
    pub id: Uuid,
    pub wallet_address: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub trust_rating: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub wars_point: UserWarsPoints,
}

impl From<Player> for User {
    fn from(player: Player) -> Self {
        player.into_user()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,    // user ID
    pub wallet: String, // wallet address
    pub exp: usize,     // expiration time
}
