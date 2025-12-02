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
        user::get::get_user_id,
    },
    models::leaderboard::LeaderBoard,
    state::AppState,
};

#[derive(Deserialize)]
pub struct LeaderboardQuery {
    pub limit: Option<u64>,
}

pub async fn get_leaderboard_handler(
    Query(query): Query<LeaderboardQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<LeaderBoard>>, (StatusCode, String)> {
    let leaderboard = get_leaderboard(query.limit, state.redis)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get leaderboard: {}", e);
            e.to_response()
        })?;

    Ok(Json(leaderboard))
}

#[derive(Deserialize)]
pub struct GetUserStatPayload {
    pub user_id: Option<Uuid>,
    pub identifier: Option<String>,
}

pub async fn get_user_stat_handler(
    Query(payload): Query<GetUserStatPayload>,
    State(state): State<AppState>,
) -> Result<Json<LeaderBoard>, (StatusCode, String)> {
    let user_id = match (payload.user_id, payload.identifier) {
        (Some(id), _) => id,
        (None, Some(identifier)) => {
            if identifier.trim().is_empty() {
                tracing::warn!("Empty identifier provided");
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Identifier cannot be empty".to_string(),
                ));
            }

            get_user_id(identifier, state.redis.clone())
                .await
                .map_err(|e| {
                    tracing::error!("Failed to get user ID from identifier: {}", e);
                    match e {
                        crate::errors::AppError::NotFound(_) => (
                            StatusCode::NOT_FOUND,
                            format!("User not found for the provided identifier"),
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

    let user_stat = get_user_stat(user_id, state.redis).await.map_err(|e| {
        tracing::error!("Failed to get user stat for {}: {}", user_id, e);
        e.to_response()
    })?;

    Ok(Json(user_stat))
}
