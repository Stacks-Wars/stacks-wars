use axum::{Router, routing::{get, post}};

use crate::state::AppState;

use super::handlers::{create_game, get_game, get_games_by_creator, list_games};

/// Mounted at `/api/games`.
pub fn read_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_games))
        .route("/by-creator/{creator_id}", get(get_games_by_creator))
        .route("/{identifier}", get(get_game))
}

/// Mounted at `/api/games` (auth rate limit).
pub fn auth_routes() -> Router<AppState> {
    Router::new().route("/", post(create_game))
}
