use axum::{Router, routing::get};

use crate::state::AppState;

use super::handlers::{
    get_game_leaderboard_handler, get_leaderboard_handler, get_platform_game_stats_handler,
    get_player_leaderboard, get_user_top_games_handler,
};

/// Mounted at `/api/leaderboards`.
pub fn read_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_leaderboard_handler))
        .route("/game/{game_id}", get(get_game_leaderboard_handler))
        .route(
            "/user/{user_id}/top-games",
            get(get_user_top_games_handler),
        )
        .route("/stats/games", get(get_platform_game_stats_handler))
        .route("/{user_id}", get(get_player_leaderboard))
}
