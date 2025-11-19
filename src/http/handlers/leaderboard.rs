// Legacy Redis-backed leaderboard handlers (deprecated)

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

/// Get global leaderboard (cached in Redis). Returns top players and stats.
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

/// Get user statistics and rank by `user_id` or `identifier`.
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
