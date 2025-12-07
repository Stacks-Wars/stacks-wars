use crate::{
    state::AppState,
    ws::{lobby::lobby_handler, room::room_handler},
};
use axum::{routing::get, Router};

/// Create WebSocket routes (grouped under `/ws`).
///
/// Routes:
/// - GET `/ws/room/{lobby_id}` - Connect to a specific lobby room (game + chat)
/// - GET `/ws/lobbies?status=waiting,starting` - Browse lobbies with optional status filter
pub fn create_ws_routes(state: AppState) -> Router {
    Router::new()
        .route("/room/{lobby_id}", get(room_handler))
        .route("/lobbies", get(lobby_handler))
        .with_state(state)
}
