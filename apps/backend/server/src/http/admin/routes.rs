use axum::{Router, routing::{patch, post, put}};

use crate::{
    http::{games::handlers::toggle_game_active, seasons::handlers::{create_season, update_season}},
    state::AppState,
};

/// Mounted at `/api/admin`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/season", post(create_season))
        .route("/season/{season_id}", put(update_season))
        .route("/game/{game_id}/active", patch(toggle_game_active))
}
