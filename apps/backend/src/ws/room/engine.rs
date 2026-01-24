// Lobby engine - handles lobby-specific messages and state management
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use crate::db::join_request::{JoinRequestRepository, JoinRequestState};
use crate::db::lobby::LobbyRepository;
use crate::db::lobby_chat::LobbyChatRepository;
use crate::db::lobby_state::LobbyStateRepository;
use crate::db::player_state::PlayerStateRepository;
use crate::db::user::UserRepository;
use crate::models::{LobbyStatus, PlayerState};
use crate::state::{AppState, ConnectionInfo};
use crate::ws::room::{
    RoomError,
    messages::{RoomClientMessage, RoomServerMessage},
};
use crate::ws::{broadcast, core::manager};
use chrono::Utc;

/// Helper to require authentication for a lobby action
async fn require_auth(conn: &Arc<ConnectionInfo>, auth_user_id: Option<Uuid>) -> Result<Uuid, ()> {
    match auth_user_id {
        Some(uid) => Ok(uid),
        None => {
            let err = RoomError::NotAuthenticated;
            let msg = RoomServerMessage::from(err);
            let _ = manager::send_to_connection(conn, &msg).await;
            Err(())
        }
    }
}

/// Handle an individual lobby message
pub async fn handle_room_message(
    room_msg: RoomClientMessage,
    lobby_id: Uuid,
    auth_user_id: Option<Uuid>,
    conn: &Arc<ConnectionInfo>,
    state: &AppState,
    player_repo: &PlayerStateRepository,
    lobby_state_repo: &LobbyStateRepository,
) {
    let lobby_status = match lobby_state_repo.get_state(lobby_id).await {
        Ok(ls) => ls.status,
        Err(_) => return, // Can't process without status
    };

    match room_msg {
        RoomClientMessage::Ping { ts } => {
            let now_ms = Utc::now().timestamp_millis() as u64;
            let elapsed = now_ms.saturating_sub(ts);

            let _ = manager::send_to_connection(
                conn,
                &RoomServerMessage::Pong {
                    elapsed_ms: elapsed,
                },
            )
            .await;

            if let Some(user_id) = auth_user_id {
                if player_repo.exists(lobby_id, user_id).await.unwrap_or(false) {
                    let _ = player_repo.update_ping(lobby_id, user_id).await;
                }
            }
        }

        // LOBBY-ONLY: Block if game is in progress (i guess ...)
        RoomClientMessage::Join => {
            if lobby_status == LobbyStatus::InProgress {
                let err = RoomError::JoinFailed("Cannot join during active game".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let jr_repo = JoinRequestRepository::new(state.redis.clone());
            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => {
                    let _ = manager::send_to_connection(
                        conn,
                        &RoomServerMessage::from(RoomError::JoinFailed(
                            "not authenticated".to_string(),
                        )),
                    )
                    .await;
                    return;
                }
            };

            // Check join request (for private lobbies) or allow direct join (public lobbies)
            let join_request = jr_repo.get(lobby_id, user_id).await;

            let allowed = match &join_request {
                Some(jr) => matches!(jr.state, JoinRequestState::Accepted),
                None => true, // Public lobby, allow direct join
            };

            if allowed {
                // TODO: kinda buggy if user changed profile between join request and join
                let (wallet_address, username, display_name, trust_rating) = match join_request {
                    Some(jr) => (
                        jr.wallet_address,
                        jr.username,
                        jr.display_name,
                        jr.trust_rating,
                    ),
                    None => {
                        let user_repo = UserRepository::new(state.postgres.clone());
                        let user = match user_repo.find_by_id(user_id).await {
                            Ok(u) => u,
                            Err(e) => {
                                let msg =
                                    RoomServerMessage::from(RoomError::JoinFailed(e.to_string()));
                                let _ = manager::send_to_connection(conn, &msg).await;
                                return;
                            }
                        };
                        (
                            user.wallet_address.to_string(),
                            user.username,
                            user.display_name,
                            user.trust_rating,
                        )
                    }
                };

                // Create or upsert player state with user data
                let pstate = PlayerState::new(
                    user_id,
                    lobby_id,
                    wallet_address,
                    username,
                    display_name,
                    trust_rating,
                    None,
                    false,
                );
                let _ = player_repo.upsert_state(pstate, Some(state.clone())).await;

                let participant_count = lobby_state_repo
                    .increment_participants(lobby_id)
                    .await
                    .unwrap_or(0);

                // broadcast joined and updated player list
                let _ = broadcast::broadcast_user(
                    state,
                    user_id,
                    &RoomServerMessage::PlayerJoined { user_id },
                )
                .await;

                if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
                    let _ = broadcast::broadcast_room(
                        state,
                        lobby_id,
                        &RoomServerMessage::PlayerUpdated { players },
                    )
                    .await;
                }

                let lobby_repo = LobbyRepository::new(state.postgres.clone());
                let db_lobby = lobby_repo.find_by_id(lobby_id).await.ok();

                let current_amount = db_lobby.as_ref().and_then(|l| l.current_amount);

                // Broadcast lobby status change with updated participant count and current amount
                let _ = broadcast::broadcast_room(
                    state,
                    lobby_id,
                    &RoomServerMessage::LobbyStatusChanged {
                        status: lobby_status,
                        participant_count,
                        current_amount,
                    },
                )
                .await;

                // Handle private lobby join request cleanup
                if let Some(lobby) = db_lobby {
                    if lobby.is_private {
                        let _ = jr_repo.remove(lobby_id, user_id).await.ok();
                        if let Ok(list) = jr_repo.list(lobby_id).await {
                            let _ = broadcast::broadcast_room(
                                state,
                                lobby_id,
                                &RoomServerMessage::JoinRequestsUpdated {
                                    join_requests: list,
                                },
                            );
                        }
                    }
                }
            } else {
                let err = RoomError::JoinFailed("join request not accepted".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
            }
        }

        RoomClientMessage::Leave => {
            if lobby_status == LobbyStatus::InProgress {
                let err = RoomError::LeaveFailed("Cannot leave during active game".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => {
                    let _ = manager::send_to_connection(
                        conn,
                        &RoomServerMessage::from(RoomError::LeaveFailed(
                            "not authenticated".to_string(),
                        )),
                    )
                    .await;
                    return;
                }
            };

            // Check if user is the creator
            let is_creator = player_repo
                .is_creator(lobby_id, user_id)
                .await
                .unwrap_or(false);

            if is_creator {
                // Creator can only leave if they are the only participant
                let participant_count = player_repo.count_players(lobby_id).await.unwrap_or(0);

                if participant_count == 1 {
                    // Delete the entire lobby
                    let lobby_repo = LobbyRepository::new(state.postgres.clone());
                    let lobby_chat_repo = LobbyChatRepository::new(state.redis.clone());

                    // Delete all resources
                    let _ = player_repo.cleanup_lobby(lobby_id).await;
                    let _ = lobby_state_repo.delete_state_soft(lobby_id).await;
                    let _ = lobby_chat_repo.cleanup_lobby(lobby_id).await;
                    let _ = lobby_repo.delete_lobby(lobby_id, Some(state.clone())).await;

                    let _ = broadcast::broadcast_room(
                        state,
                        lobby_id,
                        &RoomServerMessage::PlayerLeft { user_id },
                    )
                    .await;
                    return;
                } else {
                    // Creator cannot leave if there are other participants
                    let err = RoomError::LeaveFailed(
                        "Creator cannot leave while other players are in the lobby".to_string(),
                    );
                    let msg = RoomServerMessage::from(err);
                    let _ = manager::send_to_connection(conn, &msg).await;
                    return;
                }
            }

            // remove player state
            let _ = player_repo
                .delete_state(lobby_id, user_id, Some(state.clone()))
                .await
                .ok();

            let participant_count = lobby_state_repo
                .decrement_participants(lobby_id)
                .await
                .unwrap_or(0);

            let _ = broadcast::broadcast_user(
                state,
                user_id,
                &RoomServerMessage::PlayerLeft { user_id },
            )
            .await;
            if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
                let _ = broadcast::broadcast_room(
                    state,
                    lobby_id,
                    &RoomServerMessage::PlayerUpdated { players },
                )
                .await;
            }

            // Broadcast lobby status change with updated participant count and current amount
            let lobby_repo = LobbyRepository::new(state.postgres.clone());
            let current_amount = lobby_repo
                .find_by_id(lobby_id)
                .await
                .ok()
                .and_then(|l| l.current_amount);

            let _ = broadcast::broadcast_room(
                state,
                lobby_id,
                &RoomServerMessage::LobbyStatusChanged {
                    status: lobby_status,
                    participant_count,
                    current_amount,
                },
            )
            .await;
        }

        RoomClientMessage::UpdateLobbyStatus { status } => {
            if lobby_status == LobbyStatus::InProgress {
                let err = RoomError::LobbyStatusFailed(
                    "Cannot change status during active game".to_string(),
                );
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => {
                    let _ = manager::send_to_connection(
                        conn,
                        &RoomServerMessage::from(RoomError::LobbyStatusFailed(
                            "not authenticated".to_string(),
                        )),
                    )
                    .await;
                    return;
                }
            };

            // Only the lobby creator can change lobby status
            let is_creator = player_repo
                .is_creator(lobby_id, user_id)
                .await
                .unwrap_or(false);

            if !is_creator {
                let err = RoomError::LobbyStatusFailed(
                    "Only creator can change lobby status".to_string(),
                );
                let msg = RoomServerMessage::from(err);
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

                        let _ = broadcast::broadcast_room(
                            &spawn_state,
                            spawn_lobby,
                            &RoomServerMessage::StartCountdown {
                                seconds_remaining: sec as u8,
                            },
                        )
                        .await;
                    }

                    // Clear countdown and mark started
                    let _ = spawn_repo.clear_countdown(spawn_lobby).await.ok();
                    let _ = spawn_repo.mark_started(spawn_lobby).await.ok();

                    // Get participant count and current amount for broadcast
                    let participant_count = spawn_repo
                        .get_state(spawn_lobby)
                        .await
                        .map(|s| s.participant_count)
                        .unwrap_or(0);

                    let lobby_repo_spawn = LobbyRepository::new(spawn_state.postgres.clone());
                    let current_amount = lobby_repo_spawn
                        .find_by_id(spawn_lobby)
                        .await
                        .ok()
                        .and_then(|l| l.current_amount);

                    let _ = broadcast::broadcast_room(
                        &spawn_state,
                        spawn_lobby,
                        &RoomServerMessage::LobbyStatusChanged {
                            status: LobbyStatus::InProgress,
                            participant_count,
                            current_amount,
                        },
                    )
                    .await;

                    let lobby_repo = LobbyRepository::new(spawn_state.postgres.clone());
                    let game_id = match lobby_repo.find_by_id(spawn_lobby).await {
                        Ok(db_lobby) => db_lobby.game_id,
                        _ => {
                            tracing::error!(
                                "Failed to fetch lobby metadata for game initialization"
                            );
                            return;
                        }
                    };

                    if let Some(factory) = spawn_state.game_registry.get(&game_id) {
                        let mut engine = factory(spawn_lobby);

                        // Set app state before initialize so engine can access Redis/DB
                        engine.set_state(spawn_state.clone()).await;

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

                                // Start the game loop (for games with background tasks)
                                // This must be called BEFORE storing in active_games
                                // so the engine can set up internal state sharing
                                engine.start_loop(spawn_state.clone());

                                // Store the active game engine
                                {
                                    let mut active_games = spawn_state.active_games.lock().await;
                                    active_games.insert(spawn_lobby, engine);
                                }

                                // Broadcast initialization events to room
                                // These are RoomServerMessage variants (GameStarted, GameStartFailed)
                                // which should be broadcast directly without game wrapper
                                for event in events {
                                    let game_msg =
                                        crate::ws::core::message::JsonMessage::from(event);
                                    let _ = broadcast::broadcast_room(
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

            // Get current state and lobby info for broadcast
            let participant_count = lobby_state_repo
                .get_state(lobby_id)
                .await
                .map(|s| s.participant_count)
                .unwrap_or(0);

            let lobby_repo = LobbyRepository::new(state.postgres.clone());
            let current_amount = lobby_repo
                .find_by_id(lobby_id)
                .await
                .ok()
                .and_then(|l| l.current_amount);

            let _ = broadcast::broadcast_room(
                state,
                lobby_id,
                &RoomServerMessage::LobbyStatusChanged {
                    status: status,
                    participant_count,
                    current_amount,
                },
            )
            .await;
        }

        RoomClientMessage::JoinRequest => {
            if lobby_status == LobbyStatus::InProgress {
                let err =
                    RoomError::JoinFailed("Cannot request to join during active game".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => {
                    let _ = manager::send_to_connection(
                        conn,
                        &RoomServerMessage::from(RoomError::JoinFailed(
                            "not authenticated".to_string(),
                        )),
                    )
                    .await;
                    return;
                }
            };

            // Fetch user profile to include in join request
            let user_repo = UserRepository::new(state.postgres.clone());
            let user = match user_repo.find_by_id(user_id).await {
                Ok(u) => u,
                Err(e) => {
                    let msg = RoomServerMessage::from(RoomError::JoinFailed(e.to_string()));
                    let _ = manager::send_to_connection(conn, &msg).await;
                    return;
                }
            };

            let jr_repo = JoinRequestRepository::new(state.redis.clone());
            let _ = jr_repo
                .create_pending(
                    lobby_id,
                    user_id,
                    user.wallet_address.to_string(),
                    user.username,
                    user.display_name,
                    user.trust_rating,
                    15 * 60,
                )
                .await;
            if let Ok(list) = jr_repo.list(lobby_id).await {
                let _ = broadcast::broadcast_room(
                    state,
                    lobby_id,
                    &RoomServerMessage::JoinRequestsUpdated {
                        join_requests: list,
                    },
                )
                .await;
            }
        }

        RoomClientMessage::ApproveJoin {
            user_id: approved_user_id,
        } => {
            if lobby_status == LobbyStatus::InProgress {
                let err =
                    RoomError::ApproveFailed("Cannot approve joins during active game".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => {
                    let _ = manager::send_to_connection(
                        conn,
                        &RoomServerMessage::from(RoomError::ApproveFailed(
                            "not authenticated".to_string(),
                        )),
                    )
                    .await;
                    return;
                }
            };

            // Only creator can approve join requests
            let is_creator = player_repo
                .is_creator(lobby_id, user_id)
                .await
                .unwrap_or(false);

            if !is_creator {
                let err =
                    RoomError::ApproveFailed("Only creator can approve join request".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let jr_repo = JoinRequestRepository::new(state.redis.clone());
            let _ = jr_repo
                .set_state(lobby_id, approved_user_id, JoinRequestState::Accepted)
                .await;
            let _ = broadcast::broadcast_user(
                state,
                approved_user_id,
                &RoomServerMessage::JoinRequestStatus {
                    user_id: approved_user_id,
                    accepted: true,
                },
            )
            .await;
            if let Ok(list) = jr_repo.list(lobby_id).await {
                let _ = broadcast::broadcast_room(
                    state,
                    lobby_id,
                    &RoomServerMessage::JoinRequestsUpdated {
                        join_requests: list,
                    },
                )
                .await;
            }
        }

        RoomClientMessage::RejectJoin {
            user_id: rejected_user_id,
        } => {
            if lobby_status == LobbyStatus::InProgress {
                let err =
                    RoomError::RejectFailed("Cannot reject joins during active game".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => {
                    let _ = manager::send_to_connection(
                        conn,
                        &RoomServerMessage::from(RoomError::RejectFailed(
                            "not authenticated".to_string(),
                        )),
                    )
                    .await;
                    return;
                }
            };

            // Only creator can reject join requests
            let is_creator = player_repo
                .is_creator(lobby_id, user_id)
                .await
                .unwrap_or(false);

            if !is_creator {
                let err =
                    RoomError::RejectFailed("Only creator can reject join request".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let jr_repo = JoinRequestRepository::new(state.redis.clone());
            let _ = jr_repo
                .set_state(lobby_id, rejected_user_id, JoinRequestState::Rejected)
                .await;
            let _ = broadcast::broadcast_user(
                state,
                rejected_user_id,
                &RoomServerMessage::JoinRequestStatus {
                    user_id: rejected_user_id,
                    accepted: false,
                },
            )
            .await;
            if let Ok(list) = jr_repo.list(lobby_id).await {
                let _ = broadcast::broadcast_room(
                    state,
                    lobby_id,
                    &RoomServerMessage::JoinRequestsUpdated {
                        join_requests: list,
                    },
                )
                .await;
            }
        }

        RoomClientMessage::Kick {
            user_id: kicked_user_id,
        } => {
            if lobby_status == LobbyStatus::InProgress {
                let err =
                    RoomError::KickFailed("Cannot kick players during active game".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => {
                    let _ = manager::send_to_connection(
                        conn,
                        &RoomServerMessage::from(RoomError::KickFailed(
                            "not authenticated".to_string(),
                        )),
                    )
                    .await;
                    return;
                }
            };

            // Only creator can kick players
            let is_creator = player_repo
                .is_creator(lobby_id, user_id)
                .await
                .unwrap_or(false);

            if !is_creator {
                let err = RoomError::KickFailed("Only lobby creator can kick player".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            if user_id == kicked_user_id {
                let err = RoomError::KickFailed("Creator cannot kick themselves".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            // remove player state
            let _ = player_repo
                .delete_state(lobby_id, kicked_user_id, Some(state.clone()))
                .await
                .ok();

            let participant_count = lobby_state_repo
                .decrement_participants(lobby_id)
                .await
                .unwrap_or(0);

            let _ = broadcast::broadcast_room(
                state,
                lobby_id,
                &RoomServerMessage::PlayerKicked {
                    user_id: kicked_user_id,
                },
            )
            .await;
            if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
                let _ = broadcast::broadcast_room(
                    state,
                    lobby_id,
                    &RoomServerMessage::PlayerUpdated { players },
                )
                .await;
            }
            let _ = broadcast::broadcast_user(
                state,
                kicked_user_id,
                &RoomServerMessage::PlayerKicked {
                    user_id: kicked_user_id,
                },
            )
            .await;

            let lobby_repo = LobbyRepository::new(state.postgres.clone());
            let current_amount = lobby_repo
                .find_by_id(lobby_id)
                .await
                .ok()
                .and_then(|l| l.current_amount);

            let _ = broadcast::broadcast_room(
                state,
                lobby_id,
                &RoomServerMessage::LobbyStatusChanged {
                    status: lobby_status,
                    participant_count,
                    current_amount,
                },
            )
            .await;
        }

        RoomClientMessage::SendMessage { content, reply_to } => {
            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => return,
            };

            // Only participants (players + spectators) can send messages
            let is_participant = player_repo.exists(lobby_id, user_id).await.unwrap_or(false);

            if !is_participant {
                let err = RoomError::SendMessageFailed(
                    "Only lobby participants can send message".to_string(),
                );
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            // Create message
            match crate::db::lobby_chat::LobbyChatRepository::new(state.redis.clone())
                .create_message(lobby_id, user_id, &content, reply_to)
                .await
            {
                Ok(message) => {
                    let _ = broadcast::broadcast_room(
                        state,
                        lobby_id,
                        &RoomServerMessage::MessageReceived { message },
                    )
                    .await;
                }
                Err(e) => {
                    let err =
                        RoomError::SendMessageFailed(format!("Failed to create message: {}", e));
                    let msg = RoomServerMessage::from(err);
                    let _ = manager::send_to_connection(conn, &msg).await;
                }
            }
        }

        RoomClientMessage::AddReaction { message_id, emoji } => {
            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => {
                    let _ = manager::send_to_connection(
                        conn,
                        &RoomServerMessage::from(RoomError::ReactionFailed(
                            "not authenticated".to_string(),
                        )),
                    )
                    .await;
                    return;
                }
            };

            // Only participants can react
            let is_participant = player_repo.exists(lobby_id, user_id).await.unwrap_or(false);

            if !is_participant {
                let err = RoomError::ReactionFailed("Not in lobby".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            match crate::db::lobby_chat::LobbyChatRepository::new(state.redis.clone())
                .add_reaction(lobby_id, message_id, user_id, &emoji)
                .await
            {
                Ok(_) => {
                    let _ = broadcast::broadcast_room(
                        state,
                        lobby_id,
                        &RoomServerMessage::ReactionAdded {
                            message_id,
                            user_id,
                            emoji,
                        },
                    )
                    .await;
                }
                Err(e) => {
                    let err = RoomError::ReactionFailed(format!("Failed to add reaction: {}", e));
                    let msg = RoomServerMessage::from(err);
                    let _ = manager::send_to_connection(conn, &msg).await;
                }
            }
        }

        RoomClientMessage::RemoveReaction { message_id, emoji } => {
            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => {
                    let _ = manager::send_to_connection(
                        conn,
                        &RoomServerMessage::from(RoomError::ReactionFailed(
                            "not authenticated".to_string(),
                        )),
                    )
                    .await;
                    return;
                }
            };

            // Only participants can remove reactions
            let is_participant = player_repo.exists(lobby_id, user_id).await.unwrap_or(false);

            if !is_participant {
                let err = RoomError::ReactionFailed("Not in lobby".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            match crate::db::lobby_chat::LobbyChatRepository::new(state.redis.clone())
                .remove_reaction(lobby_id, message_id, user_id, &emoji)
                .await
            {
                Ok(_) => {
                    let _ = broadcast::broadcast_room(
                        state,
                        lobby_id,
                        &RoomServerMessage::ReactionRemoved {
                            message_id,
                            user_id,
                            emoji,
                        },
                    )
                    .await;
                }
                Err(e) => {
                    let err =
                        RoomError::ReactionFailed(format!("Failed to remove reaction: {}", e));
                    let msg = RoomServerMessage::from(err);
                    let _ = manager::send_to_connection(conn, &msg).await;
                }
            }
        }
    }
}
