use crate::state::AppState;
use crate::ws::handlers::lobby::lobby_ws_handler;
use axum::{Router, routing::get};

/// Create WebSocket routes (grouped under `/ws`).
///
/// Currently includes:
/// - GET `/ws/lobby/{lobby_id}?user_id={uuid}` -> lobby websocket upgrade
pub fn create_ws_routes(state: AppState) -> Router {
    Router::new()
        .route("/lobby/{lobby_id}", get(lobby_ws_handler))
        .with_state(state)
}
