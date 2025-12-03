use crate::{state::AppState, ws::handlers::lobby_ws_handler};
use axum::{Router, routing::get};

/// Create WebSocket routes (grouped under `/ws`).
///
/// Currently includes:
/// - GET `/ws/lobby/{lobby_id}`
pub fn create_ws_routes(state: AppState) -> Router {
    Router::new()
        .route("/lobby/{lobby_id}", get(lobby_ws_handler))
        .with_state(state)
}
