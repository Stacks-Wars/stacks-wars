// Clean, single-definition lobby websocket handler loop.
use axum::extract::ws::Message;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use uuid::Uuid;

use crate::lobby::handler::LobbySession;
use crate::lobby::manager;
use crate::models::redis::LobbyStatus;
use crate::{
    db::{
        lobby_state::LobbyStateRepository, player_state::PlayerStateRepository,
        spectator_state::SpectatorStateRepository,
    },
    models::redis::spectator_state::SpectatorState,
    state::{AppState, ConnectionInfo},
};

use crate::lobby::handler::messages::{LobbyClientMessage, LobbyServerMessage};

/// The core websocket loop that listens for client messages and delegates
/// higher-level actions to `LobbyHandler` methods.
pub async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    lobby_id: Uuid,
    user_id: Uuid,
    state: AppState,
) {
    let (sender, mut receiver) = socket.split();

    let conn = Arc::new(ConnectionInfo {
        sender: Arc::new(TokioMutex::new(sender)),
    });

    manager::register_connection(&state, user_id, conn.clone()).await;

    // Ensure the connecting user exists as spectator if not a player
    let player_repo = PlayerStateRepository::new(state.redis.clone());
    if player_repo.exists(lobby_id, user_id).await.unwrap_or(false) == false {
        let spec_repo = SpectatorStateRepository::new(state.redis.clone());
        let spec = SpectatorState::new(user_id, lobby_id);
        let _ = spec_repo.upsert_state(spec).await;
    }

    // send initial state & players
    let lobby_state_repo = LobbyStateRepository::new(state.redis.clone());
    match lobby_state_repo.get_state(lobby_id).await {
        Ok(state_info) => {
            let started = matches!(state_info.status, LobbyStatus::InProgress);
            let lobby_msg = LobbyServerMessage::LobbyState {
                state: state_info.status.clone(),
                joined_players: None,
                started,
            };
            let _ = manager::send_to_connection(&conn, &lobby_msg).await;

            if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
                let players_msg = LobbyServerMessage::PlayerUpdated { players };
                let _ = manager::send_to_connection(&conn, &players_msg).await;
            }
        }
        Err(_) => {
            let _ = manager::send_to_connection(
                &conn,
                &LobbyServerMessage::Error {
                    message: "lobby not found".to_string(),
                },
            )
            .await;
            manager::unregister_connection(&state, &user_id).await;
            return;
        }
    }

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => match serde_json::from_str::<LobbyClientMessage>(&text) {
                Ok(LobbyClientMessage::Join) => {
                    let session = LobbySession::new(state.clone(), lobby_id, user_id);

                    match session.join_lobby().await {
                        Ok(Some(msg)) => {
                            let _ = manager::send_to_connection(&conn, &msg).await;
                        }
                        Ok(None) => {}
                        Err(e) => {
                            let _ = manager::send_to_connection(
                                &conn,
                                &LobbyServerMessage::Error {
                                    message: e.to_string(),
                                },
                            )
                            .await;
                        }
                    }
                }
                Ok(LobbyClientMessage::Leave) => {
                    let session = LobbySession::new(state.clone(), lobby_id, user_id);

                    match session.leave_lobby().await {
                        Ok(Some(msg)) => {
                            let _ = manager::send_to_connection(&conn, &msg).await;
                        }
                        Ok(None) => {}
                        Err(e) => {
                            let _ = manager::send_to_connection(
                                &conn,
                                &LobbyServerMessage::Error {
                                    message: e.to_string(),
                                },
                            )
                            .await;
                        }
                    }
                }
                Ok(LobbyClientMessage::ToggleStart) => {
                    let session = LobbySession::new(state.clone(), lobby_id, user_id);

                    match session.toggle_start_countdown().await {
                        Ok(Some(msg)) => {
                            let _ = manager::send_to_connection(&conn, &msg).await;
                        }
                        Ok(None) => {}
                        Err(e) => {
                            let _ = manager::send_to_connection(
                                &conn,
                                &LobbyServerMessage::Error {
                                    message: e.to_string(),
                                },
                            )
                            .await;
                        }
                    }
                }
                Ok(LobbyClientMessage::Ping { .. }) => {}
                Err(_) => {
                    let _ = manager::send_to_connection(
                        &conn,
                        &LobbyServerMessage::Error {
                            message: "invalid message".to_string(),
                        },
                    )
                    .await;
                }
            },
            Ok(Message::Binary(_)) => {}
            Ok(Message::Close(_)) | Ok(Message::Pong(_)) | Ok(Message::Ping(_)) => {}
            Err(e) => {
                tracing::debug!("ws recv err: {}", e);
                break;
            }
        }
    }

    manager::unregister_connection(&state, &user_id).await;

    // Broadcast final player list to lobby
    if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
        manager::broadcast(
            &state,
            lobby_id,
            &LobbyServerMessage::PlayerUpdated { players },
        )
        .await;
    }
}
