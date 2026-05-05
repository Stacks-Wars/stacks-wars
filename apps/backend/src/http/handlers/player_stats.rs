// User Wars Points handlers: leaderboard and rankings

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;

use crate::{
    db::user_game_stats::UserGameStatsRepository,
    db::user_wars_points::UserWarsPointsRepository,
    models::user_game_stats::{GameLeaderBoard, GameStats, UserTopGame},
    models::user_wars_point::LeaderBoard,
    state::AppState,
};

// ============================================================================
// Request/Response Types
// ============================================================================

/// Query parameters for leaderboard requests
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardQuery {
    /// Optional season ID to filter by specific season
    /// If not provided, uses the current active season
    pub season_id: Option<i32>,
    /// Maximum number of results to return (default: 50, max: 100)
    pub limit: Option<i64>,
    /// Number of results to skip for pagination (default: 0)
    pub offset: Option<i64>,
    /// Optional sort key: `points`, `matches`, `win_rate`, `pnl` (defaults to `points`)
    pub sort_by: Option<String>,
    /// Optional sort order: `asc` or `desc` (defaults to `desc`)
    pub order: Option<String>,
}

/// Sorting options for leaderboard queries.
#[derive(Debug, Clone, Copy)]
pub enum LeaderboardSortBy {
    WarsPoints,
    TotalMatches,
    WinRate,
    TotalPnl,
}

/// Sort order for leaderboard queries.
#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    Asc,
    Desc,
}

// ============================================================================
// Handlers
// ============================================================================

/// Response type for leaderboard with total count (for pagination)
#[derive(Debug, serde::Serialize)]
pub struct LeaderboardResponse {
    pub total: i64,
    pub leaderboard: Vec<LeaderBoard>,
}

/// Response type for game-specific leaderboard
#[derive(Debug, serde::Serialize)]
pub struct GameLeaderboardResponse {
    pub total: i64,
    pub leaderboard: Vec<GameLeaderBoard>,
}

/// Get the leaderboard rankings for a season.
///
/// Public endpoint returning paginated leaderboard data and total count for pagination.
/// Supports optional season filtering and pagination via query parameters.
///
/// Query parameters:
/// - `seasonId`: Optional season ID (defaults to current season)
/// - `limit`: Maximum results to return (default: 10, max: 100)
/// - `offset`: Number of results to skip (default: 0)
///
/// Returns a struct with leaderboard entries and total count.
pub async fn get_leaderboard_handler(
    State(state): State<AppState>,
    Query(query): Query<LeaderboardQuery>,
) -> Result<Json<LeaderboardResponse>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(10).min(100).max(1);
    let offset = query.offset.unwrap_or(0).max(0);

    let repo = UserWarsPointsRepository::new(state.postgres.clone());

    // Map query params to repository enums
    let sort_by_enum = match query.sort_by.as_deref().map(|s| s.to_lowercase()) {
        Some(s) if s == "matches" => Some(LeaderboardSortBy::TotalMatches),
        Some(s) if s == "win_rate" || s == "winRate" => Some(LeaderboardSortBy::WinRate),
        Some(s) if s == "pnl" => Some(LeaderboardSortBy::TotalPnl),
        _ => Some(LeaderboardSortBy::WarsPoints),
    };

    let order_enum = match query.order.as_deref().map(|s| s.to_lowercase()) {
        Some(s) if s == "asc" => Some(SortOrder::Asc),
        Some(_) => Some(SortOrder::Desc),
        None => Some(SortOrder::Desc),
    };

    let (leaderboard, total) = repo
        .get_leaderboard(query.season_id, limit, offset, sort_by_enum, order_enum)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(LeaderboardResponse { leaderboard, total }))
}

/// Get a single player's leaderboard entry for a season.
///
/// Public endpoint returning a player's complete stats including wars points and performance metrics.
/// Supports optional season filtering.
///
/// Path parameters:
/// - `user_id`: The UUID of the user to get stats for
///
/// Query parameters:
/// - `seasonId`: Optional season ID (defaults to current season)
///
/// Returns a single `LeaderBoard` entry with combined wars points and player statistics.
pub async fn get_player_leaderboard(
    State(state): State<AppState>,
    Path(user_id): Path<uuid::Uuid>,
    Query(query): Query<LeaderboardQuery>,
) -> Result<Json<LeaderBoard>, (StatusCode, String)> {
    let repo = UserWarsPointsRepository::new(state.postgres.clone());
    let entry = repo
        .get_player_leaderboard(user_id, query.season_id)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(entry))
}

// ============================================================================
// Game-Specific Leaderboard Handlers
// ============================================================================

/// Get the leaderboard for a specific game.
///
/// Public endpoint returning paginated per-game leaderboard data.
///
/// Path parameters:
/// - `game_id`: UUID of the game
///
/// Query parameters: same as global leaderboard (seasonId, limit, offset, sortBy, order)
pub async fn get_game_leaderboard_handler(
    State(state): State<AppState>,
    Path(game_id): Path<uuid::Uuid>,
    Query(query): Query<LeaderboardQuery>,
) -> Result<Json<GameLeaderboardResponse>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(10).min(100).max(1);
    let offset = query.offset.unwrap_or(0).max(0);

    let repo = UserGameStatsRepository::new(state.postgres.clone());

    let sort_by_enum = match query.sort_by.as_deref().map(|s| s.to_lowercase()) {
        Some(s) if s == "matches" => Some(LeaderboardSortBy::TotalMatches),
        Some(s) if s == "win_rate" || s == "winRate" => Some(LeaderboardSortBy::WinRate),
        Some(s) if s == "pnl" => Some(LeaderboardSortBy::TotalPnl),
        _ => Some(LeaderboardSortBy::WarsPoints),
    };

    let order_enum = match query.order.as_deref().map(|s| s.to_lowercase()) {
        Some(s) if s == "asc" => Some(SortOrder::Asc),
        Some(_) => Some(SortOrder::Desc),
        None => Some(SortOrder::Desc),
    };

    let (leaderboard, total) = repo
        .get_game_leaderboard(game_id, query.season_id, limit, offset, sort_by_enum, order_enum)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(GameLeaderboardResponse { leaderboard, total }))
}

/// Get a user's top/most-played games.
///
/// Public endpoint returning the user's favourite games ranked by matches played.
///
/// Path parameters:
/// - `user_id`: UUID of the user
///
/// Query parameters:
/// - `seasonId`: Optional season ID (defaults to current season)
/// - `limit`: Maximum results (default: 5)
pub async fn get_user_top_games_handler(
    State(state): State<AppState>,
    Path(user_id): Path<uuid::Uuid>,
    Query(query): Query<LeaderboardQuery>,
) -> Result<Json<Vec<UserTopGame>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(5).min(20).max(1);
    let repo = UserGameStatsRepository::new(state.postgres.clone());

    let top_games = repo
        .get_user_top_games(user_id, query.season_id, limit)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(top_games))
}

/// Get per-game platform stats for the stats page.
///
/// Public endpoint returning aggregated stats per game.
///
/// Query parameters:
/// - `seasonId`: Optional season ID (defaults to current season)
pub async fn get_platform_game_stats_handler(
    State(state): State<AppState>,
    Query(query): Query<LeaderboardQuery>,
) -> Result<Json<Vec<GameStats>>, (StatusCode, String)> {
    let repo = UserGameStatsRepository::new(state.postgres.clone());

    let game_stats = repo
        .get_platform_game_stats(query.season_id)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(game_stats))
}