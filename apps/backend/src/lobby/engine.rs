// Lobby engine - handles lobby-specific messages and state management
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use crate::db::join_request::{JoinRequestDTO, JoinRequestRepository, JoinRequestState};
use crate::db::lobby::LobbyRepository;
use crate::db::lobby_state::LobbyStateRepository;
use crate::db::player_state::PlayerStateRepository;
use crate::lobby::{
    LobbyError,
    messages::{LobbyClientMessage, LobbyServerMessage},
};
use crate::models::{LobbyStatus, PlayerState};
use crate::state::{AppState, ConnectionInfo};
use crate::ws::core::{hub, manager};
use chrono::Utc;

/// Helper to require authentication for a lobby action
async fn require_auth(conn: &Arc<ConnectionInfo>, auth_user_id: Option<Uuid>) -> Result<Uuid, ()> {
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

/// Handle an individual lobby message
pub async fn handle_lobby_message(
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
                let _ = hub::broadcast_to_lobby(
                    state,
                    lobby_id,
                    &LobbyServerMessage::PlayerJoined { player_id: user_id },
                )
                .await;

                if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
                    let _ = hub::broadcast_to_lobby(
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
                            let _ = hub::broadcast_to_lobby(
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

            let _ = hub::broadcast_to_lobby(
                state,
                lobby_id,
                &LobbyServerMessage::PlayerLeft { player_id: user_id },
            )
            .await;
            if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
                let _ = hub::broadcast_to_lobby(
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
                        let _ = hub::broadcast_to_lobby(
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
                    let _ = hub::broadcast_to_lobby(
                        &spawn_state,
                        spawn_lobby,
                        &LobbyServerMessage::LobbyStateChanged {
                            state: LobbyStatus::InProgress,
                        },
                    )
                    .await;

                    // Phase 4: Initialize game
                    let lobby_repo = LobbyRepository::new(spawn_state.postgres.clone());
                    let game_id = match lobby_repo.find_by_id(spawn_lobby).await {
                        Ok(Some(db_lobby)) => db_lobby.game_id,
                        _ => {
                            tracing::error!(
                                "Failed to fetch lobby metadata for game initialization"
                            );
                            return;
                        }
                    };

                    if let Some(factory) = spawn_state.game_registry.get(&game_id) {
                        let mut engine = factory(spawn_lobby);

                        // Get all player IDs in the lobby
                        let player_repo = PlayerStateRepository::new(spawn_state.redis.clone());
                        let player_ids = match player_repo.get_all_in_lobby(spawn_lobby).await {
                            Ok(players) => players.into_iter().map(|p| p.user_id).collect(),
                            Err(e) => {
                                tracing::error!(
                                    "Failed to fetch players for game initialization: {}",
                                    e
                                );
                                return;
                            }
                        };

                        // Initialize the game engine
                        match engine.initialize(player_ids).await {
                            Ok(events) => {
                                tracing::info!(
                                    "Game initialized successfully for lobby {}",
                                    spawn_lobby
                                );

                                // Store the active game engine
                                {
                                    let mut active_games = spawn_state.active_games.lock().await;
                                    active_games.insert(spawn_lobby, engine);
                                }

                                // Broadcast initialization events to all players
                                for event in events {
                                    let game_msg = crate::ws::message::JsonMessage::from(event);
                                    let _ = hub::broadcast_to_lobby_participants(
                                        &spawn_state,
                                        spawn_lobby,
                                        &game_msg,
                                    )
                                    .await;
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to initialize game: {}", e);
                            }
                        }
                    } else {
                        tracing::warn!("No game factory registered for game_id: {}", game_id);
                    }
                });
            }

            let _ = hub::broadcast_to_lobby(
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
                let _ = hub::broadcast_to_lobby(
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
            let _ = hub::broadcast_to_user(
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
                let _ = hub::broadcast_to_lobby(
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
            let _ = hub::broadcast_to_user(
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
                let _ = hub::broadcast_to_lobby(
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
            let _ = hub::broadcast_to_lobby(
                state,
                lobby_id,
                &LobbyServerMessage::PlayerKicked { player_id },
            )
            .await;
            if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
                let _ = hub::broadcast_to_lobby(
                    state,
                    lobby_id,
                    &LobbyServerMessage::PlayerUpdated { players },
                )
                .await;
            }
            let _ = hub::broadcast_to_user(
                state,
                lobby_id,
                player_id,
                &LobbyServerMessage::PlayerKicked { player_id },
            )
            .await;
        }
    }
}
