// Lobby WebSocket handlers

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

pub use crate::lobby::handler::messages::{LobbyClientMessage, LobbyServerMessage};

/// WebSocket connection query parameters
#[derive(Deserialize)]
pub struct WsQueryParams {
    pub user_id: Uuid,
}

/// Lobby websocket entrypoint - upgrades an HTTP request to a websocket and
/// forwards the socket to the handler loop in `lobby::handler::websocket`.
pub async fn lobby_ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQueryParams>,
    Path(lobby_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    let user_id = query.user_id;
    Ok(ws.on_upgrade(move |socket| {
        crate::lobby::handler::websocket::handle_socket(socket, lobby_id, user_id, state)
    }))
}
