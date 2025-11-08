//! Leaderboard Handlers
//!
//! Provides competitive rankings and player statistics.
//! Rankings are based on wars points, win rates, and other performance metrics.
//!
//! **Note**: This module currently uses legacy Redis operations and will be
//! refactored to use LeaderboardRepository in a future update.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    db::{
        leaderboard::get::{get_leaderboard, get_user_stat},
        user::legacy::get_legacy::get_user_id,
    },
    errors::AppError,
    models::leaderboard::LeaderBoard,
    state::AppState,
};

// ============================================================================
// Request/Response Types
// ============================================================================

/// Query parameters for leaderboard listing
#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    /// Optional limit for number of results (default: all, max: 1000)
    pub limit: Option<u64>,
}

/// Query parameters for user statistics lookup
#[derive(Debug, Deserialize)]
pub struct UserStatQuery {
    /// User UUID (preferred)
    pub user_id: Option<Uuid>,
    /// User identifier (username or STX address)
    pub identifier: Option<String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Get global leaderboard
///
/// Returns the ranked list of top players based on wars points.
/// Includes win rates, total matches, and profit/loss statistics.
///
/// # Authentication
/// - **Required**: No
///
/// # Query Parameters
/// - `limit` (optional): Number of top players to return (default: all, max: 1000)
///
/// # Response
/// - **200 OK**: Leaderboard rankings
/// ```json
/// [
///   {
///     "user": {
///       "id": "123e4567-e89b-12d3-a456-426614174000",
///       "username": "player123",
///       "displayName": "Pro Player",
///       "warsPoint": 1500.0
///     },
///     "rank": 1,
///     "winRate": 75.5,
///     "totalMatch": 100,
///     "totalWins": 75,
///     "pnl": 250.50
///   }
/// ]
/// ```
///
/// # Errors
/// - **500 Internal Server Error**: Redis error
///
/// # Notes
/// - Leaderboard is calculated from Redis sorted sets for performance
/// - Rankings update in real-time as matches complete
/// - Win rate is calculated as (wins / total_matches) * 100
/// - PNL (Profit and Loss) represents total STX earned/lost
pub async fn get_leaderboard_rankings(
    Query(query): Query<LeaderboardQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<LeaderBoard>>, (StatusCode, String)> {
    // Cap limit to prevent excessive memory usage
    let limit = query.limit.map(|l| l.min(1000));

    let leaderboard = get_leaderboard(limit, state.redis).await.map_err(|e| {
        tracing::error!("Failed to get leaderboard: {}", e);
        e.to_response()
    })?;

    tracing::info!("Retrieved leaderboard with {} players", leaderboard.len());
    Ok(Json(leaderboard))
}

/// Get user statistics and rank
///
/// Returns detailed statistics for a specific user including their global rank,
/// win rate, total matches, and earnings.
///
/// # Authentication
/// - **Required**: No
///
/// # Query Parameters
/// - `user_id` (optional): User UUID
/// - `identifier` (optional): Username or STX address
/// - **Note**: Must provide either `user_id` or `identifier`
///
/// # Response
/// - **200 OK**: User statistics found
/// ```json
/// {
///   "user": {
///     "id": "123e4567-e89b-12d3-a456-426614174000",
///     "username": "player123",
///     "displayName": "Pro Player",
///     "warsPoint": 1500.0
///   },
///   "rank": 42,
///   "winRate": 68.5,
///   "totalMatch": 150,
///   "totalWins": 103,
///   "pnl": 125.75
/// }
/// ```
///
/// # Errors
/// - **400 Bad Request**: Neither user_id nor identifier provided, or empty identifier
/// - **404 Not Found**: User not found
/// - **500 Internal Server Error**: Redis error
///
/// # Examples
/// - By user_id: `GET /api/leaderboard/user?user_id=123e4567-e89b-12d3-a456-426614174000`
/// - By username: `GET /api/leaderboard/user?identifier=player123`
/// - By STX address: `GET /api/leaderboard/user?identifier=SP2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKNRV9EJ7`
pub async fn get_user_statistics(
    Query(query): Query<UserStatQuery>,
    State(state): State<AppState>,
) -> Result<Json<LeaderBoard>, (StatusCode, String)> {
    // Resolve user_id from either direct UUID or identifier
    let user_id = match (query.user_id, query.identifier) {
        (Some(id), _) => {
            // Direct user_id provided
            id
        }
        (None, Some(identifier)) => {
            // Validate identifier is not empty
            if identifier.trim().is_empty() {
                tracing::warn!("Empty identifier provided");
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Identifier cannot be empty".to_string(),
                ));
            }

            // Look up user_id from identifier
            get_user_id(identifier.clone(), state.redis.clone())
                .await
                .map_err(|e| {
                    tracing::error!(
                        "Failed to resolve user ID from identifier '{}': {}",
                        identifier,
                        e
                    );
                    match e {
                        AppError::NotFound(_) => (
                            StatusCode::NOT_FOUND,
                            "User not found for the provided identifier".to_string(),
                        ),
                        _ => e.to_response(),
                    }
                })?
        }
        (None, None) => {
            tracing::warn!("Neither user_id nor identifier provided");
            return Err((
                StatusCode::BAD_REQUEST,
                "Either user_id or identifier must be provided".to_string(),
            ));
        }
    };

    // Fetch user statistics
    let user_stat = get_user_stat(user_id, state.redis).await.map_err(|e| {
        tracing::error!("Failed to get statistics for user {}: {}", user_id, e);
        e.to_response()
    })?;

    tracing::info!(
        "Retrieved statistics for user {}: rank {}",
        user_id,
        user_stat.rank
    );
    Ok(Json(user_stat))
}
