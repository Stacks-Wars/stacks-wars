use axum::extract::ws::Message;
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use uuid::Uuid;

use crate::{
    db::lobby::{
        get::get_lobby_players,
        join_requests::{get_lobby_join_requests, get_player_join_request, update_join_request},
    },
    errors::AppError,
    models::{
        User,
        chat::ChatServerMessage,
        game::{Player, PlayerState},
        lobby::{JoinState, LobbyClientMessage, LobbyServerMessage, PendingJoin},
    },
    state::{ChatConnectionInfoMap, ConnectionInfoMap, RedisClient},
    ws::handlers::{
        chat::utils::send_chat_message_to_player,
        lobby::message_handler::{
            join_lobby::join_lobby, kick_player, last_ping, leave_lobby, permit_join, ping,
            request_join, request_leave, update_game_state, update_player_state,
        },
        utils::queue_message_for_player,
    },
};

pub async fn broadcast_to_lobby(
    lobby_id: Uuid,
    msg: &LobbyServerMessage,
    connections: &ConnectionInfoMap,
    chat_connections: Option<&ChatConnectionInfoMap>,
    redis: RedisClient,
) {
    let serialized = match serde_json::to_string(msg) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("Failed to serialize message: {}", e);
            return;
        }
    };

    if let Ok(players) = get_lobby_players(lobby_id, None, redis.clone()).await {
        let connection_guard = connections.lock().await;

        for player in &players {
            if let Some(conn_info) = connection_guard.get(&player.id) {
                // Try to send immediately
                let mut sender = conn_info.sender.lock().await;
                if let Err(e) = sender.send(Message::Text(serialized.clone().into())).await {
                    tracing::debug!("Failed to send message to player {}: {}", player.id, e);

                    // Only queue the message if it should be queued
                    if msg.should_queue() {
                        drop(sender); // Release the sender lock
                        drop(connection_guard); // Release the connection guard

                        if let Err(queue_err) = queue_message_for_player(
                            player.id,
                            lobby_id,
                            serialized.clone(),
                            &redis,
                        )
                        .await
                        {
                            tracing::error!(
                                "Failed to queue message for player {}: {}",
                                player.id,
                                queue_err
                            );
                        }
                    }

                    return; // Exit early to avoid double-locking
                }
            } else {
                // Player not connected, only queue if message should be queued
                if msg.should_queue() {
                    if let Err(e) =
                        queue_message_for_player(player.id, lobby_id, serialized.clone(), &redis)
                            .await
                    {
                        tracing::error!(
                            "Failed to queue message for offline player {}: {}",
                            player.id,
                            e
                        );
                    }
                }
            }
        }
    }

    // Notify chat connections about lobby changes
    if matches!(msg, LobbyServerMessage::PlayerUpdated { .. }) {
        if let Some(chat_conns) = chat_connections {
            notify_chat_about_lobby_changes(lobby_id, chat_conns, &redis).await;
        }
    }
}

async fn notify_chat_about_lobby_changes(
    lobby_id: Uuid,
    chat_connections: &ChatConnectionInfoMap,
    redis: &RedisClient,
) {
    // Get current lobby players
    if let Ok(lobby_players) =
        get_lobby_players(lobby_id, Some(PlayerState::Joined), redis.clone()).await
    {
        let lobby_player_ids: std::collections::HashSet<Uuid> =
            lobby_players.iter().map(|p| p.id).collect();

        let connection_guard = chat_connections.lock().await;

        // Check all chat connections and update permissions for players in this lobby
        for (&player_id, _) in connection_guard.iter() {
            let is_lobby_member = lobby_player_ids.contains(&player_id);
            let permit_msg = ChatServerMessage::PermitChat {
                allowed: is_lobby_member,
            };

            drop(connection_guard);

            send_chat_message_to_player(player_id, &permit_msg, chat_connections).await;

            return;
        }
    }
}

pub async fn send_error_to_player(
    player_id: Uuid,
    lobby_id: Uuid,
    message: impl Into<String>,
    connection_info: &ConnectionInfoMap,
    redis: &RedisClient,
) {
    let error_msg = LobbyServerMessage::Error {
        message: message.into(),
    };
    send_to_player(player_id, lobby_id, connection_info, &error_msg, redis).await;
}

pub async fn send_to_player(
    player_id: Uuid,
    lobby_id: Uuid,
    connection_info: &ConnectionInfoMap,
    msg: &LobbyServerMessage,
    redis: &RedisClient,
) {
    let serialized = match serde_json::to_string(msg) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("Failed to serialize message: {}", e);
            return;
        }
    };

    let conns = connection_info.lock().await;
    if let Some(conn_info) = conns.get(&player_id) {
        let mut sender = conn_info.sender.lock().await;
        if let Err(e) = sender.send(Message::Text(serialized.clone().into())).await {
            tracing::debug!("Failed to send message to player {}: {}", player_id, e);

            // Only queue the message if it should be queued
            if msg.should_queue() {
                drop(sender);
                drop(conns);

                if let Err(queue_err) =
                    queue_message_for_player(player_id, lobby_id, serialized, redis).await
                {
                    tracing::error!(
                        "Failed to queue message for player {}: {}",
                        player_id,
                        queue_err
                    );
                }
            }
        }
    } else {
        // Player not connected, only queue if message should be queued
        if msg.should_queue() {
            if let Err(e) = queue_message_for_player(player_id, lobby_id, serialized, redis).await {
                tracing::error!(
                    "Failed to queue message for offline player {}: {}",
                    player_id,
                    e
                );
            }
        }
    }
}

pub async fn request_to_join(
    lobby_id: Uuid,
    user: User,
    redis: RedisClient,
) -> Result<(), AppError> {
    if let Ok(Some(existing)) = get_player_join_request(lobby_id, user.id, redis.clone()).await {
        if existing.state == JoinState::Pending {
            return Ok(()); // Already pending
        }
    }

    update_join_request(lobby_id, user, JoinState::Pending, redis).await
}

pub async fn set_join_state(
    lobby_id: Uuid,
    user: User,
    state: JoinState,
    redis: RedisClient,
) -> Result<(), AppError> {
    update_join_request(lobby_id, user, state, redis).await
}

pub async fn get_pending_players(
    lobby_id: Uuid,
    redis: RedisClient,
) -> Result<Vec<PendingJoin>, AppError> {
    let join_requests = get_lobby_join_requests(lobby_id, None, redis).await?;

    let pending_players = join_requests
        .into_iter()
        .map(|req| PendingJoin {
            user: req.user,
            state: req.state,
        })
        .collect();

    Ok(pending_players)
}

pub async fn handle_incoming_messages(
    mut receiver: impl StreamExt<Item = Result<Message, axum::Error>> + Unpin,
    lobby_id: Uuid,
    player: &Player,
    connections: &ConnectionInfoMap,
    chat_connections: &ChatConnectionInfoMap,
    redis: RedisClient,
    bot: teloxide::Bot,
) {
    while let Some(msg_result) = receiver.next().await {
        match msg_result {
            Ok(msg) => match msg {
                Message::Text(text) => {
                    if let Ok(parsed) = serde_json::from_str::<LobbyClientMessage>(&text) {
                        match parsed {
                            LobbyClientMessage::Ping { ts } => {
                                ping(ts, player, lobby_id, connections, &redis).await
                            }
                            LobbyClientMessage::LastPing { ts } => {
                                last_ping(ts, lobby_id, player, connections, &redis).await
                            }
                            LobbyClientMessage::JoinLobby { tx_id } => {
                                join_lobby(
                                    tx_id,
                                    lobby_id,
                                    player,
                                    connections,
                                    chat_connections,
                                    &redis,
                                )
                                .await
                            }
                            LobbyClientMessage::RequestJoin => {
                                request_join(player, lobby_id, connections, &redis).await
                            }
                            LobbyClientMessage::RequestLeave => {
                                request_leave(player, lobby_id, connections, &redis).await
                            }
                            LobbyClientMessage::PermitJoin { user_id, allow } => {
                                permit_join(
                                    allow,
                                    user_id,
                                    player.clone(),
                                    lobby_id,
                                    connections,
                                    &redis,
                                )
                                .await
                            }
                            LobbyClientMessage::UpdatePlayerState { new_state } => {
                                update_player_state(
                                    new_state,
                                    lobby_id,
                                    player,
                                    connections,
                                    chat_connections,
                                    &redis,
                                )
                                .await
                            }
                            LobbyClientMessage::LeaveLobby => {
                                leave_lobby(
                                    lobby_id,
                                    player,
                                    connections,
                                    chat_connections,
                                    &redis,
                                    bot.clone(),
                                )
                                .await
                            }
                            LobbyClientMessage::KickPlayer { player_id } => {
                                kick_player(
                                    player_id,
                                    lobby_id,
                                    player,
                                    connections,
                                    chat_connections,
                                    &redis,
                                    bot.clone(),
                                )
                                .await
                            }
                            LobbyClientMessage::UpdateLobbyState { new_state } => {
                                update_game_state(
                                    new_state,
                                    lobby_id,
                                    player,
                                    connections,
                                    &redis,
                                    &bot,
                                )
                                .await
                            }
                        }
                    } else {
                        tracing::debug!("uncaught message: {text}");
                    }
                }
                Message::Ping(data) => {
                    // Handle WebSocket ping with custom timestamp logic
                    if data.len() == 8 {
                        // Extract timestamp from ping data (8 bytes for u64)
                        let mut ts_bytes = [0u8; 8];
                        if data.len() >= 8 {
                            ts_bytes.copy_from_slice(&data[..8]);
                        }
                        let client_ts = u64::from_le_bytes(ts_bytes);

                        let now = Utc::now().timestamp_millis() as u64;
                        let pong_time = now.saturating_sub(client_ts);

                        // Send as LobbyServerMessage::Pong JSON instead of WebSocket pong
                        let pong_msg = LobbyServerMessage::Pong {
                            ts: client_ts,
                            pong: pong_time,
                        };
                        send_to_player(player.id, lobby_id, &connections, &pong_msg, &redis).await;
                    } else {
                        // For standard WebSocket pings without timestamp, use current time
                        let now = Utc::now().timestamp_millis() as u64;
                        let pong_msg = LobbyServerMessage::Pong { ts: now, pong: 0 };
                        send_to_player(player.id, lobby_id, &connections, &pong_msg, &redis).await;
                    }
                }
                Message::Pong(_) => {
                    // Client responded to our ping (if we were sending any)
                    tracing::debug!("Received pong from player {}", player.id);
                }
                Message::Close(_) => {
                    tracing::debug!("Player {} closed connection", player.id);
                    break;
                }
                _ => {}
            },
            Err(e) => {
                tracing::debug!("WebSocket error for player {}: {}", player.id, e);
                break;
            }
        }
    }
}
