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

use crate::auth::extractors::WsAuth;
use crate::db::lobby::LobbyRepository;
use crate::middleware::{ApiRateLimit, check_rate_limit};
use crate::ws::core::manager;
use crate::ws::room::{RoomError, engine::handle_room_message, messages::RoomServerMessage};
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
/// - Try parsing as RoomClientMessage (lobby management)
/// - Try parsing as game_action (game-specific messages)
/// - Game developers control their own message validation
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
        context: ConnectionContext::Room(lobby_id),
        sender: Arc::new(TokioMutex::new(sender)),
    });

    // Register the connection
    manager::register_connection(&state, connection_id, conn.clone()).await;

    let lobby_repo = LobbyRepository::new(state.postgres.clone());
    let lobby_state_repo = LobbyStateRepository::new(state.redis.clone());
    let player_repo = PlayerStateRepository::new(state.redis.clone());
    let jr_repo = JoinRequestRepository::new(state.redis.clone());

    let (db_lobby_result, state_info_result, players_result, join_requests_result, chat_history_result) = tokio::join!(
        lobby_repo.find_by_id(lobby_id),
        lobby_state_repo.get_state(lobby_id),
        player_repo.get_all_in_lobby(lobby_id),
        jr_repo.list(lobby_id),
        crate::db::lobby_chat::get_chat_history(&state.redis, lobby_id, Some(50))
    );

    // Extract game_id for message routing (stored for entire connection lifecycle)
    let game_id = db_lobby_result.as_ref().ok().and_then(|opt| opt.as_ref().map(|l| l.game_id));

    // Validate we have the minimum required data
    match (db_lobby_result, state_info_result) {
        (Ok(Some(db_lobby)), Ok(state_info)) => {
            let lobby_ext = LobbyExtended::from_parts(db_lobby, state_info);
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
        (Ok(None), _) | (_, Err(_)) => {
            let err = RoomError::NotFound;
            let msg = RoomServerMessage::from(err);
            let _ = manager::send_to_connection(&conn, &msg).await;
            manager::unregister_connection(&state, &connection_id).await;
            return;
        }
        (Err(_), _) => {
            let err = RoomError::MetadataMissing;
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
                                if game_id.is_some() {
                                    handle_game_action(&state, lobby_id, user_id, action.clone())
                                        .await;
                                } else {
                                    tracing::warn!("Game action received but no game_id found");
                                }
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
                tracing::debug!("Unknown message type received: {:?}", parsed_msg.get("type"));
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
) {
    // Get the active game engine for this lobby
    let mut active_games = state.active_games.lock().await;
    if let Some(game_engine) = active_games.get_mut(&lobby_id) {
        // Handle the action and get response events
        match game_engine.handle_action(user_id, action).await {
            Ok(events) => {
                // Broadcast all response events to lobby participants
                for event in events {
                    let game_msg = crate::ws::core::message::JsonMessage::from(event);
                    let _ = crate::ws::broadcast::broadcast_room_participants(
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
                let game_error = crate::ws::core::message::JsonMessage::from(error_msg);
                let _ = crate::ws::broadcast::broadcast_user(state, user_id, &game_error).await;
            }
        }
    } else {
        tracing::warn!("No active game found for lobby {}", lobby_id);
    }
}
