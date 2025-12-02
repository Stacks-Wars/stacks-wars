use axum::{Router, routing::get};

use crate::{
    state::AppState,
    ws::handlers::{chat::chat_handler::chat_handler, lexi_wars_handler, lobby_ws_handler},
};

pub fn create_ws_routes(state: AppState) -> Router {
    Router::new()
        .route("/ws/lexiwars/{lobby_id}", get(lexi_wars_handler))
        .route("/ws/lobby/{lobby_id}", get(lobby_ws_handler))
        .route("/ws/chat/{lobby_id}", get(chat_handler))
        .with_state(state)
}
