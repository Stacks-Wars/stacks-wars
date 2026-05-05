use axum::{Router, routing::{get, post}};

use crate::state::AppState;

use super::handlers;

/// Mounted at `/api/session`.
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(handlers::get_me))
        .route("/unclaimed-reward", get(handlers::get_unclaimed_rewards))
        .route("/logout", post(handlers::logout))
}
