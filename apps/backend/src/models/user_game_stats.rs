use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::models::WalletAddress;

/// Per-game player statistics for a season.
/// Maps to the `user_game_stats` table.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserGameStats {
    pub id: Uuid,
    pub user_id: Uuid,
    pub game_id: Uuid,
    pub season_id: i32,
    pub points: f64,
    pub total_matches: i32,
    pub total_wins: i32,
    pub total_pnl: f64,
    pub win_rate: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Game leaderboard entry — joins user_game_stats with users and games tables.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct GameLeaderBoard {
    pub id: Uuid,
    pub game_id: Uuid,
    pub game_name: String,
    pub game_image_url: String,
    pub season_id: i32,
    pub points: f64,
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

/// Summary of a user's top/most-played games for profile display.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserTopGame {
    pub game_id: Uuid,
    pub game_name: String,
    pub game_image_url: String,
    pub total_matches: i32,
    pub total_wins: i32,
    pub win_rate: f64,
    pub points: f64,
    pub total_pnl: f64,
}

/// Per-game stats for the platform stats page.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStats {
    pub game_id: Uuid,
    pub game_name: String,
    pub game_image_url: String,
    pub total_matches: i64,
    pub total_players: i64,
    pub total_wins: i64,
    pub avg_win_rate: f64,
    pub total_pnl: f64,
    pub total_points: f64,
}
