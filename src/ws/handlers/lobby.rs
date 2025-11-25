// Lobby WebSocket handlers

use axum::{
    extract::{ConnectInfo, Path, State, WebSocketUpgrade},
    response::IntoResponse,
};

use crate::auth::extractors::WsAuth;
use crate::middleware::ApiRateLimit;
use crate::middleware::check_rate_limit;
use crate::state::AppState;
use std::net::SocketAddr;

pub use crate::lobby::messages::{LobbyClientMessage, LobbyServerMessage};

/// Lobby websocket entrypoint - upgrades an HTTP request to a websocket and
/// forwards the socket to the handler loop in `lobby::websocket`.
pub async fn lobby_ws_handler(
    ws: WebSocketUpgrade,
    Path(lobby_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    WsAuth(auth): WsAuth,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    // Determine optional user id from auth claims
    let auth_user_id = auth.and_then(|claims| claims.user_id().ok());

    // Rate-limit the upgrade (fail early)
    let ip = addr.ip().to_string();
    if let Err((code, msg)) = check_rate_limit::<ApiRateLimit>(&state, &ip, auth_user_id).await {
        return Err((code, msg));
    }

    Ok(ws.on_upgrade(move |socket| {
        crate::lobby::websocket::handle_socket(socket, lobby_id, auth_user_id, state)
    }))
}
