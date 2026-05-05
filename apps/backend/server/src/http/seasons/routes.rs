use axum::{Router, routing::get};

use crate::state::AppState;

use super::handlers::{get_current_season, list_seasons};

/// Mounted at `/api/seasons`.
pub fn read_routes() -> Router<AppState> {
    Router::new()
        .route("/current", get(get_current_season))
        .route("/", get(list_seasons))
}
