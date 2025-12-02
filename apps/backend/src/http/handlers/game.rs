use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    db::game::{
        get::{get_all_games, get_game},
        post::create_game,
    },
    models::game::GameType,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct AddGamePayload {
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub tags: Option<Vec<String>>,
    pub min_players: u8,
}
pub async fn create_game_handler(
    State(state): State<AppState>,
    Json(payload): Json<AddGamePayload>,
) -> Result<Json<Uuid>, (StatusCode, String)> {
    let id = create_game(
        payload.name,
        payload.description,
        payload.image_url,
        payload.tags,
        payload.min_players,
        state.redis.clone(),
    )
    .await
    .map_err(|e| {
        tracing::error!("Error adding new game: {}", e);
        e.to_response()
    })?;

    tracing::info!("Success adding game {id}");
    Ok(Json(id))
}

pub async fn get_game_handler(
    Path(game_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<GameType>, (StatusCode, String)> {
    let game = get_game(game_id, state.redis.clone()).await.map_err(|e| {
        tracing::error!("Error retrieving {} game: {}", game_id, e);
        e.to_response()
    })?;

    tracing::info!("Success retrieving {game_id} game");
    Ok(Json(game))
}

pub async fn get_all_games_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<GameType>>, (StatusCode, String)> {
    let games = get_all_games(state.redis.clone()).await.map_err(|e| {
        tracing::error!("Error retrieving all games: {}", e);
        e.to_response()
    })?;

    tracing::info!("Success retrieving all game");
    Ok(Json(games))
}
