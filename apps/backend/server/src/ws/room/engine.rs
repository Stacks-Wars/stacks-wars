// Lobby engine - handles lobby-specific messages and state management
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use crate::db::game::GameRepository;
use crate::db::join_request::{JoinRequestRepository, JoinRequestState};
use crate::db::lobby::LobbyRepository;
use crate::db::lobby_chat::LobbyChatRepository;
use crate::db::lobby_state::LobbyStateRepository;
use crate::db::player_state::PlayerStateRepository;
use crate::db::user::UserRepository;
use crate::http::bot::broadcasts::delete_lobby_creation_message;
use crate::http::chain::stacks::has_joined;
use crate::models::player_state::ClaimState;
use crate::models::player_state::PlayerStatus;
use crate::models::{LobbyStatus, PlayerState, WalletAddress};
use crate::game_host::KernelGameHost;
use crate::state::{AppState, ConnectionInfo};
use stacks_wars_core::GameHostRef;
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
    contract_address: Option<&WalletAddress>,
) {
    let lobby_status = match lobby_state_repo.get_state(lobby_id).await {
        Ok(ls) => ls.status,
        Err(_) => return, // Can't process without status
    };

    match room_msg {
        RoomClientMessage::Ping { ts } => {
            let now_ms = Utc::now().timestamp_millis() as u64;
            let elapsed = now_ms.saturating_sub(ts);

            // Send personal pong response with latency
            let _ = manager::send_to_connection(
                conn,
                &RoomServerMessage::Pong {
                    elapsed_ms: elapsed,
                },
            )
            .await;

            if let Some(user_id) = auth_user_id {
                if player_repo.exists(lobby_id, user_id).await.unwrap_or(false) {
                    // Update player's last ping
                    let _ = player_repo.update_ping(lobby_id, user_id).await;

                    // If this user is the creator, also update creator ping and notify lobby list
                    if player_repo.is_creator(lobby_id, user_id).await.unwrap_or(false) {
                        let _ = lobby_state_repo.update_creator_ping(lobby_id).await;
                        broadcast::broadcast_lobby_update(state.clone(), lobby_id).await;
                    }

                    // Broadcast updated player list to the room so everyone sees activity status
                    if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
                        let msg = RoomServerMessage::PlayerUpdated { players };
                        broadcast::broadcast_room(state, lobby_id, &msg).await;
                    }
                }
            }
        }

        // Block joins when game is in progress or finished
        RoomClientMessage::Join => {
            if lobby_status == LobbyStatus::InProgress || lobby_status == LobbyStatus::Finished {
                let err = RoomError::JoinFailed("Cannot join during active or finished game".to_string());
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

                let wallet_address_obj = match WalletAddress::try_from(wallet_address.as_str()) {
                    Ok(addr) => addr,
                    Err(_) => {
                        let msg = RoomServerMessage::from(RoomError::JoinFailed(
                            "Invalid wallet address".to_string(),
                        ));
                        let _ = manager::send_to_connection(conn, &msg).await;
                        return;
                    }
                };

                // Check if player has joined the vault contract if present
                if let Some(contract_addr) = contract_address {
                    match has_joined(contract_addr, &wallet_address_obj, state).await {
                        Ok(true) => {} // Proceed
                        Ok(false) => {
                            let msg = RoomServerMessage::from(RoomError::JoinFailed(
                                "Player has not joined the vault contract".to_string(),
                            ));
                            let _ = manager::send_to_connection(conn, &msg).await;
                            return;
                        }
                        Err(e) => {
                            let msg = RoomServerMessage::from(RoomError::JoinFailed(format!(
                                "Failed to check contract join: {}",
                                e
                            )));
                            let _ = manager::send_to_connection(conn, &msg).await;
                            return;
                        }
                    }

                    // Increment current_amount by entry_amount if entry_amount exists and is > 0
                    let lobby_repo = LobbyRepository::new(state.postgres.clone());
                    if let Ok(lobby) = lobby_repo.find_by_id(lobby_id).await {
                        if let Some(entry_amt) = lobby.entry_amount {
                            if entry_amt > 0.0 {
                                let _ = lobby_repo
                                    .increment_current_amount(lobby_id, entry_amt, state.clone())
                                    .await;
                            }
                        }
                    }
                }

                // Create or upsert player state with user data
                let claim_state = if contract_address.is_some() {
                    Some(ClaimState::NotClaimed)
                } else {
                    None
                };
                let pstate = PlayerState::new(
                    user_id,
                    lobby_id,
                    wallet_address,
                    username,
                    display_name,
                    trust_rating,
                    claim_state,
                    false,
                    PlayerStatus::Joined,
                );
                let _ = player_repo
                    .upsert_state(pstate.clone(), Some(state.clone()))
                    .await;

                let participant_count = lobby_state_repo
                    .increment_participants(lobby_id)
                    .await
                    .unwrap_or(0);

                // broadcast joined and updated player list
                let _ = broadcast::broadcast_room(
                    state,
                    lobby_id,
                    &RoomServerMessage::PlayerJoined { player: pstate },
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

            // When finished, only players with NotJoined status can leave (for refunds)
            if lobby_status == LobbyStatus::Finished {
                if let Some(uid) = auth_user_id {
                    if let Ok(ps) = player_repo.get_state(lobby_id, uid).await {
                        if ps.status != PlayerStatus::NotJoined {
                            let err = RoomError::LeaveFailed(
                                "Only inactive players can leave a finished game".to_string(),
                            );
                            let msg = RoomServerMessage::from(err);
                            let _ = manager::send_to_connection(conn, &msg).await;
                            return;
                        }
                    }
                }
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
                    // Get lobby state for tg_msg_id before deletion
                    let lobby_state = lobby_state_repo.get_state(lobby_id).await.ok();

                    // Get player state before deletion for broadcast
                    let player = player_repo.get_state(lobby_id, user_id).await.ok();

                    // Delete Telegram message if exists
                    if let Some(lobby_state) = &lobby_state {
                        if let Some(tg_msg_id) = lobby_state.tg_msg_id {
                            delete_lobby_creation_message(state.clone(), tg_msg_id).await;
                        }
                    }

                    // Delete the entire lobby
                    let lobby_repo = LobbyRepository::new(state.postgres.clone());
                    let lobby_chat_repo = LobbyChatRepository::new(state.redis.clone());

                    // Delete all resources
                    let _ = player_repo.cleanup_lobby(lobby_id).await;
                    let _ = lobby_state_repo.delete_state_soft(lobby_id).await;
                    let _ = lobby_chat_repo.cleanup_lobby(lobby_id).await;
                    let _ = lobby_repo.delete_lobby(lobby_id, Some(state.clone())).await;

                    if let Some(player) = player {
                        let _ = broadcast::broadcast_room(
                            state,
                            lobby_id,
                            &RoomServerMessage::PlayerLeft { player },
                        )
                        .await;
                    }
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

            // Get player state before deletion for broadcast
            let player = player_repo.get_state(lobby_id, user_id).await.ok();

            // Decrement current_amount by entry_amount if contract_address exists and entry_amount > 0
            if contract_address.is_some() {
                let lobby_repo = LobbyRepository::new(state.postgres.clone());
                if let Ok(lobby) = lobby_repo.find_by_id(lobby_id).await {
                    if let Some(entry_amt) = lobby.entry_amount {
                        if entry_amt > 0.0 {
                            let _ = lobby_repo
                                .decrement_current_amount(lobby_id, entry_amt, state.clone())
                                .await;
                        }
                    }
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

            if let Some(player) = player {
                let _ = broadcast::broadcast_room(
                    state,
                    lobby_id,
                    &RoomServerMessage::PlayerLeft { player },
                )
                .await;
            }
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
            if lobby_status == LobbyStatus::InProgress || lobby_status == LobbyStatus::Finished {
                let err = RoomError::LobbyStatusFailed(
                    "Cannot change status during active/finished game".to_string(),
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

                        sleep(Duration::from_secs(1)).await;

                        // If status changed or lobby missing, abort countdown.
                        let lobby_state_repo_bg = LobbyStateRepository::new(spawn_redis.clone());
                        if let Ok(ls) = lobby_state_repo_bg.get_state(spawn_lobby).await {
                            if !matches!(ls.status, LobbyStatus::Starting) {
                                // Broadcast None to signal countdown cancellation
                                let _ = broadcast::broadcast_room(
                                    &spawn_state,
                                    spawn_lobby,
                                    &RoomServerMessage::StartCountdown {
                                        seconds_remaining: None,
                                    },
                                )
                                .await;
                                return;
                            }
                        } else {
                            return;
                        }

                        let _ = broadcast::broadcast_room(
                            &spawn_state,
                            spawn_lobby,
                            &RoomServerMessage::StartCountdown {
                                seconds_remaining: Some(sec as u8),
                            },
                        )
                        .await;

                        if sec == 0 {
                            break;
                        }
                    }

                    // Clear countdown and mark started
                    let _ = spawn_repo.clear_countdown(spawn_lobby).await.ok();

                    // Active Player Check
                    // Before starting the game, check which players are truly active.
                    // Players with last_ping older than 10 seconds are considered inactive.
                    let player_repo = PlayerStateRepository::new(spawn_state.redis.clone());
                    let all_players = match player_repo.get_all_in_lobby(spawn_lobby).await {
                        Ok(players) => players,
                        Err(e) => {
                            tracing::error!("Failed to fetch players for active check: {}", e);
                            return;
                        }
                    };

                    let now_ms = Utc::now().timestamp_millis() as u64;
                    const INACTIVE_THRESHOLD_MS: u64 = 10_000; // 10 seconds

                    let mut active_player_ids: Vec<Uuid> = Vec::new();
                    let mut inactive_player_ids: Vec<Uuid> = Vec::new();

                    for player in &all_players {
                        // Only check players who are currently Joined (participants)
                        if player.status != PlayerStatus::Joined {
                            continue;
                        }
                        let is_active = match player.last_ping {
                            Some(ping_ms) => now_ms.saturating_sub(ping_ms) <= INACTIVE_THRESHOLD_MS,
                            None => false,
                        };
                        if is_active {
                            active_player_ids.push(player.user_id);
                        } else {
                            inactive_player_ids.push(player.user_id);
                        }
                    }

                    // Get game min_players to validate
                    let lobby_repo_spawn = LobbyRepository::new(spawn_state.postgres.clone());
                    let db_lobby = match lobby_repo_spawn.find_by_id(spawn_lobby).await {
                        Ok(lobby) => lobby,
                        _ => {
                            tracing::error!(
                                "Failed to fetch lobby metadata for game initialization"
                            );
                            return;
                        }
                    };

                    let game_repo = GameRepository::new(spawn_state.postgres.clone());
                    let game = match game_repo.find_by_id(db_lobby.game_id).await {
                        Ok(g) => g,
                        Err(_) => {
                            tracing::error!("Failed to fetch game for min_players check");
                            return;
                        }
                    };

                    if (active_player_ids.len() as i16) < game.min_players {
                        // Not enough active players - revert to waiting
                        let _ = spawn_repo
                            .update_status(spawn_lobby, LobbyStatus::Waiting)
                            .await;
                        let _ = lobby_repo_spawn
                            .update_status(spawn_lobby, LobbyStatus::Waiting, spawn_state.clone())
                            .await;

                        let participant_count = spawn_repo
                            .get_state(spawn_lobby)
                            .await
                            .map(|s| s.participant_count)
                            .unwrap_or(0);

                        let current_amount = db_lobby.current_amount;

                        let _ = broadcast::broadcast_room(
                            &spawn_state,
                            spawn_lobby,
                            &RoomServerMessage::LobbyStatusChanged {
                                status: LobbyStatus::Waiting,
                                participant_count,
                                current_amount,
                            },
                        )
                        .await;

                        let game_msg = RoomServerMessage::GameStartFailed {
                            reason: format!(
                                "Need at least {} active players to start",
                                game.min_players
                            ),
                        };
                        let _ = broadcast::broadcast_room(
                            &spawn_state,
                            spawn_lobby,
                            &game_msg,
                        )
                        .await;
                        return;
                    }

                    // Batch update inactive players to NotJoined so they can leave later for refund
                    for &inactive_id in &inactive_player_ids {
                        let _ = player_repo
                            .update_status(spawn_lobby, inactive_id, PlayerStatus::NotJoined)
                            .await;
                    }

                    // Broadcast updated player list after status changes
                    if !inactive_player_ids.is_empty() {
                        if let Ok(players) = player_repo.get_all_in_lobby(spawn_lobby).await {
                            let _ = broadcast::broadcast_room(
                                &spawn_state,
                                spawn_lobby,
                                &RoomServerMessage::PlayerUpdated { players },
                            )
                            .await;
                        }
                    }

                    let _ = spawn_repo.mark_started(spawn_lobby).await.ok();
                    // Update PostgreSQL status to InProgress
                    let _ = lobby_repo_spawn
                        .update_status(spawn_lobby, LobbyStatus::InProgress, spawn_state.clone())
                        .await;

                    // Calculate the correct current_amount for the game:
                    // - Sponsored lobbies: use the full current_amount (sponsor put up the pool)
                    // - Normal lobbies: entry_amount * active_player_count
                    let game_current_amount = if db_lobby.is_sponsored {
                        db_lobby.current_amount
                    } else {
                        match db_lobby.entry_amount {
                            Some(entry) if entry > 0.0 => {
                                Some(entry * active_player_ids.len() as f64)
                            }
                            _ => db_lobby.current_amount,
                        }
                    };

                    // Get participant count for broadcast
                    let participant_count = spawn_repo
                        .get_state(spawn_lobby)
                        .await
                        .map(|s| s.participant_count)
                        .unwrap_or(0);

                    let _ = broadcast::broadcast_room(
                        &spawn_state,
                        spawn_lobby,
                        &RoomServerMessage::LobbyStatusChanged {
                            status: LobbyStatus::InProgress,
                            participant_count,
                            current_amount: game_current_amount,
                        },
                    )
                    .await;

                    let game_id = db_lobby.game_id;

                    if let Some(factory) = spawn_state.game_registry.get(&game_id) {
                        let host: GameHostRef = Arc::new(KernelGameHost(spawn_state.clone()));
                        let mut engine = factory(spawn_lobby, host.clone());

                        // Set lobby context with the calculated game amount
                        engine
                            .set_lobby_context(
                                game_id,
                                db_lobby.entry_amount,
                                game_current_amount,
                                db_lobby.is_sponsored,
                                db_lobby.creator_id,
                                db_lobby.token_symbol.clone(),
                                db_lobby.token_contract_id.map(|w| w.to_string()),
                            )
                            .await;

                        // Initialize the game engine with only active player IDs
                        match engine.initialize(active_player_ids).await {
                            Ok(events) => {
                                tracing::info!(
                                    "Game initialized successfully for lobby {}",
                                    spawn_lobby
                                );

                                // Start the game loop (for games with background tasks)
                                // This must be called BEFORE storing in active_games
                                // so the engine can set up internal state sharing
                                engine.start_loop(host);

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
            if lobby_status == LobbyStatus::InProgress || lobby_status == LobbyStatus::Finished {
                let err =
                    RoomError::JoinFailed("Cannot request to join during active or finished game".to_string());
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
            if matches!(lobby_status, LobbyStatus::InProgress | LobbyStatus::Starting | LobbyStatus::Finished) {
                let err =
                    RoomError::ApproveFailed("Cannot approve joins at this time".to_string());
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
            if matches!(lobby_status, LobbyStatus::InProgress | LobbyStatus::Starting | LobbyStatus::Finished) {
                let err =
                    RoomError::RejectFailed("Cannot reject joins at this time".to_string());
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
            if matches!(lobby_status, LobbyStatus::InProgress | LobbyStatus::Starting | LobbyStatus::Finished) {
                let err =
                    RoomError::KickFailed("Cannot kick players at this time".to_string());
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

            // Get player state before deletion for broadcast
            let kicked_player = player_repo.get_state(lobby_id, kicked_user_id).await.ok();

            // Decrement current_amount by entry_amount if contract_address exists and entry_amount > 0
            if contract_address.is_some() {
                let lobby_repo = LobbyRepository::new(state.postgres.clone());
                if let Ok(lobby) = lobby_repo.find_by_id(lobby_id).await {
                    if let Some(entry_amt) = lobby.entry_amount {
                        if entry_amt > 0.0 {
                            let _ = lobby_repo
                                .decrement_current_amount(lobby_id, entry_amt, state.clone())
                                .await;
                        }
                    }
                }
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

            if let Some(ref player) = kicked_player {
                let _ = broadcast::broadcast_room(
                    state,
                    lobby_id,
                    &RoomServerMessage::PlayerKicked {
                        player: player.clone(),
                    },
                )
                .await;
            }
            if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
                let _ = broadcast::broadcast_room(
                    state,
                    lobby_id,
                    &RoomServerMessage::PlayerUpdated { players },
                )
                .await;
            }

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
            if lobby_status == LobbyStatus::Finished {
                let err = RoomError::SendMessageFailed("Cannot send messages in a finished lobby".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

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
            if lobby_status == LobbyStatus::Finished {
                let err = RoomError::ReactionFailed("Cannot react in a finished lobby".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

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
            if lobby_status == LobbyStatus::Finished {
                let err = RoomError::ReactionFailed("Cannot react in a finished lobby".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

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

        RoomClientMessage::ClaimReward { tx_id } => {
            if matches!(lobby_status, LobbyStatus::Waiting | LobbyStatus::Starting) {
                let err = RoomError::ClaimFailed("Cannot claim reward before game ends".to_string());
                let msg = RoomServerMessage::from(err);
                let _ = manager::send_to_connection(conn, &msg).await;
                return;
            }

            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => return,
            };

            // Get player state
            let player_state = match player_repo.get_state(lobby_id, user_id).await {
                Ok(ps) => ps,
                Err(_) => {
                    let _ = manager::send_to_connection(
                        conn,
                        &RoomServerMessage::from(RoomError::ClaimFailed(
                            "Player not found in lobby".to_string(),
                        )),
                    )
                    .await;
                    return;
                }
            };

            // Check if has prize and not claimed
            if player_state.prize.is_none()
                || player_state.prize.unwrap() <= 0.0
                || player_state.has_claimed()
            {
                let _ = manager::send_to_connection(
                    conn,
                    &RoomServerMessage::from(RoomError::ClaimFailed(
                        "No prize available to claim".to_string(),
                    )),
                )
                .await;
                return;
            }

            let prize = player_state.prize.unwrap();

            // Update claim state
            if let Err(_) = player_repo
                .update_claim_state(
                    lobby_id,
                    user_id,
                    ClaimState::Claimed {
                        tx_id: tx_id.clone(),
                    },
                )
                .await
            {
                let _ = manager::send_to_connection(
                    conn,
                    &RoomServerMessage::from(RoomError::ClaimFailed(
                        "Failed to update claim state".to_string(),
                    )),
                )
                .await;
                return;
            }

            // Subtract from lobby current_amount
            if let Err(_) = lobby_state_repo
                .subtract_current_amount(lobby_id, prize)
                .await
            {
                let _ = manager::send_to_connection(
                    conn,
                    &RoomServerMessage::from(RoomError::ClaimFailed(
                        "Failed to update lobby amount".to_string(),
                    )),
                )
                .await;
                return;
            }

            // Send success
            let _ = manager::send_to_connection(conn, &RoomServerMessage::ClaimSuccess).await;
        }

        RoomClientMessage::ToggleParticipation { participate } => {
            if matches!(lobby_status, LobbyStatus::Starting | LobbyStatus::InProgress | LobbyStatus::Finished) {
                let err = RoomError::ParticipationFailed(
                    "Cannot toggle participation at this time".to_string(),
                );
                let _ = manager::send_to_connection(conn, &RoomServerMessage::from(err)).await;
                return;
            }

            let user_id = match require_auth(conn, auth_user_id).await {
                Ok(uid) => uid,
                Err(_) => return,
            };

            // Only the lobby creator can toggle participation
            let is_creator = player_repo
                .is_creator(lobby_id, user_id)
                .await
                .unwrap_or(false);

            if !is_creator {
                let err = RoomError::ParticipationFailed(
                    "Only the lobby creator can toggle participation".to_string(),
                );
                let _ = manager::send_to_connection(conn, &RoomServerMessage::from(err)).await;
                return;
            }

            // Check that the lobby is sponsored
            let lobby_repo = LobbyRepository::new(state.postgres.clone());
            let db_lobby = match lobby_repo.find_by_id(lobby_id).await {
                Ok(l) => l,
                Err(_) => {
                    let err = RoomError::ParticipationFailed(
                        "Lobby not found".to_string(),
                    );
                    let _ = manager::send_to_connection(conn, &RoomServerMessage::from(err)).await;
                    return;
                }
            };

            if !db_lobby.is_sponsored {
                let err = RoomError::ParticipationFailed(
                    "Participation toggle is only available for sponsored lobbies".to_string(),
                );
                let _ = manager::send_to_connection(conn, &RoomServerMessage::from(err)).await;
                return;
            }

            // Toggle the status
            let new_status = if participate {
                PlayerStatus::Joined
            } else {
                PlayerStatus::NotJoined
            };

            if let Err(e) = player_repo.update_status(lobby_id, user_id, new_status).await {
                let err = RoomError::ParticipationFailed(
                    format!("Failed to update status: {}", e),
                );
                let _ = manager::send_to_connection(conn, &RoomServerMessage::from(err)).await;
                return;
            }

            // Update participant count: increment when joining, decrement when leaving
            let participant_count = if participate {
                lobby_state_repo
                    .increment_participants(lobby_id)
                    .await
                    .unwrap_or(0)
            } else {
                lobby_state_repo
                    .decrement_participants(lobby_id)
                    .await
                    .unwrap_or(0)
            };

            // Broadcast participation toggle to room
            let _ = broadcast::broadcast_room(
                state,
                lobby_id,
                &RoomServerMessage::ParticipationToggled {
                    user_id,
                    participating: participate,
                },
            )
            .await;

            // Broadcast updated player list
            if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
                let _ = broadcast::broadcast_room(
                    state,
                    lobby_id,
                    &RoomServerMessage::PlayerUpdated { players },
                )
                .await;
            }

            // Broadcast lobby status change with updated participant count
            let current_amount = db_lobby.current_amount;
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
    }
}
