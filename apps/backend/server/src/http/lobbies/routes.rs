use axum::{Router, routing::{get, post}};

use crate::state::AppState;

use super::handlers::{
    create_lobby, get_all_lobbies, get_lobby, list_lobbies_by_game_and_status, list_my_lobbies,
};

/// Mounted at `/api/lobbies`.
pub fn read_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/game/{game_identifier}/lobbies",
            get(list_lobbies_by_game_and_status),
        )
        .route("/", get(get_all_lobbies))
        .route("/my", get(list_my_lobbies))
        .route("/{identifier}", get(get_lobby))
}

/// Mounted at `/api/lobbies` (auth rate limit).
pub fn auth_routes() -> Router<AppState> {
    Router::new().route("/", post(create_lobby))
}
