use axum::{Router, routing::get};

use crate::state::AppState;

use super::handlers::get_platform_stats;

/// Mounted at `/api/stats`.
pub fn read_routes() -> Router<AppState> {
    Router::new().route("/", get(get_platform_stats))
}
