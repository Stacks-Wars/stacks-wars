use axum::{
    extract::{ConnectInfo, Path, Query, State, WebSocketUpgrade, ws::WebSocket},
    http::StatusCode,
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt, stream::SplitStream};
use std::net::SocketAddr;
use uuid::Uuid;

use crate::{
    db::{
        game::state::{
            get_current_rule, get_current_turn, get_game_started, get_rule_context,
            set_current_turn, set_rule_context, set_rule_index,
        },
        lobby::{
            get::{get_connected_players_ids, get_lobby_info, get_lobby_players},
            patch::{
                add_connected_player, add_spectator, remove_connected_player, remove_spectator,
            },
        },
    },
    errors::AppError,
    games::lexi_wars::{
        self,
        engine::start_auto_start_timer,
        rules::RuleContext,
        utils::{broadcast_to_player, generate_random_letter},
    },
    models::{
        game::{ClaimState, LobbyInfo, LobbyState, Player, PlayerState, WsQueryParams},
        lexi_wars::{LexiWarsServerMessage, PlayerStanding},
    },
    state::{AppState, ConnectionInfoMap, RedisClient},
    ws::handlers::utils::{remove_connection, store_connection_and_send_queued_messages},
};

pub async fn lexi_wars_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQueryParams>,
    Path(lobby_id): Path<Uuid>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::debug!("New Lexi-Wars WebSocket connection from {}", addr);

    let player_id = query.user_id;
    let redis = state.redis.clone();
    let connections = state.connections.clone();
    let bot = state.bot.clone();

    let lobby = get_lobby_info(lobby_id, redis.clone())
        .await
        .map_err(|e| e.to_response())?;

    // Check lobby state
    if lobby.state != LobbyState::InProgress {
        if lobby.state == LobbyState::Finished {
            tracing::info!("Player {} trying to connect to finished game", player_id);

            // Send game over info and close connection
            return Ok(ws.on_upgrade(move |mut socket| async move {
                // Send GameOver message first
                let game_over_msg = LexiWarsServerMessage::GameOver;
                let serialized = serde_json::to_string(&game_over_msg).unwrap();
                let _ = socket
                    .send(axum::extract::ws::Message::Text(serialized.into()))
                    .await;

                // Get lobby data for final standing
                if let Ok(players) =
                    get_lobby_players(lobby_id, Some(PlayerState::Joined), redis.clone()).await
                {
                    // Create final standing from players (they should have ranks)
                    let mut players_with_ranks: Vec<_> =
                        players.into_iter().filter(|p| p.rank.is_some()).collect();

                    // Sort by rank (1st place first)
                    players_with_ranks.sort_by_key(|p| p.rank.unwrap());

                    let standing: Vec<PlayerStanding> = players_with_ranks
                        .clone()
                        .into_iter()
                        .map(|player| PlayerStanding {
                            rank: player.rank.unwrap(),
                            player,
                        })
                        .collect();

                    let final_standing_msg = LexiWarsServerMessage::FinalStanding { standing };
                    let serialized = serde_json::to_string(&final_standing_msg).unwrap();
                    let _ = socket
                        .send(axum::extract::ws::Message::Text(serialized.into()))
                        .await;

                    // Send prize message if player has one AND hasn't claimed it yet
                    if let Some(connecting_player) =
                        players_with_ranks.iter().find(|p| p.id == player_id)
                    {
                        if let Some(prize_amount) = connecting_player.prize {
                            // Check if player has not claimed the prize
                            let should_send_prize = match &connecting_player.claim {
                                Some(ClaimState::NotClaimed) => true,
                                None => false,
                                Some(ClaimState::Claimed { .. }) => false,
                            };

                            if should_send_prize {
                                let prize_msg = LexiWarsServerMessage::Prize {
                                    amount: prize_amount,
                                };
                                let serialized = serde_json::to_string(&prize_msg).unwrap();
                                let _ = socket
                                    .send(axum::extract::ws::Message::Text(serialized.into()))
                                    .await;
                            } else {
                                let prize_msg = LexiWarsServerMessage::Prize { amount: 0.0 };
                                let serialized = serde_json::to_string(&prize_msg).unwrap();
                                let _ = socket
                                    .send(axum::extract::ws::Message::Text(serialized.into()))
                                    .await;
                            }
                        }

                        if let Some(rank) = connecting_player.rank {
                            let rank_msg = LexiWarsServerMessage::Rank {
                                rank: rank.to_string(),
                            };
                            let serialized = serde_json::to_string(&rank_msg).unwrap();
                            let _ = socket
                                .send(axum::extract::ws::Message::Text(serialized.into()))
                                .await;
                        }
                    }
                }

                let _ = socket.close().await;
            }));
        } else if lobby.state == LobbyState::Starting {
            tracing::debug!("Player {} trying to connect to starting lobby", player_id);

            // Send StartFailed message and close connection
            return Ok(ws.on_upgrade(move |mut socket| async move {
                let start_failed_msg = LexiWarsServerMessage::StartFailed;
                let serialized = serde_json::to_string(&start_failed_msg).unwrap();
                let _ = socket
                    .send(axum::extract::ws::Message::Text(serialized.into()))
                    .await;
                let _ = socket.close().await;
            }));
        } else if lobby.state == LobbyState::Waiting {
            tracing::debug!("Player {} trying to connect to waiting lobby", player_id);

            // Send StartFailed message and close connection
            return Ok(ws.on_upgrade(move |mut socket| async move {
                let start_failed_msg = LexiWarsServerMessage::StartFailed;
                let serialized = serde_json::to_string(&start_failed_msg).unwrap();
                let _ = socket
                    .send(axum::extract::ws::Message::Text(serialized.into()))
                    .await;
                let _ = socket.close().await;
            }));
        } else {
            tracing::error!("lobby {} has unexpected state: {:?}", lobby_id, lobby.state);
            return Err(AppError::BadRequest("lobby has unexpected state".into()).to_response());
        }
    }

    let players = get_lobby_players(lobby_id, Some(PlayerState::Joined), redis.clone())
        .await
        .map_err(|e| e.to_response())?;

    let players_clone = players.clone();

    let matched_player = players.into_iter().find(|p| p.id == player_id);

    let is_game_started = get_game_started(lobby_id, redis.clone())
        .await
        .map_err(|e| e.to_response())?;

    let connected_player_ids = get_connected_players_ids(lobby_id, redis.clone())
        .await
        .map_err(|e| e.to_response())?;

    // Handle different connection scenarios
    match (matched_player, is_game_started) {
        // Case 1: Player is a lobby member
        (Some(player), game_started) => {
            let is_reconnecting = connected_player_ids.contains(&player_id);

            if game_started && !is_reconnecting {
                // Lobby member connecting to started game for first time -> spectator
                tracing::info!(
                    "Lobby member {} joining started game {} as spectator (first connection)",
                    player.id,
                    lobby_id
                );

                Ok(ws.on_upgrade(move |socket| {
                    let lobby_info = lobby.clone();
                    handle_lexi_wars_socket(
                        socket,
                        lobby_id,
                        player_id,
                        None, // Pass None to make them a spectator
                        players_clone,
                        connected_player_ids,
                        connections,
                        lobby_info,
                        redis,
                        is_game_started,
                        bot.clone(),
                    )
                }))
            } else {
                // Either game hasn't started or player is reconnecting -> normal player
                if is_reconnecting {
                    tracing::info!("Player {} reconnecting to started game", player_id);
                } else {
                    tracing::info!(
                        "Player {} allowed to join lexi wars {} as participant",
                        player.id,
                        lobby_id
                    );
                }

                Ok(ws.on_upgrade(move |socket| {
                    let lobby_info = lobby.clone();
                    handle_lexi_wars_socket(
                        socket,
                        lobby_id,
                        player_id,
                        Some(player),
                        players_clone,
                        connected_player_ids,
                        connections,
                        lobby_info,
                        redis,
                        is_game_started,
                        bot.clone(),
                    )
                }))
            }
        }
        // Case 2: Not a lobby member, but game has started - add as spectator
        (None, true) => {
            tracing::info!("User {} joining lobby {} as spectator", player_id, lobby_id);

            Ok(ws.on_upgrade(move |socket| {
                let lobby_info = lobby.clone();
                handle_lexi_wars_socket(
                    socket,
                    lobby_id,
                    player_id,
                    None,
                    players_clone,
                    connected_player_ids,
                    connections,
                    lobby_info,
                    redis,
                    is_game_started,
                    bot.clone(),
                )
            }))
        }
        // Case 3: Not a lobby member and game hasn't started - add as spectator. TODO we should probably disconnect
        (None, false) => {
            tracing::info!(
                "User {} trying to join lobby {} but is not a member and game hasn't started",
                player_id,
                lobby_id
            );
            Ok(ws.on_upgrade(move |socket| {
                let lobby_info = lobby.clone();
                handle_lexi_wars_socket(
                    socket,
                    lobby_id,
                    player_id,
                    None,
                    players_clone,
                    connected_player_ids,
                    connections,
                    lobby_info,
                    redis,
                    is_game_started,
                    bot.clone(),
                )
            }))
        }
    }
}

async fn handle_lexi_wars_socket(
    socket: WebSocket,
    lobby_id: Uuid,
    user_id: Uuid,
    player: Option<Player>,
    players: Vec<Player>,
    connected_player_ids: Vec<Uuid>,
    connections: ConnectionInfoMap,
    lobby_info: LobbyInfo,
    redis: RedisClient,
    game_started: bool,
    bot: teloxide::Bot,
) {
    let (sender, receiver) = socket.split();

    // Handle connection setup differently for players vs spectators
    if let Some(ref p) = player {
        // This is a lobby participant (player)
        store_connection_and_send_queued_messages(p.id, lobby_id, sender, &connections, &redis)
            .await;

        let start_msg = LexiWarsServerMessage::Start {
            time: if game_started { 0 } else { 15 },
            started: game_started,
        };
        broadcast_to_player(p.id, lobby_id, &start_msg, &connections, &redis).await;

        setup_player_and_lobby(
            p,
            lobby_info.clone(),
            players.clone(),
            connected_player_ids.clone(),
            game_started,
            &connections,
            &redis,
            &bot,
        )
        .await;

        // Handle player reconnection state
        if game_started {
            // Send current turn if available
            if let Ok(Some(current_turn_id)) = get_current_turn(lobby_id, redis.clone()).await {
                if let Some(current_player) = players.iter().find(|gp| gp.id == current_turn_id) {
                    let turn_msg = LexiWarsServerMessage::Turn {
                        current_turn: current_player.clone(),
                        countdown: 15, // Default countdown for reconnection
                    };
                    broadcast_to_player(p.id, lobby_id, &turn_msg, &connections, &redis).await;
                }
            }

            // Send current rule if available AND this player is the current player
            if let Ok(Some(current_turn_id)) = get_current_turn(lobby_id, redis.clone()).await {
                if current_turn_id == p.id {
                    if let Ok(Some(current_rule)) = get_current_rule(lobby_id, redis.clone()).await
                    {
                        let rule_msg = LexiWarsServerMessage::Rule { rule: current_rule };
                        broadcast_to_player(p.id, lobby_id, &rule_msg, &connections, &redis).await;
                    }
                }
            }

            tracing::debug!("Sent reconnection state to player {}", p.id);
        }

        // Handle incoming messages as a player
        lexi_wars::engine::handle_incoming_messages(
            p,
            lobby_id,
            receiver,
            &connections,
            redis.clone(),
            bot.clone(),
        )
        .await;

        // Handle player disconnection
        let game_started = get_game_started(lobby_id, redis.clone())
            .await
            .unwrap_or(false);
        if !game_started {
            // If game hasn't started, remove from connected players
            if let Err(e) = remove_connected_player(lobby_id, p.id, redis.clone()).await {
                tracing::error!("Failed to remove disconnected player: {}", e);
            }

            tracing::info!(
                "Player {} disconnected from lobby {} (pre-game)",
                p.id,
                lobby_id
            );
        } else {
            tracing::info!(
                "Player {} disconnected from lobby {} (during game). Keeping in connected_player_ids for game continuity.",
                p.id,
                lobby_id
            );
        }

        remove_connection(p.id, &connections).await;
    } else {
        // This is a spectator - use the provided user_id
        let spectator_id = user_id;

        // Add as spectator
        if let Err(e) = add_spectator(lobby_id, spectator_id, redis.clone()).await {
            tracing::error!("Failed to add spectator: {}", e);
        }

        // Store connection for spectator
        store_connection_and_send_queued_messages(
            spectator_id,
            lobby_id,
            sender,
            &connections,
            &redis,
        )
        .await;

        let start_msg = LexiWarsServerMessage::Start {
            time: if game_started { 0 } else { 15 },
            started: game_started,
        };
        broadcast_to_player(spectator_id, lobby_id, &start_msg, &connections, &redis).await;

        // Send spectator message to indicate their status
        let spectator_msg = LexiWarsServerMessage::Spectator;
        broadcast_to_player(spectator_id, lobby_id, &spectator_msg, &connections, &redis).await;

        // Send current game state to spectator
        if game_started {
            // Send current turn info
            if let Ok(Some(current_turn_id)) = get_current_turn(lobby_id, redis.clone()).await {
                if let Some(current_player) = players.iter().find(|gp| gp.id == current_turn_id) {
                    let turn_msg = LexiWarsServerMessage::Turn {
                        current_turn: current_player.clone(),
                        countdown: 15, // Default countdown for spectator
                    };
                    broadcast_to_player(spectator_id, lobby_id, &turn_msg, &connections, &redis)
                        .await;
                }
            }
            // Spectators can see rules too
            if let Ok(Some(current_rule)) = get_current_rule(lobby_id, redis.clone()).await {
                let rule_msg = LexiWarsServerMessage::Rule { rule: current_rule };
                broadcast_to_player(spectator_id, lobby_id, &rule_msg, &connections, &redis).await;
            }
        }

        // Handle spectator messages (they can only receive, not send game messages)
        if let Err(e) =
            handle_spectator_messages(spectator_id, lobby_id, receiver, &connections, &redis).await
        {
            tracing::error!("Error handling spectator messages: {}", e);
        }

        // Handle spectator disconnection
        if let Err(e) = remove_spectator(lobby_id, spectator_id, redis.clone()).await {
            tracing::error!("Failed to remove spectator: {}", e);
        }

        remove_connection(spectator_id, &connections).await;

        tracing::info!(
            "Spectator {} disconnected from lobby {}",
            spectator_id,
            lobby_id
        );
    }
}

async fn handle_spectator_messages(
    spectator_id: Uuid,
    _lobby_id: Uuid,
    mut receiver: SplitStream<WebSocket>,
    _connections: &ConnectionInfoMap,
    _redis: &RedisClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Spectators can only receive messages, they just need to stay connected
    // We'll consume the receiver to keep the connection alive but ignore most messages
    while let Some(msg_result) = receiver.next().await {
        match msg_result {
            Ok(msg) => match msg {
                axum::extract::ws::Message::Close(_) => {
                    tracing::debug!("WebSocket close from spectator {}", spectator_id);
                    break;
                }
                axum::extract::ws::Message::Ping(_) => {
                    tracing::debug!("WebSocket ping from spectator {}", spectator_id);
                }
                axum::extract::ws::Message::Pong(_) => {
                    tracing::debug!("WebSocket pong from spectator {}", spectator_id);
                }
                _ => {
                    // Spectators can't send game messages, but we'll just ignore them
                }
            },
            Err(e) => {
                tracing::debug!("WebSocket error for spectator {}: {}", spectator_id, e);
                break;
            }
        }
    }

    Ok(())
}

async fn setup_player_and_lobby(
    player: &Player,
    lobby_info: LobbyInfo,
    players: Vec<Player>,
    connected_player_ids: Vec<Uuid>,
    game_started: bool,
    connections: &ConnectionInfoMap,
    redis: &RedisClient,
    telegram_bot: &teloxide::Bot,
) {
    let lobby_id = lobby_info.id;

    // Initialize game state if not exists
    if !game_started {
        // Initialize rule context if not set
        if get_rule_context(lobby_id, redis.clone())
            .await
            .unwrap_or(None)
            .is_none()
        {
            let rule_context = RuleContext {
                min_word_length: 4,
                random_letter: generate_random_letter(),
            };
            let _ = set_rule_context(lobby_id, &rule_context, redis.clone()).await;
            let _ = set_rule_index(lobby_id, 0, redis.clone()).await;
        }

        // Set initial current turn if not set
        if get_current_turn(lobby_id, redis.clone())
            .await
            .unwrap_or(None)
            .is_none()
        {
            let _ = set_current_turn(lobby_id, lobby_info.creator.id, redis.clone()).await;
        }
    }

    // Track connected player by adding to Redis connected players set
    if !connected_player_ids.contains(&player.id) {
        // Verify the player is actually part of the lobby before adding to connected players
        if players.iter().any(|p| p.id == player.id) {
            if let Err(e) = add_connected_player(lobby_id, player.id, redis.clone()).await {
                tracing::error!("Failed to add connected player: {}", e);
            }
        } else {
            tracing::warn!(
                "Player {} not found in lobby {} players list, skipping connected player tracking",
                player.id,
                lobby_id
            );
        }
    }

    // Get updated count for logging and auto-start check
    let updated_connected_count = connected_player_ids.len()
        + if connected_player_ids.contains(&player.id) {
            0
        } else {
            1
        };

    tracing::info!(
        "Player {} connected to lobby {}. Connected: {}/{}",
        player.id,
        lobby_id,
        updated_connected_count,
        players.len()
    );

    // Start auto-start timer when first player connects and game hasn't started
    if updated_connected_count == 1 && !game_started {
        tracing::info!(
            "First player connected, starting auto-start timer for lobby {}",
            lobby_id
        );
        start_auto_start_timer(
            lobby_id,
            connections.clone(),
            redis.clone(),
            telegram_bot.clone(),
        );
    }
}
