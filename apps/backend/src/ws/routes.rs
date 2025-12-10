use crate::{
    state::AppState,
    ws::{lobby::lobby_handler, room::room_handler},
};
use axum::{Router, routing::get};

/// Create WebSocket routes (grouped under `/ws`).
///
/// Routes:
/// - GET `/ws/room/{lobby_path}` - Connect to a specific lobby room (game + chat)
/// - GET `/ws/lobbies?status=waiting,starting` - Browse lobbies with optional status filter
pub fn create_ws_routes(state: AppState) -> Router {
    let ws_router = Router::new()
        .route("/room/{lobby_path}", get(room_handler))
        .route("/lobbies", get(lobby_handler))
        .with_state(state);

    Router::new().nest("/ws", ws_router)
}
