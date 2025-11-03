use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::str::FromStr;
use uuid::Uuid;

use crate::{
    auth::AuthClaims,
    db::game::{
        get::{get_game_v2, get_games},
        post::create_game_v2,
    },
    errors::AppError,
    models::game::{GameV2, Order, Pagination},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct AddGamePayload {
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub min_players: u8,
    pub max_players: u8,
    pub category: Option<String>,
}
pub async fn create_game_handler(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(payload): Json<AddGamePayload>,
) -> Result<Json<GameV2>, (StatusCode, String)> {
    let creator_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Unauthorized access attempt");
        AppError::Unauthorized("Invalid user ID in token".into()).to_response()
    })?;

    let game = create_game_v2(
        payload.name,
        payload.description,
        payload.image_url,
        payload.min_players as i16,
        payload.max_players as i16,
        payload.category,
        creator_id,
        state.postgres.clone(),
    )
    .await
    .map_err(|e| {
        tracing::error!("Error adding new game: {}", e);
        e.to_response()
    })?;

    tracing::info!("Success adding {}: ({})", game.name, game.id);
    Ok(Json(game))
}

pub async fn get_game_handler(
    Path(game_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<GameV2>, (StatusCode, String)> {
    let game = get_game_v2(game_id, state.postgres.clone())
        .await
        .map_err(|e| {
            tracing::error!("Error retrieving {} game: {}", game_id, e);
            e.to_response()
        })?;

    tracing::info!("Success retrieving {game_id} game");
    Ok(Json(game))
}

#[derive(Deserialize)]
pub struct GetGamesPayload {
    pub page: u32,
    pub limit: u32,
    pub order: Option<String>,
}

pub async fn get_games_handler(
    State(state): State<AppState>,
    Json(payload): Json<GetGamesPayload>,
) -> Result<Json<Vec<GameV2>>, (StatusCode, String)> {
    let pagination = Pagination {
        page: payload.page as i64,
        limit: payload.limit as i64,
    };
    let order = payload
        .order
        .as_deref()
        .and_then(|s| Order::from_str(s).ok())
        .unwrap_or(Order::Descending);

    let games = get_games(pagination, order, state.postgres.clone())
        .await
        .map_err(|e| {
            tracing::error!("Error retrieving all games: {}", e);
            e.to_response()
        })?;

    tracing::info!("Success retrieving all game");
    Ok(Json(games))
}
