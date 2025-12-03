// Connection handlers - manages lobby and game WebSocket connections
//
// This module handles the full lifecycle of WebSocket connections:
// - HTTP upgrade to WebSocket
// - Connection registration and bootstrap
// - Message routing to lobby or game handlers based on state
// - Connection cleanup

use axum::{
    extract::{ConnectInfo, Path, State, WebSocketUpgrade, ws::Message},
    response::IntoResponse,
};
use futures::StreamExt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use uuid::Uuid;

use crate::auth::extractors::WsAuth;
use crate::db::lobby::LobbyRepository;
use crate::lobby::{LobbyError, engine::handle_lobby_message, messages::LobbyServerMessage};
use crate::middleware::{ApiRateLimit, check_rate_limit};
use crate::models::redis::LobbyStatus;
use crate::ws::core::{hub, manager};
use crate::{
    db::{
        join_request::JoinRequestRepository, lobby_state::LobbyStateRepository,
        player_state::PlayerStateRepository,
    },
    models::db::LobbyExtended,
    state::{AppState, ConnectionInfo},
};

/// HTTP endpoint: Upgrades an HTTP request to a WebSocket connection for lobby/game communication.
///
/// This is the entry point for all WebSocket connections. After rate limiting and authentication,
/// it upgrades the connection and hands off to `handle_socket` for message handling.
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

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, lobby_id, auth_user_id, state)))
}

/// Core WebSocket handler: Manages connection lifecycle and routes messages.
///
/// Responsibilities:
/// - Connection registration and initial bootstrap
/// - Message routing to lobby engine or game-specific handlers
/// - Connection cleanup on disconnect
///
/// Message routing logic:
/// - Try parsing as LobbyClientMessage first
/// - If that fails and lobby status is InProgress, try game-specific parsing
/// - Otherwise, send invalid message error
async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    lobby_id: Uuid,
    auth_user_id: Option<Uuid>,
    state: AppState,
) {
    let (sender, mut receiver) = socket.split();
    let connection_id = Uuid::new_v4();
    let conn = Arc::new(ConnectionInfo {
        connection_id,
        user_id: auth_user_id,
        lobby_id,
        sender: Arc::new(TokioMutex::new(sender)),
    });
    let player_repo = PlayerStateRepository::new(state.redis.clone());

    // Register the connection (conn_id may be authenticated user_id or generated spectator id)
    manager::register_connection(&state, connection_id, conn.clone()).await;

    // send initial state & players
    let lobby_state_repo = LobbyStateRepository::new(state.redis.clone());

    // Fetch lobby metadata to get game_id for game message routing
    let lobby_repo = LobbyRepository::new(state.postgres.clone());
    let game_id = match lobby_repo.find_by_id(lobby_id).await {
        Ok(Some(db_lobby)) => Some(db_lobby.game_id),
        _ => None,
    };

    match lobby_state_repo.get_state(lobby_id).await {
        Ok(state_info) => {
            // Build lobby bootstrap: require Postgres lobby metadata and combine with runtime state.
            let lobby_repo = LobbyRepository::new(state.postgres.clone());
            let lobby_ext = match lobby_repo.find_by_id(lobby_id).await {
                Ok(Some(db_lobby)) => LobbyExtended::from_parts(db_lobby, state_info.clone()),
                _ => {
                    let err = LobbyError::MetadataMissing;
                    let msg = LobbyServerMessage::from(err);
                    let _ = manager::send_to_connection(&conn, &msg).await;
                    manager::unregister_connection(&state, &connection_id).await;
                    return;
                }
            };

            // Fetch current players and join-requests so clients receive a full bootstrap.
            let players = match player_repo.get_all_in_lobby(lobby_id).await {
                Ok(p) => p,
                Err(_) => Vec::new(),
            };

            let jr_repo = JoinRequestRepository::new(state.redis.clone());
            let join_requests = match jr_repo.list(lobby_id).await {
                Ok(list) => list.into_iter().map(Into::into).collect(),
                Err(_) => Vec::new(),
            };

            let _ = manager::send_to_connection(
                &conn,
                &LobbyServerMessage::LobbyBootstrap {
                    lobby: lobby_ext,
                    players,
                    join_requests,
                },
            )
            .await;
        }
        Err(_) => {
            let err = LobbyError::NotFound;
            let msg = LobbyServerMessage::from(err);
            let _ = manager::send_to_connection(&conn, &msg).await;
            manager::unregister_connection(&state, &connection_id).await;
            return;
        }
    }

    // Main message loop
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Parse message as JSON first
                let parsed_msg: serde_json::Value = match serde_json::from_str(&text) {
                    Ok(msg) => msg,
                    Err(_) => {
                        tracing::warn!("Invalid JSON message received");
                        continue;
                    }
                };

                // Try parsing as LobbyClientMessage first
                if let Ok(lobby_msg) = serde_json::from_str(&text) {
                    handle_lobby_message(
                        lobby_msg,
                        lobby_id,
                        auth_user_id,
                        &conn,
                        &state,
                        &player_repo,
                        &lobby_state_repo,
                    )
                    .await;
                    continue;
                }

                // Not a lobby message - check if it's a game message
                match lobby_state_repo.get_state(lobby_id).await {
                    Ok(lobby_state) if lobby_state.status == LobbyStatus::InProgress => {
                        // Game is active - attempt to handle as game message
                        if let Some(_gid) = game_id {
                            // Parse game action message
                            if let Some(msg_type) = parsed_msg.get("type").and_then(|v| v.as_str())
                            {
                                if msg_type == "game_action" {
                                    if let Some(action) = parsed_msg.get("action") {
                                        if let Some(user_id) = auth_user_id {
                                            handle_game_action(
                                                &state,
                                                lobby_id,
                                                user_id,
                                                action.clone(),
                                            )
                                            .await;
                                        } else {
                                            tracing::warn!("Game action from unauthenticated user");
                                        }
                                    } else {
                                        tracing::warn!("Game action missing 'action' field");
                                    }
                                } else {
                                    tracing::debug!("Unknown game message type: {}", msg_type);
                                }
                            } else {
                                tracing::warn!("Game message missing 'type' field");
                            }
                        }
                    }
                    _ => {
                        // Game not active or state fetch failed - invalid message
                        let err = LobbyError::InvalidMessage;
                        let msg = LobbyServerMessage::from(err);
                        let _ = manager::send_to_connection(&conn, &msg).await;
                    }
                }
            }

            Ok(Message::Binary(_)) => {}
            Ok(Message::Close(_)) | Ok(Message::Pong(_)) | Ok(Message::Ping(_)) => {}
            Err(e) => {
                tracing::debug!("ws recv err: {}", e);
                break;
            }
        }
    }

    // Cleanup on disconnect
    manager::unregister_connection(&state, &connection_id).await;

    // Broadcast final player list to lobby
    if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
        hub::broadcast_to_lobby(
            &state,
            lobby_id,
            &LobbyServerMessage::PlayerUpdated { players },
        )
        .await;
    }
}

/// Handle a game action message from a player
async fn handle_game_action(
    state: &AppState,
    lobby_id: Uuid,
    user_id: Uuid,
    action: serde_json::Value,
) {
    // Get the active game engine for this lobby
    let mut active_games = state.active_games.lock().await;
    if let Some(game_engine) = active_games.get_mut(&lobby_id) {
        // Handle the action and get response events
        match game_engine.handle_action(user_id, action).await {
            Ok(events) => {
                // Broadcast all response events to lobby participants
                for event in events {
                    let game_msg = crate::ws::message::JsonMessage::from(event);
                    let _ = crate::ws::core::hub::broadcast_to_lobby_participants(
                        state, lobby_id, &game_msg,
                    )
                    .await;
                }
            }
            Err(e) => {
                tracing::error!("Game action handling failed for lobby {}: {}", lobby_id, e);

                // Send error message back to the specific user
                let error_msg = serde_json::json!({
                    "type": "game_error",
                    "message": e.to_string()
                });
                let game_error = crate::ws::message::JsonMessage::from(error_msg);
                let _ =
                    crate::ws::core::hub::broadcast_to_user(state, lobby_id, user_id, &game_error)
                        .await;
            }
        }
    } else {
        tracing::warn!("No active game found for lobby {}", lobby_id);
    }
}
