// Clean, single-definition lobby websocket handler loop.
use axum::extract::ws::Message;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use uuid::Uuid;

use crate::db::join_request::JoinRequestState;
use crate::db::lobby::LobbyRepository;
use crate::lobby::LobbyError;
use crate::lobby::manager;
use crate::models::redis::LobbyStatus;
use crate::ws::core::hub;
use crate::{
    db::{
        join_request::JoinRequestRepository, lobby_state::LobbyStateRepository,
        player_state::PlayerStateRepository,
    },
    models::redis::PlayerState,
    state::{AppState, ConnectionInfo},
};
use chrono::Utc;
use std::time::Duration;
use tokio::time::sleep;

use crate::db::join_request::JoinRequestDTO;
use crate::lobby::messages::{LobbyClientMessage, LobbyServerMessage};
use crate::models::db::LobbyExtended;

async fn require_auth(
    conn: &std::sync::Arc<ConnectionInfo>,
    auth_user_id: Option<Uuid>,
) -> Result<Uuid, ()> {
    match auth_user_id {
        Some(uid) => Ok(uid),
        None => {
            let err = LobbyError::NotAuthenticated;
            let msg = LobbyServerMessage::from(err);
            let _ = manager::send_to_connection(conn, &msg).await;
            Err(())
        }
    }
}

/// The core websocket loop that listens for client messages and delegates
/// higher-level actions to `LobbyHandler` methods.
pub async fn handle_socket(
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

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Try parsing as LobbyClientMessage first
                if let Ok(lobby_msg) = serde_json::from_str::<LobbyClientMessage>(&text) {
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
                        if let Some(gid) = game_id {
                            if state.game_registry.contains_key(&gid) {
                                // TODO: Route to game-specific message handler
                                // Each game will implement its own message parsing
                                tracing::debug!("Received game message for game_id: {}", gid);
                            } else {
                                tracing::warn!("Game not registered: {}", gid);
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

    manager::unregister_connection(&state, &connection_id).await;

    // Broadcast final player list to lobby
    if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
        hub::broadcast_lobby(
            &state,
            lobby_id,
            &LobbyServerMessage::PlayerUpdated { players },
        )
        .await;
    }
}

/// Handle an individual lobby message
async fn handle_lobby_message(
    lobby_msg: LobbyClientMessage,
    lobby_id: Uuid,
    auth_user_id: Option<Uuid>,
    conn: &Arc<ConnectionInfo>,
    state: &AppState,
    player_repo: &PlayerStateRepository,
    lobby_state_repo: &LobbyStateRepository,
) {
    // Check lobby status for gating (Phase 3)
    let lobby_status = match lobby_state_repo.get_state(lobby_id).await {
        Ok(ls) => ls.status,
        Err(_) => return, // Can't process without status
    };

    match lobby_msg {
        // UNIVERSAL: Ping is always allowed
        LobbyClientMessage::Ping { ts } => {
            let now_ms = Utc::now().timestamp_millis() as u64;
            let elapsed = now_ms.saturating_sub(ts);

            if let Some(user_id) = auth_user_id {
                if player_repo.exists(lobby_id, user_id).await.unwrap_or(false) {
                    let _ = player_repo.update_ping(lobby_id, user_id).await;
                }
            }

            let _ = manager::send_to_connection(
                conn,
                &LobbyServerMessage::Pong {
                    elapsed_ms: elapsed,
                },
            )
            .await;
        }

        // LOBBY-ONLY: Block if game is in progress
        LobbyClientMessage::Join => {
            if lobby_status == LobbyStatus::InProgress {
                let err = LobbyError::JoinFailed("Cannot join during active game".to_string());
                let msg = LobbyServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let jr_repo = JoinRequestRepository::new(state.redis.clone());
            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => return,
            };

            let allowed = match jr_repo.get(lobby_id, user_id).await {
                Some(jr) => matches!(jr.state, JoinRequestState::Accepted),
                None => true,
            };

            if allowed {
                // create or upsert player state
                let pstate = PlayerState::new(user_id, lobby_id, None, false);
                let _ = player_repo.upsert_state(pstate).await;

                // broadcast joined and updated player list
                let _ = hub::broadcast_lobby(
                    state,
                    lobby_id,
                    &LobbyServerMessage::PlayerJoined { player_id: user_id },
                )
                .await;

                if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
                    let _ = hub::broadcast_lobby(
                        state,
                        lobby_id,
                        &LobbyServerMessage::PlayerUpdated { players },
                    )
                    .await;
                }

                let lobby_repo = LobbyRepository::new(state.postgres.clone());
                if let Ok(Some(db_lobby)) = lobby_repo.find_by_id(lobby_id).await {
                    if db_lobby.is_private {
                        let _ = jr_repo.remove(lobby_id, user_id).await.ok();
                        if let Ok(list) = jr_repo.list(lobby_id).await {
                            let dtos: Vec<JoinRequestDTO> =
                                list.into_iter().map(Into::into).collect();
                            let _ = hub::broadcast_lobby(
                                state,
                                lobby_id,
                                &LobbyServerMessage::JoinRequestsUpdated {
                                    join_requests: dtos,
                                },
                            )
                            .await;
                        }
                    }
                }
            } else {
                let err = LobbyError::JoinFailed("join request not accepted".to_string());
                let msg = LobbyServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
            }
        }

        LobbyClientMessage::Leave => {
            if lobby_status == LobbyStatus::InProgress {
                let err = LobbyError::JoinFailed("Cannot leave during active game".to_string());
                let msg = LobbyServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => return,
            };

            // Prevent the creator from leaving the lobby
            let is_creator = player_repo
                .is_creator(lobby_id, user_id)
                .await
                .unwrap_or(false);

            if is_creator {
                let err = LobbyError::NotCreator;
                let msg = LobbyServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            // remove player state
            let _ = player_repo.remove_from_lobby(lobby_id, user_id).await.ok();

            let _ = hub::broadcast_lobby(
                state,
                lobby_id,
                &LobbyServerMessage::PlayerLeft { player_id: user_id },
            )
            .await;
            if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
                let _ = hub::broadcast_lobby(
                    state,
                    lobby_id,
                    &LobbyServerMessage::PlayerUpdated { players },
                )
                .await;
            }
        }

        LobbyClientMessage::UpdateLobbyStatus { status } => {
            if lobby_status == LobbyStatus::InProgress {
                let err =
                    LobbyError::JoinFailed("Cannot change status during active game".to_string());
                let msg = LobbyServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => return,
            };

            // Only the lobby creator can change lobby status
            let is_creator = player_repo
                .is_creator(lobby_id, user_id)
                .await
                .unwrap_or(false);

            if !is_creator {
                let err = LobbyError::NotCreator;
                let msg = LobbyServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let _ = lobby_state_repo
                .update_status(lobby_id, status.clone())
                .await;
            if matches!(status, LobbyStatus::Starting) {
                let spawn_state = state.clone();
                let spawn_redis = state.redis.clone();
                let spawn_lobby = lobby_id;
                tokio::spawn(async move {
                    let spawn_repo = LobbyStateRepository::new(spawn_redis.clone());

                    // Countdown from 5 down to 0
                    for sec in (0..=5).rev() {
                        let _ = hub::broadcast_lobby(
                            &spawn_state,
                            spawn_lobby,
                            &LobbyServerMessage::StartCountdown {
                                seconds_remaining: sec as u8,
                            },
                        )
                        .await;

                        let _ = spawn_repo.set_countdown(spawn_lobby, sec as u8).await.ok();

                        if sec == 0 {
                            break;
                        }
                        sleep(Duration::from_secs(1)).await;

                        // If status changed or lobby missing, abort countdown.
                        let lobby_state_repo_bg = LobbyStateRepository::new(spawn_redis.clone());
                        if let Ok(ls) = lobby_state_repo_bg.get_state(spawn_lobby).await {
                            if !matches!(ls.status, LobbyStatus::Starting) {
                                return;
                            }
                        } else {
                            return;
                        }
                    }

                    // Clear countdown and mark started
                    let _ = spawn_repo.clear_countdown(spawn_lobby).await.ok();
                    let _ = spawn_repo.mark_started(spawn_lobby).await.ok();
                    let _ = hub::broadcast_lobby(
                        &spawn_state,
                        spawn_lobby,
                        &LobbyServerMessage::LobbyStateChanged {
                            state: LobbyStatus::InProgress,
                        },
                    )
                    .await;

                    // TODO Phase 4: Initialize game here
                    // if let Some(gid) = game_id {
                    //     if let Some(factory) = spawn_state.game_registry.get(&gid) {
                    //         let mut engine = factory();
                    //         let players = get_player_ids();
                    //         let events = engine.initialize(players);
                    //         // broadcast events
                    //     }
                    // }
                });
            }

            let _ = hub::broadcast_lobby(
                state,
                lobby_id,
                &LobbyServerMessage::LobbyStateChanged { state: status },
            )
            .await;
        }

        LobbyClientMessage::JoinRequest => {
            if lobby_status == LobbyStatus::InProgress {
                let err =
                    LobbyError::JoinFailed("Cannot request to join during active game".to_string());
                let msg = LobbyServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => return,
            };

            let jr_repo = JoinRequestRepository::new(state.redis.clone());
            let _ = jr_repo.create_pending(lobby_id, user_id, 15 * 60).await;
            if let Ok(list) = jr_repo.list(lobby_id).await {
                let dtos: Vec<JoinRequestDTO> = list.into_iter().map(Into::into).collect();
                let _ = hub::broadcast_lobby(
                    state,
                    lobby_id,
                    &LobbyServerMessage::JoinRequestsUpdated {
                        join_requests: dtos,
                    },
                )
                .await;
            }
        }

        LobbyClientMessage::ApproveJoin { player_id } => {
            if lobby_status == LobbyStatus::InProgress {
                let err =
                    LobbyError::JoinFailed("Cannot approve joins during active game".to_string());
                let msg = LobbyServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => return,
            };

            // Only creator can approve join requests
            let is_creator = player_repo
                .is_creator(lobby_id, user_id)
                .await
                .unwrap_or(false);

            if !is_creator {
                let err = LobbyError::NotCreator;
                let msg = LobbyServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let jr_repo = JoinRequestRepository::new(state.redis.clone());
            let _ = jr_repo
                .set_state(lobby_id, player_id, JoinRequestState::Accepted)
                .await;
            let _ = hub::send_to_user(
                state,
                lobby_id,
                player_id,
                &LobbyServerMessage::JoinRequestStatus {
                    player_id,
                    accepted: true,
                },
            )
            .await;
            if let Ok(list) = jr_repo.list(lobby_id).await {
                let dtos: Vec<JoinRequestDTO> = list.into_iter().map(Into::into).collect();
                let _ = hub::broadcast_lobby(
                    state,
                    lobby_id,
                    &LobbyServerMessage::JoinRequestsUpdated {
                        join_requests: dtos,
                    },
                )
                .await;
            }
        }

        LobbyClientMessage::RejectJoin { player_id } => {
            if lobby_status == LobbyStatus::InProgress {
                let err =
                    LobbyError::JoinFailed("Cannot reject joins during active game".to_string());
                let msg = LobbyServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => return,
            };

            // Only creator can reject join requests
            let is_creator = player_repo
                .is_creator(lobby_id, user_id)
                .await
                .unwrap_or(false);

            if !is_creator {
                let err = LobbyError::NotCreator;
                let msg = LobbyServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let jr_repo = JoinRequestRepository::new(state.redis.clone());
            let _ = jr_repo
                .set_state(lobby_id, player_id, JoinRequestState::Rejected)
                .await;
            let _ = hub::send_to_user(
                state,
                lobby_id,
                player_id,
                &LobbyServerMessage::JoinRequestStatus {
                    player_id,
                    accepted: false,
                },
            )
            .await;
            if let Ok(list) = jr_repo.list(lobby_id).await {
                let dtos: Vec<JoinRequestDTO> = list.into_iter().map(Into::into).collect();
                let _ = hub::broadcast_lobby(
                    state,
                    lobby_id,
                    &LobbyServerMessage::JoinRequestsUpdated {
                        join_requests: dtos,
                    },
                )
                .await;
            }
        }

        LobbyClientMessage::Kick { player_id } => {
            if lobby_status == LobbyStatus::InProgress {
                let err =
                    LobbyError::JoinFailed("Cannot kick players during active game".to_string());
                let msg = LobbyServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => return,
            };

            // Only creator can kick players
            let is_creator = player_repo
                .is_creator(lobby_id, user_id)
                .await
                .unwrap_or(false);

            if !is_creator {
                let err = LobbyError::NotCreator;
                let msg = LobbyServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            // remove player state
            let _ = player_repo
                .remove_from_lobby(lobby_id, player_id)
                .await
                .ok();
            let _ = hub::broadcast_lobby(
                state,
                lobby_id,
                &LobbyServerMessage::PlayerKicked { player_id },
            )
            .await;
            if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
                let _ = hub::broadcast_lobby(
                    state,
                    lobby_id,
                    &LobbyServerMessage::PlayerUpdated { players },
                )
                .await;
            }
            let _ = hub::send_to_user(
                state,
                lobby_id,
                player_id,
                &LobbyServerMessage::PlayerKicked { player_id },
            )
            .await;
        }
    }
}
