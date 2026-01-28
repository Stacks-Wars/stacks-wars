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

use crate::ws::{broadcast_room, broadcast_user, core::manager};
use crate::{auth::extractors::WsAuth, db::lobby_chat::LobbyChatRepository};
use crate::{db::lobby::LobbyRepository, models::LobbyInfo};
use crate::{
    db::{game::GameRepository, user::UserRepository},
    middleware::{ApiRateLimit, check_rate_limit},
};
use crate::{
    db::{
        join_request::JoinRequestRepository, lobby_state::LobbyStateRepository,
        player_state::PlayerStateRepository,
    },
    models::LobbyExtended,
    state::{AppState, ConnectionContext, ConnectionInfo},
};
use crate::{
    models::LobbyStatus,
    ws::room::{RoomError, engine::handle_room_message, messages::RoomServerMessage},
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

    // Fetch lobby by path with joined user and game data
    let lobby = match lobby_repo.find_by_path(&lobby_path).await {
        Ok(l) => l,
        Err(_) => {
            let err = RoomError::NotFound;
            tracing::error!("Lobby not found for path {}: {:?}", lobby_path, err);
            return;
        }
    };

    let lobby_id = lobby.id;

    let conn = Arc::new(ConnectionInfo {
        connection_id,
        user_id: auth_user_id,
        context: ConnectionContext::Room(lobby_id),
        sender: Arc::new(TokioMutex::new(sender)),
    });

    // Register the connection
    manager::register_connection(&state, connection_id, conn.clone()).await;

    let game_repo = GameRepository::new(state.postgres.clone());
    let user_repo = UserRepository::new(state.postgres.clone());
    let lobby_state_repo = LobbyStateRepository::new(state.redis.clone());
    let player_repo = PlayerStateRepository::new(state.redis.clone());
    let jr_repo = JoinRequestRepository::new(state.redis.clone());

    let chat_repo = LobbyChatRepository::new(state.redis.clone());

    let contract_address = lobby.contract_address.clone();

    let (
        game,
        creator,
        state_info_result,
        players_result,
        join_requests_result,
        chat_history_result,
    ) = tokio::join!(
        game_repo.find_by_id(lobby.game_id),
        user_repo.find_by_id(lobby.creator_id),
        lobby_state_repo.get_state(lobby_id),
        player_repo.get_all_in_lobby(lobby_id),
        jr_repo.list(lobby_id),
        chat_repo.get_history(lobby_id, Some(50))
    );

    // Validate we have the minimum required data
    match (state_info_result, game, creator) {
        (Ok(state_info), Ok(game), Ok(creator)) => {
            let lobby_ext = LobbyExtended::from_parts(lobby, state_info);
            let lobby_status = lobby_ext.status;
            let players = players_result.unwrap_or_default();
            let join_requests = join_requests_result
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect();
            let chat_history = chat_history_result.unwrap_or_default();

            let lobby_info = LobbyInfo {
                lobby: lobby_ext,
                game,
                creator,
            };

            let _ = manager::send_to_connection(
                &conn,
                &RoomServerMessage::LobbyBootstrap {
                    lobby_info,
                    players,
                    join_requests,
                    chat_history,
                },
            )
            .await;

            // If game is in progress, send GameState for reconnecting user
            if lobby_status == LobbyStatus::InProgress {
                let active_games = state.active_games.lock().await;
                if let Some(game_engine) = active_games.get(&lobby_id) {
                    if let Ok(game_state) = game_engine.get_game_state(auth_user_id).await {
                        let _ = manager::send_to_connection(
                            &conn,
                            &RoomServerMessage::GameState { game_state },
                        )
                        .await;
                    }
                }
            }

            // If game is finished, send FinalStanding and GameOver for authenticated users
            if lobby_status == LobbyStatus::Finished {
                // Get all players sorted by rank for standings
                if let Ok(mut standings) = player_repo.get_all_in_lobby(lobby_id).await {
                    // Sort by rank (players with rank come first, sorted ascending)
                    standings.sort_by(|a, b| match (&a.rank, &b.rank) {
                        (Some(ra), Some(rb)) => ra.cmp(rb),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    });

                    let _ = manager::send_to_connection(
                        &conn,
                        &RoomServerMessage::FinalStanding {
                            standings: standings.clone(),
                        },
                    )
                    .await;

                    // Send GameOver to authenticated user if they were a participant
                    if let Some(user_id) = auth_user_id {
                        if let Some(player) = standings.iter().find(|p| p.user_id == user_id) {
                            if let Some(rank) = player.rank {
                                let _ = manager::send_to_connection(
                                    &conn,
                                    &RoomServerMessage::GameOver {
                                        rank,
                                        prize: player.prize,
                                        wars_point: player.wars_point.unwrap_or(0.0),
                                    },
                                )
                                .await;
                            }
                        }
                    }
                }
            }
        }
        _ => {
            let err = RoomError::NotFound;
            tracing::error!("Lobby state not found for id {}: {:?}", lobby_id, err);
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
                        contract_address.as_ref(),
                    )
                    .await;
                    continue;
                }

                // Try parsing as game action message wrapped in "game" object
                // Format: { "game": { "type": "submitWord", "word": "hello" } }
                if let Some(game_action) = parsed_msg.get("game") {
                    if let Some(user_id) = auth_user_id {
                        handle_game_action(&state, lobby_id, user_id, game_action.clone()).await;
                    } else {
                        tracing::warn!("Game action from unauthenticated user");
                    }
                    continue;
                }

                // Unknown message type - log and ignore
                tracing::warn!(
                    "Unknown message type received: {:?}",
                    parsed_msg.get("type")
                );
            }

            Ok(Message::Binary(_)) => {}
            Ok(Message::Close(_)) | Ok(Message::Pong(_)) | Ok(Message::Ping(_)) => {}
            Err(e) => {
                tracing::warn!("ws recv err: {}", e);
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
///
/// Input format: { "type": "submitWord", "word": "hello" }
/// (already extracted from the "game" wrapper)
///
/// Response events are wrapped back in the "game" format:
/// { "game": { "type": "...", ...fields } }
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
                // Broadcast all response events wrapped in "game" object to room
                for event in events {
                    // Wrap event in "game" object: { "game": { "type": "...", ...fields } }
                    let wrapped_msg = serde_json::json!({
                        "game": event
                    });

                    let game_msg = crate::ws::core::message::JsonMessage::from(wrapped_msg);
                    let _ = broadcast_room(state, lobby_id, &game_msg).await;
                }
            }
            Err(e) => {
                tracing::error!("Game action handling failed for lobby {}: {}", lobby_id, e);

                // Send error message back to the specific user
                let wrapped_error = serde_json::json!({
                    "game": {
                        "type": "error",
                        "message": e.to_string()
                    }
                });
                let game_error = crate::ws::core::message::JsonMessage::from(wrapped_error);
                let _ = broadcast_user(state, user_id, &game_error).await;
            }
        }
    } else {
        tracing::warn!("No active game found for lobby {}", lobby_id);
    }
}
