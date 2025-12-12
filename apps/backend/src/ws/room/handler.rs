// Room WebSocket handler - manages lobby room connections (game + chat)
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

use crate::db::lobby::LobbyRepository;
use crate::middleware::{ApiRateLimit, check_rate_limit};
use crate::ws::core::manager;
use crate::ws::room::{RoomError, engine::handle_room_message, messages::RoomServerMessage};
use crate::{auth::extractors::WsAuth, db::lobby_chat::get_chat_history};
use crate::{
    db::{
        join_request::JoinRequestRepository, lobby_state::LobbyStateRepository,
        player_state::PlayerStateRepository,
    },
    models::LobbyExtended,
    state::{AppState, ConnectionContext, ConnectionInfo},
};

/// HTTP endpoint: Upgrades an HTTP request to a WebSocket connection for lobby/game communication.
///
/// This is the entry point for all WebSocket connections. After rate limiting and authentication,
/// it upgrades the connection and hands off to `handle_socket` for message handling.
pub async fn room_handler(
    ws: WebSocketUpgrade,
    Path(lobby_path): Path<String>,
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

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, lobby_path, auth_user_id, state)))
}

/// Core WebSocket handler: Manages connection lifecycle and routes messages.
///
/// Responsibilities:
/// - Connection registration and initial bootstrap
/// - Message routing to lobby engine or game-specific handlers
/// - Connection cleanup on disconnect
///
/// Message routing logic:
/// - Try parsing as RoomClientMessage (lobby management)
/// - Try parsing as game_action (game-specific messages)
/// - Game developers control their own message validation
async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    lobby_path: String,
    auth_user_id: Option<Uuid>,
    state: AppState,
) {
    let (sender, mut receiver) = socket.split();
    let connection_id = Uuid::new_v4();

    let lobby_repo = LobbyRepository::new(state.postgres.clone());

    // Fetch lobby by path first to get the ID
    let db_lobby_result = lobby_repo.find_by_path(&lobby_path).await;
    let lobby = match db_lobby_result {
        Ok(db_lobby) => db_lobby,
        Err(_) => {
            let err = RoomError::NotFound;
            tracing::error!("Lobby not found for path {}: {:?}", lobby_path, err);
            return;
        }
    };
    let lobby_id = lobby.id;
    let game_path = lobby.game_path.clone();

    let conn = Arc::new(ConnectionInfo {
        connection_id,
        user_id: auth_user_id,
        context: ConnectionContext::Room(lobby_id),
        sender: Arc::new(TokioMutex::new(sender)),
    });

    // Register the connection
    manager::register_connection(&state, connection_id, conn.clone()).await;

    let lobby_state_repo = LobbyStateRepository::new(state.redis.clone());
    let player_repo = PlayerStateRepository::new(state.redis.clone());
    let jr_repo = JoinRequestRepository::new(state.redis.clone());

    let (state_info_result, players_result, join_requests_result, chat_history_result) = tokio::join!(
        lobby_state_repo.get_state(lobby_id),
        player_repo.get_all_in_lobby(lobby_id),
        jr_repo.list(lobby_id),
        get_chat_history(&state.redis, lobby_id, Some(50))
    );

    // Validate we have the minimum required data
    match state_info_result {
        Ok(state_info) => {
            let lobby_ext = LobbyExtended::from_parts(lobby, state_info);
            let players = players_result.unwrap_or_default();
            let join_requests = join_requests_result
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect();
            let chat_history = chat_history_result.unwrap_or_default();

            let _ = manager::send_to_connection(
                &conn,
                &RoomServerMessage::LobbyBootstrap {
                    lobby: lobby_ext,
                    players,
                    join_requests,
                    chat_history,
                },
            )
            .await;
        }
        Err(_) => {
            let err = RoomError::NotFound;
            let msg = RoomServerMessage::from(err);
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

                // Try parsing as RoomClientMessage
                if let Ok(room_msg) = serde_json::from_str(&text) {
                    let player_repo = PlayerStateRepository::new(state.redis.clone());
                    let lobby_state_repo = LobbyStateRepository::new(state.redis.clone());
                    handle_room_message(
                        room_msg,
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

                // Try parsing as game action message
                // Game developers control their own message validation and restrictions
                if let Some(msg_type) = parsed_msg.get("type").and_then(|v| v.as_str()) {
                    if msg_type == "game_action" {
                        if let Some(action) = parsed_msg.get("action") {
                            if let Some(user_id) = auth_user_id {
                                handle_game_action(
                                    &state,
                                    lobby_id,
                                    user_id,
                                    action.clone(),
                                    &game_path,
                                )
                                .await;
                            } else {
                                tracing::warn!("Game action from unauthenticated user");
                            }
                        } else {
                            tracing::warn!("Game action missing 'action' field");
                        }
                        continue;
                    }
                }

                // Unknown message type - log and ignore
                tracing::debug!(
                    "Unknown message type received: {:?}",
                    parsed_msg.get("type")
                );
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
    let player_repo = PlayerStateRepository::new(state.redis.clone());
    if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
        crate::ws::broadcast::broadcast_room(
            &state,
            lobby_id,
            &RoomServerMessage::PlayerUpdated { players },
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
    game_path: &str,
) {
    // Get the active game engine for this lobby
    let mut active_games = state.active_games.lock().await;
    if let Some(game_engine) = active_games.get_mut(&lobby_id) {
        // Handle the action and get response events
        match game_engine.handle_action(user_id, action).await {
            Ok(events) => {
                // Broadcast all response events wrapped with game identifier
                for event in events {
                    // Extract type and payload from the event
                    if let Some(obj) = event.as_object() {
                        if let Some(msg_type) = obj.get("type").and_then(|v| v.as_str()) {
                            // Wrap with game identifier for frontend router
                            let wrapped_msg = serde_json::json!({
                                "game": game_path,
                                "type": msg_type,
                                "payload": event
                            });

                            let game_msg = crate::ws::core::message::JsonMessage::from(wrapped_msg);
                            let _ = crate::ws::broadcast::broadcast_room_participants(
                                state, lobby_id, &game_msg,
                            )
                            .await;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Game action handling failed for lobby {}: {}", lobby_id, e);

                // Send error message back to the specific user (wrapped with game identifier)
                let wrapped_error = serde_json::json!({
                    "game": game_path,
                    "type": "gameError",
                    "payload": {
                        "message": e.to_string()
                    }
                });
                let game_error = crate::ws::core::message::JsonMessage::from(wrapped_error);
                let _ = crate::ws::broadcast::broadcast_user(state, user_id, &game_error).await;
            }
        }
    } else {
        tracing::warn!("No active game found for lobby {}", lobby_id);
    }
}
