use axum::{Router, routing::get};

use crate::state::AppState;

use super::handlers::{health_handler, root_handler};

/// Public routes — no authentication or rate limiting.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_handler))
        .route("/", get(root_handler))
}
