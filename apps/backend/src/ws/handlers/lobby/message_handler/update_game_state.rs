use crate::{
    db::lobby::{
        countdown::{clear_lobby_countdown, set_lobby_countdown},
        get::{get_lobby_info, get_lobby_players},
        patch::{leave_lobby, update_lobby_state},
    },
    models::{
        game::{LobbyState, Player, PlayerState},
        lobby::LobbyServerMessage,
    },
    state::{ConnectionInfoMap, RedisClient},
    ws::handlers::{
        lobby::message_handler::{broadcast_to_lobby, handler::send_error_to_player},
        utils::remove_connection,
    },
};
use axum::extract::ws::{CloseFrame, Message};
use futures::SinkExt;
use uuid::Uuid;

pub async fn update_game_state(
    new_state: LobbyState,
    lobby_id: Uuid,
    player: &Player,
    connections: &ConnectionInfoMap,
    redis: &RedisClient,
    bot: &teloxide::Bot,
) {
    let lobby_info = match get_lobby_info(lobby_id, redis.clone()).await {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("Failed to fetch lobby info: {}", e);
            send_error_to_player(player.id, lobby_id, e.to_string(), &connections, &redis).await;
            return;
        }
    };

    if lobby_info.creator.id != player.id {
        tracing::warn!("Unauthorized game state update attempt by {}", player.id);
        send_error_to_player(
            player.id,
            lobby_id,
            "Only creator can update game state",
            &connections,
            &redis,
        )
        .await;
        return;
    }

    if let Err(e) = update_lobby_state(lobby_id, new_state.clone(), redis.clone()).await {
        tracing::error!("Failed to update game state: {}", e);
        send_error_to_player(player.id, lobby_id, e.to_string(), &connections, &redis).await;
    } else {
        tracing::info!(
            "Lobby {} state updated to {:?} by player {}",
            lobby_id,
            new_state,
            player.id
        );
        let game_starting = LobbyServerMessage::LobbyState {
            state: new_state.clone(),
            joined_players: None,
            started: false,
        };
        broadcast_to_lobby(lobby_id, &game_starting, &connections, None, redis.clone()).await;
        if new_state == LobbyState::Starting {
            let redis_clone = redis.clone();
            let conns_clone = connections.clone();
            let player_clone = player.clone();
            let bot_clone = bot.clone();
            tokio::spawn(async move {
                start_countdown(lobby_id, player_clone, redis_clone, conns_clone, bot_clone).await;
            });
        } else {
            // If game state is not starting, clear any existing countdown
            if let Err(e) = clear_lobby_countdown(lobby_id, redis.clone()).await {
                tracing::error!("Failed to clear countdown for lobby {}: {}", lobby_id, e);
            }
        }
    }
}

async fn close_lobby_connections(
    lobby_id: Uuid,
    player_ids: &[Uuid],
    connections: &ConnectionInfoMap,
    redis: &RedisClient,
    bot: &teloxide::Bot,
) {
    let connections_guard = connections.lock().await;

    let mut target_connections = Vec::new();
    for &player_id in player_ids {
        if let Some(connection_info) = connections_guard.get(&player_id) {
            target_connections.push((player_id, connection_info.clone()));
        }
    }

    drop(connections_guard);

    let idle_players =
        match get_lobby_players(lobby_id, Some(PlayerState::NotJoined), redis.clone()).await {
            Ok(players) => players,
            Err(e) => {
                tracing::error!("Failed to get idle players for cleanup: {}", e);
                vec![]
            }
        };

    for (player_id, connection_info) in target_connections {
        {
            let mut sender = connection_info.sender.lock().await;
            tracing::info!(
                "Closing lobby connection for player {} (game starting)",
                player_id
            );

            let close_frame = CloseFrame {
                code: axum::extract::ws::close_code::NORMAL,
                reason: "Game starting - redirecting to game".into(),
            };

            let _ = sender.send(Message::Close(Some(close_frame))).await;
        }
    }

    // Remove all idle players from the lobby when game starts
    for idle_player in idle_players {
        if let Err(e) = leave_lobby(lobby_id, idle_player.id, redis.clone(), bot.clone()).await {
            tracing::error!(
                "Failed to remove idle player {} from lobby {}: {}",
                idle_player.id,
                lobby_id,
                e
            );
        } else {
            tracing::info!(
                "Removed idle player {} from lobby {} (game starting)",
                idle_player.id,
                lobby_id
            );
        }
    }
}

async fn start_countdown(
    lobby_id: Uuid,
    player: Player,
    redis: RedisClient,
    connections: ConnectionInfoMap,
    bot: teloxide::Bot,
) {
    // Initialize countdown state in Redis
    if let Err(e) = set_lobby_countdown(lobby_id, 15, redis.clone()).await {
        tracing::error!("Failed to set countdown for lobby {}: {}", lobby_id, e);
        return;
    }

    for i in (0..=15).rev() {
        // Update countdown state in Redis
        if let Err(e) = set_lobby_countdown(lobby_id, i, redis.clone()).await {
            tracing::error!("Failed to update countdown for lobby {}: {}", lobby_id, e);
            break;
        }

        match get_lobby_info(lobby_id, redis.clone()).await {
            Ok(info) => {
                if info.state != LobbyState::Starting {
                    tracing::info!("Countdown interrupted by state change");

                    // Clear countdown state
                    if let Err(e) = clear_lobby_countdown(lobby_id, redis.clone()).await {
                        tracing::error!("Failed to clear countdown for lobby {}: {}", lobby_id, e);
                    }

                    let msg = LobbyServerMessage::LobbyState {
                        state: info.state,
                        joined_players: None,
                        started: false,
                    };
                    broadcast_to_lobby(lobby_id, &msg, &connections, None, redis.clone()).await;

                    break;
                }
            }
            Err(e) => {
                tracing::error!("Failed to check state: {}", e);
                send_error_to_player(player.id, lobby_id, e.to_string(), &connections, &redis)
                    .await;

                // Clear countdown state on error
                if let Err(e) = clear_lobby_countdown(lobby_id, redis.clone()).await {
                    tracing::error!("Failed to clear countdown for lobby {}: {}", lobby_id, e);
                }
                break;
            }
        }

        let countdown_msg = LobbyServerMessage::Countdown { time: i };
        broadcast_to_lobby(lobby_id, &countdown_msg, &connections, None, redis.clone()).await;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    // Final state confirmation
    if let Ok(info) = get_lobby_info(lobby_id, redis.clone()).await {
        if info.state == LobbyState::Starting {
            let joined_players =
                match get_lobby_players(lobby_id, Some(PlayerState::Joined), redis.clone()).await {
                    Ok(players) => players.into_iter().map(|p| p.id).collect::<Vec<_>>(),
                    Err(e) => {
                        tracing::error!("❌ Failed to get ready players: {}", e);
                        send_error_to_player(
                            player.id,
                            lobby_id,
                            e.to_string(),
                            &connections,
                            &redis,
                        )
                        .await;
                        vec![]
                    }
                };

            let players =
                match get_lobby_players(lobby_id, Some(PlayerState::Joined), redis.clone()).await {
                    Ok(players) => players,
                    Err(e) => {
                        tracing::error!("❌ Failed to get lobby players: {}", e);
                        send_error_to_player(
                            player.id,
                            lobby_id,
                            e.to_string(),
                            &connections,
                            &redis,
                        )
                        .await;
                        vec![]
                    }
                };

            tracing::info!("Game started with {} ready players", joined_players.len());

            // Update lobby state to InProgress after countdown completes
            if let Err(e) =
                update_lobby_state(lobby_id, LobbyState::InProgress, redis.clone()).await
            {
                tracing::error!("Failed to update lobby state to InProgress: {}", e);
                send_error_to_player(player.id, lobby_id, e.to_string(), &connections, &redis)
                    .await;
                return;
            }

            let msg = LobbyServerMessage::LobbyState {
                state: LobbyState::InProgress,
                joined_players: Some(joined_players.clone()),
                started: false,
            };
            broadcast_to_lobby(lobby_id, &msg, &connections, None, redis.clone()).await;

            // Clear countdown state since game has officially started
            if let Err(e) = clear_lobby_countdown(lobby_id, redis.clone()).await {
                tracing::error!("Failed to clear countdown for lobby {}: {}", lobby_id, e);
            }

            if joined_players.len() > 1 {
                // Get all player IDs from the lobby for disconnection
                let all_player_ids: Vec<Uuid> = players.iter().map(|p| p.id).collect();

                // Remove connections from state
                for player in &players {
                    remove_connection(player.id, &connections).await;
                }

                // Close WebSocket connections with proper close frame
                close_lobby_connections(lobby_id, &all_player_ids, &connections, &redis, &bot)
                    .await;
            }
        }
    }
}
