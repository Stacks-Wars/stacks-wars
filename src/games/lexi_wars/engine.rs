use axum::extract::ws::Message;
use chrono::Utc;
use futures::StreamExt;
use std::time::Duration;
use tokio::time::sleep;

use crate::{
    db::{
        game::{
            player_words::add_player_used_word,
            state::{
                add_eliminated_player, clear_lobby_game_state, get_current_turn,
                get_eliminated_players, get_rule_context, get_rule_index, set_current_rule,
                set_current_turn, set_game_started, set_rule_context, set_rule_index,
            },
            words::{add_used_word, is_valid_word, is_word_used_in_lobby},
        },
        leaderboard::patch::update_user_stats,
        lobby::{
            get::{
                get_connected_players_ids, get_current_players_ids, get_lobby_info,
                get_lobby_players,
            },
            patch::{add_spectator, update_lobby_state},
            put::{create_current_players, remove_current_player},
        },
    },
    games::lexi_wars::{
        rules::{RuleContext, get_rule_by_index, get_rules},
        utils::{
            broadcast_to_lobby_and_spectators, broadcast_to_player,
            broadcast_to_player_and_spectators, generate_random_letter,
        },
    },
    http::bot::{self, BotLobbyWinnerPayload, RunnerUp},
    models::{
        game::{LobbyInfo, LobbyState, Player, PlayerState},
        lexi_wars::{LexiWarsClientMessage, LexiWarsServerMessage, PlayerStanding},
    },
    state::{ConnectionInfoMap, RedisClient},
};
use teloxide::Bot;
use uuid::Uuid;

#[derive(Clone)]
struct GameContext {
    rule_context: RuleContext,
    rule_index: usize,
}

async fn validate_word(
    lobby_id: Uuid,
    word: &str,
    redis: &RedisClient,
    cached_game_context: Option<&GameContext>,
) -> Result<(GameContext, bool), Box<dyn std::error::Error + Send + Sync>> {
    let cleaned_word = word.trim().to_lowercase();

    // Get game context (use cache if available and valid)
    let game_context = if let Some(cached) = cached_game_context {
        cached.clone()
    } else {
        let (rule_context_result, rule_index_result) = tokio::join!(
            get_rule_context(lobby_id, redis.clone()),
            get_rule_index(lobby_id, redis.clone())
        );

        let rule_context = rule_context_result?.ok_or("No rule context found")?;
        let rule_index = rule_index_result?.ok_or("No rule index found")?;

        GameContext {
            rule_context,
            rule_index,
        }
    };

    let (used_in_lobby_result, valid_word_result) = tokio::join!(
        is_word_used_in_lobby(lobby_id, &cleaned_word, redis.clone()),
        is_valid_word(&cleaned_word, redis.clone())
    );

    if used_in_lobby_result? {
        return Ok((game_context, false));
    }

    if !valid_word_result? {
        return Ok((game_context, false));
    }

    // Apply rule validation
    if let Some(rule) = get_rule_by_index(game_context.rule_index, &game_context.rule_context) {
        // Check minimum word length first (unless it's the min_length rule itself)
        if rule.name != "min_length" {
            if cleaned_word.len() < game_context.rule_context.min_word_length {
                return Ok((game_context, false));
            }
        }

        // Validate word against rule
        if (rule.validate)(&cleaned_word, &game_context.rule_context).is_err() {
            return Ok((game_context, false));
        }
    } else {
        return Err("Invalid rule index".into());
    }

    Ok((game_context, true))
}

fn get_prize(
    lobby_info: &crate::models::game::LobbyInfo,
    connected_players_count: usize,
    position: usize,
) -> Option<f64> {
    if lobby_info.contract_address.is_none() {
        return None;
    }

    let entry_amount = lobby_info.entry_amount.unwrap_or(0.0);
    let current_amount = lobby_info.current_amount.unwrap_or(0.0);

    // Calculate total pool based on lobby type
    let total_pool = if entry_amount == 0.0 {
        // Sponsored lobby - use current_amount as the pre-funded pool
        current_amount
    } else {
        // Regular paid lobby - calculate from entry amount * connected players
        entry_amount * connected_players_count as f64
    };

    // No prizes if there's no pool
    if total_pool <= 0.0 {
        return None;
    }

    let prize = match position {
        1 => {
            if connected_players_count == 2 {
                (total_pool * 70.0) / 100.0
            } else {
                (total_pool * 50.0) / 100.0
            }
        }
        2 => (total_pool * 30.0) / 100.0,
        3 => (total_pool * 20.0) / 100.0,
        _ => 0.0,
    };

    Some(prize)
}

fn calculate_wars_point(
    lobby_info: &crate::models::game::LobbyInfo,
    connected_players_count: usize,
    rank: usize,
    prize: Option<f64>,
    player_id: Uuid,
) -> f64 {
    let base_point = (connected_players_count - rank + 1) * 2;
    let mut total_point = base_point as f64;

    // Add pool bonus if there's a pool (prize and entry amount exist)
    if let (Some(prize_amount), Some(entry_amount)) = (prize, lobby_info.entry_amount) {
        let pool_bonus = if entry_amount != 0.0 {
            (prize_amount / connected_players_count as f64) + (entry_amount / 5.0)
        } else {
            0.0
        };
        total_point += pool_bonus;
    }

    // Add sponsor bonus if this is a sponsored lobby and the player is the sponsor (creator)
    if let (Some(entry_amount), Some(current_amount)) =
        (lobby_info.entry_amount, lobby_info.current_amount)
    {
        if entry_amount == 0.0 && current_amount > 0.0 && player_id == lobby_info.creator.id {
            let sponsor_bonus = 2.5 * connected_players_count as f64;
            total_point += sponsor_bonus;
        }
    }

    // Cap at 50 points maximum
    total_point.min(50.0)
}

async fn send_rank_prize_and_wars_point(
    player_id: Uuid,
    lobby_id: Uuid,
    lobby_info: &crate::models::game::LobbyInfo,
    connected_players_count: usize,
    rank: usize,
    connections: &ConnectionInfoMap,
    redis: &RedisClient,
) {
    let prize = get_prize(lobby_info, connected_players_count, rank);
    let wars_point =
        calculate_wars_point(lobby_info, connected_players_count, rank, prize, player_id);

    // Send rank message
    let rank_msg = LexiWarsServerMessage::Rank {
        rank: rank.to_string(),
    };
    broadcast_to_player(player_id, lobby_id, &rank_msg, connections, redis).await;

    // Send prize if applicable
    if let Some(amount) = prize {
        let prize_msg = LexiWarsServerMessage::Prize { amount };
        broadcast_to_player(player_id, lobby_id, &prize_msg, connections, redis).await;
    }

    // Send wars point message
    let wars_point_msg = LexiWarsServerMessage::WarsPoint { wars_point };
    broadcast_to_player(player_id, lobby_id, &wars_point_msg, connections, redis).await;

    // Update user stats
    match update_user_stats(player_id, lobby_id, rank, prize, wars_point, redis.clone()).await {
        Ok(()) => {
            tracing::info!(
                "Player {} earned {} wars points (rank: {}, prize: {:?})",
                player_id,
                wars_point,
                rank,
                prize
            );
        }
        Err(e) => {
            tracing::error!(
                "Failed to update user stats for player {}: {}",
                player_id,
                e
            );
        }
    }
}

pub async fn handle_incoming_messages(
    player: &Player,
    lobby_id: Uuid,
    mut receiver: impl StreamExt<Item = Result<Message, axum::Error>> + Unpin,
    connections: &ConnectionInfoMap,
    redis: RedisClient,
    _telegram_bot: Bot,
) {
    let _telegram_bot_clone = _telegram_bot.clone(); // Clone at function level for use in nested scopes
    while let Some(msg_result) = receiver.next().await {
        match msg_result {
            Ok(msg) => match msg {
                Message::Text(text) => {
                    let parsed = match serde_json::from_str::<LexiWarsClientMessage>(&text) {
                        Ok(msg) => msg,
                        Err(e) => {
                            tracing::info!("Invalid message format from {}: {}", player.id, e);
                            continue;
                        }
                    };

                    match parsed {
                        LexiWarsClientMessage::Ping { ts } => {
                            let now = Utc::now().timestamp_millis() as u64;
                            let pong = now.saturating_sub(ts);
                            let pong_msg = LexiWarsServerMessage::Pong { ts, pong };
                            broadcast_to_player(
                                player.id,
                                lobby_id,
                                &pong_msg,
                                connections,
                                &redis,
                            )
                            .await;
                        }
                        LexiWarsClientMessage::WordEntry { word } => {
                            let cleaned_word = word.trim().to_lowercase();

                            // Check if it's the player's turn
                            let current_turn_id =
                                match get_current_turn(lobby_id, redis.clone()).await {
                                    Ok(Some(id)) => id,
                                    Ok(None) => {
                                        tracing::error!("No current turn set");
                                        continue;
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to get current turn: {}", e);
                                        continue;
                                    }
                                };

                            if player.id != current_turn_id {
                                tracing::info!("Not {}'s turn", player.id);
                                continue;
                            }

                            let (game_context, is_valid) = match validate_word(
                                lobby_id,
                                &cleaned_word,
                                &redis,
                                None, // Could cache this per lobby in future optimization
                            )
                            .await
                            {
                                Ok(result) => result,
                                Err(e) => {
                                    tracing::error!("Word validation error: {}", e);
                                    continue;
                                }
                            };

                            if !is_valid {
                                if is_word_used_in_lobby(lobby_id, &cleaned_word, redis.clone())
                                    .await
                                    .unwrap_or(false)
                                {
                                    let used_word_msg = LexiWarsServerMessage::UsedWord {
                                        word: cleaned_word.clone(),
                                    };
                                    broadcast_to_player(
                                        player.id,
                                        lobby_id,
                                        &used_word_msg,
                                        connections,
                                        &redis,
                                    )
                                    .await;
                                } else if !is_valid_word(&cleaned_word, redis.clone())
                                    .await
                                    .unwrap_or(false)
                                {
                                    let validation_msg = LexiWarsServerMessage::Validate {
                                        msg: "Invalid word".to_string(),
                                    };
                                    broadcast_to_player(
                                        player.id,
                                        lobby_id,
                                        &validation_msg,
                                        connections,
                                        &redis,
                                    )
                                    .await;
                                } else {
                                    // Rule validation failed
                                    if let Some(rule) = get_rule_by_index(
                                        game_context.rule_index,
                                        &game_context.rule_context,
                                    ) {
                                        if rule.name != "min_length"
                                            && cleaned_word.len()
                                                < game_context.rule_context.min_word_length
                                        {
                                            let reason = format!(
                                                "Word must be at least {} characters!",
                                                game_context.rule_context.min_word_length
                                            );
                                            let validation_msg =
                                                LexiWarsServerMessage::Validate { msg: reason };
                                            broadcast_to_player(
                                                player.id,
                                                lobby_id,
                                                &validation_msg,
                                                connections,
                                                &redis,
                                            )
                                            .await;
                                        } else if let Err(reason) = (rule.validate)(
                                            &cleaned_word,
                                            &game_context.rule_context,
                                        ) {
                                            let validation_msg =
                                                LexiWarsServerMessage::Validate { msg: reason };
                                            broadcast_to_player(
                                                player.id,
                                                lobby_id,
                                                &validation_msg,
                                                connections,
                                                &redis,
                                            )
                                            .await;
                                        }
                                    }
                                }
                                continue;
                            }

                            // Update current rule
                            if let Some(rule) = get_rule_by_index(
                                game_context.rule_index,
                                &game_context.rule_context,
                            ) {
                                if let Err(e) = set_current_rule(
                                    lobby_id,
                                    Some(rule.description.clone()),
                                    redis.clone(),
                                )
                                .await
                                {
                                    tracing::error!("Failed to set current rule: {}", e);
                                }
                            }

                            let (add_used_result, add_player_result, current_players_result) = tokio::join!(
                                add_used_word(lobby_id, &cleaned_word, redis.clone()),
                                add_player_used_word(
                                    lobby_id,
                                    player.id,
                                    &cleaned_word,
                                    redis.clone()
                                ),
                                get_current_players_ids(lobby_id, redis.clone())
                            );

                            if let Err(e) = add_used_result {
                                tracing::error!("Failed to add used word: {}", e);
                                continue;
                            }

                            if let Err(e) = add_player_result {
                                tracing::error!("Failed to add player used word: {}", e);
                            }

                            // Get current players to find next player
                            let current_players_ids = match current_players_result {
                                Ok(ids) => ids,
                                Err(e) => {
                                    tracing::error!("Failed to get current players: {}", e);
                                    continue;
                                }
                            };

                            // Find next player using current players list
                            let current_index =
                                current_players_ids.iter().position(|&id| id == player.id);
                            if let Some(index) = current_index {
                                let next_index = (index + 1) % current_players_ids.len();
                                let next_player_id = current_players_ids[next_index];

                                // Check if we wrapped back to the first player (rule progression)
                                let wrapped = next_index == 0;
                                let mut new_rule_index = game_context.rule_index;
                                let mut new_rule_context = game_context.rule_context.clone();

                                if wrapped {
                                    // We wrapped back to first player, advance rules
                                    let total_rules = get_rules(&game_context.rule_context).len();
                                    new_rule_index = (game_context.rule_index + 1) % total_rules;

                                    // If we wrapped to first rule again, increase difficulty
                                    if new_rule_index == 0 {
                                        new_rule_context.min_word_length += 2;
                                    }

                                    // Update rule context and index
                                    if let Err(e) =
                                        set_rule_context(lobby_id, &new_rule_context, redis.clone())
                                            .await
                                    {
                                        tracing::error!("Failed to update rule context: {}", e);
                                    }
                                    if let Err(e) =
                                        set_rule_index(lobby_id, new_rule_index, redis.clone())
                                            .await
                                    {
                                        tracing::error!("Failed to update rule index: {}", e);
                                    }
                                }

                                new_rule_context.random_letter = generate_random_letter();

                                if let Err(e) =
                                    set_rule_context(lobby_id, &new_rule_context, redis.clone())
                                        .await
                                {
                                    tracing::error!("Failed to update rule context: {}", e);
                                }

                                // Set next turn
                                if let Err(e) =
                                    set_current_turn(lobby_id, next_player_id, redis.clone()).await
                                {
                                    tracing::error!("Failed to set current turn: {}", e);
                                    continue;
                                }

                                // Update current rule for next turn
                                if let Some(next_rule) =
                                    get_rule_by_index(new_rule_index, &new_rule_context)
                                {
                                    if let Err(e) = set_current_rule(
                                        lobby_id,
                                        Some(next_rule.description.clone()),
                                        redis.clone(),
                                    )
                                    .await
                                    {
                                        tracing::error!("Failed to set next current rule: {}", e);
                                    }

                                    // Send rule to the next player (current turn)
                                    let rule_msg = LexiWarsServerMessage::Rule {
                                        rule: next_rule.description.clone(),
                                    };

                                    broadcast_to_player_and_spectators(
                                        &rule_msg,
                                        next_player_id,
                                        lobby_id,
                                        connections,
                                        &redis,
                                    )
                                    .await;
                                }

                                // Broadcast word entry to all players
                                let word_entry_msg = LexiWarsServerMessage::WordEntry {
                                    word: cleaned_word.clone(),
                                    sender: player.clone(),
                                };

                                if let Ok(players) =
                                    get_lobby_players(lobby_id, None, redis.clone()).await
                                {
                                    broadcast_to_lobby_and_spectators(
                                        &word_entry_msg,
                                        &players,
                                        lobby_id,
                                        connections,
                                        &redis,
                                    )
                                    .await;

                                    // Find next player object for turn message
                                    if let Some(next_player) =
                                        players.iter().find(|p| p.id == next_player_id)
                                    {
                                        // Broadcast turn change to all players and spectators
                                        let next_turn_msg = LexiWarsServerMessage::Turn {
                                            current_turn: next_player.clone(),
                                            countdown: 15,
                                        };
                                        broadcast_to_lobby_and_spectators(
                                            &next_turn_msg,
                                            &players,
                                            lobby_id,
                                            connections,
                                            &redis,
                                        )
                                        .await;
                                    }
                                }

                                // Start turn timer for next player
                                start_turn_timer(
                                    next_player_id,
                                    lobby_id,
                                    connections.clone(),
                                    redis.clone(),
                                    _telegram_bot.clone(),
                                );
                            } else {
                                tracing::error!(
                                    "Could not find current player in connected players list"
                                );
                            }
                        }
                    }
                }
                Message::Ping(_data) => {
                    tracing::debug!("WebSocket ping from player {}", player.id);
                }
                Message::Pong(_) => {
                    tracing::debug!("WebSocket pong from player {}", player.id);
                }
                Message::Close(_) => {
                    tracing::debug!("WebSocket close from player {}", player.id);
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

fn start_turn_timer(
    player_id: Uuid,
    lobby_id: Uuid,
    connections: ConnectionInfoMap,
    redis: RedisClient,
    telegram_bot: teloxide::Bot,
) {
    tokio::spawn(async move {
        for i in (0..=15).rev() {
            // Check if the turn is still this player's
            match get_current_turn(lobby_id, redis.clone()).await {
                Ok(Some(current_turn_id)) if current_turn_id == player_id => {
                    // Send countdown to current player and spectators
                    let countdown_msg = LexiWarsServerMessage::Countdown { time: i };
                    broadcast_to_player(player_id, lobby_id, &countdown_msg, &connections, &redis)
                        .await;

                    // Send turn info to all players
                    if let Ok(players) = get_lobby_players(lobby_id, None, redis.clone()).await {
                        if let Some(current_player) =
                            players.iter().find(|p| p.id == current_turn_id)
                        {
                            let turn_msg = LexiWarsServerMessage::Turn {
                                current_turn: current_player.clone(),
                                countdown: i,
                            };
                            broadcast_to_lobby_and_spectators(
                                &turn_msg,
                                &players,
                                lobby_id,
                                &connections,
                                &redis,
                            )
                            .await;
                        }
                    }
                }
                Ok(Some(_)) => {
                    // Turn has already changed, stop timer
                    let countdown_msg = LexiWarsServerMessage::Countdown { time: 15 };

                    broadcast_to_player(player_id, lobby_id, &countdown_msg, &connections, &redis)
                        .await;
                    tracing::info!("Turn changed, stopping timer for player {}", player_id);
                    return;
                }
                Ok(None) => {
                    tracing::error!("No current turn set for lobby {}", lobby_id);
                    return;
                }
                Err(e) => {
                    tracing::error!("Failed to check current turn: {}", e);
                    return;
                }
            }

            sleep(Duration::from_secs(1)).await;
        }

        // Time ran out - eliminate player
        match get_current_turn(lobby_id, redis.clone()).await {
            Ok(Some(current_turn_id)) if current_turn_id == player_id => {
                tracing::info!("Player {} timed out in lobby {}", player_id, lobby_id);

                // Handle turn timeout - eliminate player and advance turn
                if let Ok(current_players) = get_current_players_ids(lobby_id, redis.clone()).await
                {
                    // Eliminate the player
                    if let Err(e) = add_eliminated_player(lobby_id, player_id, redis.clone()).await
                    {
                        tracing::error!("Failed to eliminate player: {}", e);
                        return;
                    }

                    // Add eliminated player as spectator so they can continue watching
                    if let Err(e) = add_spectator(lobby_id, player_id, redis.clone()).await {
                        tracing::error!("Failed to add eliminated player as spectator: {}", e);
                    }
                    let spectator_msg = LexiWarsServerMessage::Spectator;
                    broadcast_to_player(player_id, lobby_id, &spectator_msg, &connections, &redis)
                        .await;

                    // Remove from current players (don't touch connected players)
                    if let Err(e) = remove_current_player(lobby_id, player_id, redis.clone()).await
                    {
                        tracing::error!("Failed to remove timed out player from current: {}", e);
                        return;
                    }

                    // Get updated current players and calculate position for stats
                    let remaining_players =
                        match get_current_players_ids(lobby_id, redis.clone()).await {
                            Ok(players) => players,
                            Err(e) => {
                                tracing::error!("Failed to get remaining players: {}", e);
                                return;
                            }
                        };

                    let connected_player_ids =
                        match get_connected_players_ids(lobby_id, redis.clone()).await {
                            Ok(ids) => ids,
                            Err(e) => {
                                tracing::error!("Failed to get connected players: {}", e);
                                return;
                            }
                        };

                    // Broadcast updated players count
                    let players_count_msg = LexiWarsServerMessage::PlayersCount {
                        connected_players: connected_player_ids.len(),
                        remaining_players: remaining_players.len(),
                    };
                    if let Ok(players) = get_lobby_players(lobby_id, None, redis.clone()).await {
                        broadcast_to_lobby_and_spectators(
                            &players_count_msg,
                            &players,
                            lobby_id,
                            &connections,
                            &redis,
                        )
                        .await;
                    }

                    let position = remaining_players.len() + 1;

                    // Get lobby info and connected players count for prize calculation
                    if let Ok(lobby_info) = get_lobby_info(lobby_id, redis.clone()).await {
                        let connected_players_count = connected_player_ids.len();

                        // Send stats to eliminated player
                        send_rank_prize_and_wars_point(
                            player_id,
                            lobby_id,
                            &lobby_info,
                            connected_players_count,
                            position,
                            &connections,
                            &redis,
                        )
                        .await;
                    }

                    if remaining_players.len() <= 1 {
                        // Game over
                        if let Err(e) = end_game(
                            lobby_id,
                            connected_player_ids,
                            &connections,
                            redis.clone(),
                            telegram_bot.clone(),
                        )
                        .await
                        {
                            tracing::error!("Failed to end game: {}", e);
                        }
                    } else {
                        // Find next active player
                        if let Some(current_index) =
                            current_players.iter().position(|&id| id == player_id)
                        {
                            let next_index = current_index % remaining_players.len();
                            let next_player_id = remaining_players[next_index];

                            // Set next turn
                            if let Err(e) =
                                set_current_turn(lobby_id, next_player_id, redis.clone()).await
                            {
                                tracing::error!("Failed to set current turn: {}", e);
                                return;
                            }

                            // Notify all players about elimination and next turn
                            if let Ok(players) =
                                get_lobby_players(lobby_id, None, redis.clone()).await
                            {
                                if let Some(next_player) =
                                    players.iter().find(|p| p.id == next_player_id)
                                {
                                    let next_turn_msg = LexiWarsServerMessage::Turn {
                                        current_turn: next_player.clone(),
                                        countdown: 15,
                                    };
                                    broadcast_to_lobby_and_spectators(
                                        &next_turn_msg,
                                        &players,
                                        lobby_id,
                                        &connections,
                                        &redis,
                                    )
                                    .await;
                                }
                            }

                            // Start timer for next player
                            start_turn_timer(
                                next_player_id,
                                lobby_id,
                                connections,
                                redis,
                                telegram_bot.clone(),
                            );
                        }
                    }
                }
            }
            Ok(Some(_)) => {
                // Turn has already changed, nothing to do
                tracing::debug!("Turn has already changed for lobby {}", lobby_id);
            }
            Ok(None) => {
                tracing::error!("No current turn set for lobby {}", lobby_id);
            }
            Err(e) => {
                tracing::error!("Failed to check current turn: {}", e);
            }
        }
    });
}

pub fn start_auto_start_timer(
    lobby_id: Uuid,
    connections: ConnectionInfoMap,
    redis: RedisClient,
    telegram_bot: teloxide::Bot,
) {
    tokio::spawn(async move {
        for i in (0..=15).rev() {
            // Get current lobby state from Redis
            let connected_player_ids =
                match get_connected_players_ids(lobby_id, redis.clone()).await {
                    Ok(ids) => ids,
                    Err(e) => {
                        tracing::error!("Failed to get connected players: {}", e);
                        return;
                    }
                };

            let lobby_players =
                match get_lobby_players(lobby_id, Some(PlayerState::Joined), redis.clone()).await {
                    Ok(players) => players,
                    Err(e) => {
                        tracing::error!("Failed to get lobby players: {}", e);
                        return;
                    }
                };

            let connected_count = connected_player_ids.len();
            let total_players = lobby_players.len();

            tracing::info!(
                "Auto-start timer: {}s, connected: {}/{}",
                i,
                connected_count,
                total_players
            );

            // If all players are connected, start immediately
            if connected_count == total_players {
                tracing::info!("All players connected, starting game early");
                if let Err(e) = start_game(
                    lobby_id,
                    connected_player_ids,
                    &connections,
                    redis.clone(),
                    telegram_bot.clone(),
                )
                .await
                {
                    tracing::error!("Failed to start game: {}", e);
                }
                return;
            }

            // Send countdown update to connected players
            let start_msg = LexiWarsServerMessage::Start {
                time: i,
                started: false,
            };
            for player_id in &connected_player_ids {
                broadcast_to_player(*player_id, lobby_id, &start_msg, &connections, &redis).await;
            }

            if i == 0 {
                // Timer expired, check if we have sufficient players
                let required_players = std::cmp::max(2, (total_players + 1) / 2); // At least 2 players and 50% (rounded up)

                tracing::info!(
                    "Auto-start timer expired: connected {}/{}, required: {}",
                    connected_count,
                    total_players,
                    required_players
                );

                if connected_count >= required_players && connected_count >= 2 {
                    tracing::info!(
                        "Sufficient players connected ({}%), starting game",
                        (connected_count * 100) / total_players
                    );
                    if let Err(e) = start_game(
                        lobby_id,
                        connected_player_ids,
                        &connections,
                        redis.clone(),
                        telegram_bot.clone(),
                    )
                    .await
                    {
                        tracing::error!("Failed to start game: {}", e);
                    }
                } else {
                    tracing::info!("Not enough players connected, canceling game");
                    let start_failed_msg = LexiWarsServerMessage::StartFailed;
                    for player_id in &connected_player_ids {
                        broadcast_to_player(
                            *player_id,
                            lobby_id,
                            &start_failed_msg,
                            &connections,
                            &redis,
                        )
                        .await;
                    }

                    // Reset lobby state
                    if let Err(e) =
                        update_lobby_state(lobby_id, LobbyState::Waiting, redis.clone()).await
                    {
                        tracing::error!("Error updating game state to Waiting: {}", e);
                    }
                }
                return;
            }

            sleep(Duration::from_secs(1)).await;
        }
    });
}

async fn start_game(
    lobby_id: Uuid,
    connected_player_ids: Vec<Uuid>,
    connections: &ConnectionInfoMap,
    redis: RedisClient,
    telegram_bot: teloxide::Bot,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Set game as started
    set_game_started(lobby_id, true, redis.clone()).await?;

    // Create current players - initially same as connected players
    create_current_players(lobby_id, connected_player_ids.clone(), redis.clone()).await?;

    // Get all players for broadcasting
    let players = get_lobby_players(lobby_id, None, redis.clone()).await?;

    // Initialize first turn with first connected player
    if let Some(&first_player_id) = connected_player_ids.first() {
        set_current_turn(lobby_id, first_player_id, redis.clone()).await?;

        // Get rule context and set first rule
        if let Some(rule_context) = get_rule_context(lobby_id, redis.clone()).await? {
            if let Some(first_rule) = get_rule_by_index(0, &rule_context) {
                set_current_rule(
                    lobby_id,
                    Some(first_rule.description.clone()),
                    redis.clone(),
                )
                .await?;

                // Send the rule to the current player
                let rule_msg = LexiWarsServerMessage::Rule {
                    rule: first_rule.description,
                };
                broadcast_to_player_and_spectators(
                    &rule_msg,
                    first_player_id,
                    lobby_id,
                    connections,
                    &redis,
                )
                .await;
            }
        }

        // Send first turn message to all players
        if let Some(first_player) = players.iter().find(|p| p.id == first_player_id) {
            let turn_msg = LexiWarsServerMessage::Turn {
                current_turn: first_player.clone(),
                countdown: 15,
            };
            broadcast_to_lobby_and_spectators(&turn_msg, &players, lobby_id, connections, &redis)
                .await;
        }

        // Send game started message to all players
        let game_started_msg = LexiWarsServerMessage::Start {
            time: 0,
            started: true,
        };
        broadcast_to_lobby_and_spectators(
            &game_started_msg,
            &players,
            lobby_id,
            connections,
            &redis,
        )
        .await;

        // Broadcast initial players count
        let players_count_msg = LexiWarsServerMessage::PlayersCount {
            connected_players: connected_player_ids.len(),
            remaining_players: connected_player_ids.len(),
        };
        broadcast_to_lobby_and_spectators(
            &players_count_msg,
            &players,
            lobby_id,
            connections,
            &redis,
        )
        .await;

        // Start turn timer for first player
        start_turn_timer(
            first_player_id,
            lobby_id,
            connections.clone(),
            redis,
            telegram_bot,
        );

        tracing::info!(
            "Game started for lobby {} with {} connected players",
            lobby_id,
            connected_player_ids.len()
        );
    }

    Ok(())
}

async fn end_game(
    lobby_id: Uuid,
    connected_player_ids: Vec<Uuid>,
    connections: &ConnectionInfoMap,
    redis: RedisClient,
    telegram_bot: Bot,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Update game state first to prevent race conditions
    update_lobby_state(lobby_id, LobbyState::Finished, redis.clone()).await?;

    // Get all players for final standing and broadcast
    let players = get_lobby_players(lobby_id, None, redis.clone()).await?;
    let lobby_info = get_lobby_info(lobby_id, redis.clone()).await?;
    let connected_players_count = connected_player_ids.len();

    // Handle remaining player(s) - give them final ranking
    if let Ok(remaining_players) = get_current_players_ids(lobby_id, redis.clone()).await {
        for (index, &remaining_player_id) in remaining_players.iter().enumerate() {
            let final_rank = index + 1;
            send_rank_prize_and_wars_point(
                remaining_player_id,
                lobby_id,
                &lobby_info,
                connected_players_count,
                final_rank,
                connections,
                &redis,
            )
            .await;
        }
    }

    // Get eliminated players for final standing
    let eliminated_players = get_eliminated_players(lobby_id, redis.clone()).await?;

    // Create final standing - reverse order so winner is first
    let mut final_standings = Vec::new();

    // Add remaining players first (winners)
    if let Ok(remaining_player_ids) = get_current_players_ids(lobby_id, redis.clone()).await {
        for (index, &player_id) in remaining_player_ids.iter().enumerate() {
            if let Some(mut player) = players.iter().find(|p| p.id == player_id).cloned() {
                let rank = index + 1;
                // Calculate and set the prize for this player
                player.prize = get_prize(&lobby_info, connected_players_count, rank);

                final_standings.push(PlayerStanding { player, rank });
            }
        }
    }

    // Add eliminated players in reverse order (last eliminated gets better rank)
    for (index, &player_id) in eliminated_players.iter().rev().enumerate() {
        if let Some(mut player) = players.iter().find(|p| p.id == player_id).cloned() {
            let rank = final_standings.len() + index + 1;
            // Calculate and set the prize for this player
            player.prize = get_prize(&lobby_info, connected_players_count, rank);

            final_standings.push(PlayerStanding { player, rank });
        }
    }

    // Send game over messages
    let gameover_msg = LexiWarsServerMessage::GameOver;
    broadcast_to_lobby_and_spectators(&gameover_msg, &players, lobby_id, connections, &redis).await;

    // Broadcast final standing
    let final_standing_msg = LexiWarsServerMessage::FinalStanding {
        standing: final_standings.iter().cloned().collect(),
    };
    broadcast_to_lobby_and_spectators(&final_standing_msg, &players, lobby_id, connections, &redis)
        .await;

    if let Some(tg_msg_id) = lobby_info.tg_msg_id {
        tokio::spawn(async move {
            let winner_payload = create_winner_payload(
                lobby_id,
                &lobby_info,
                &final_standings,
                connected_players_count,
                tg_msg_id,
            );
            let chat_id = std::env::var("TELEGRAM_CHAT_ID")
                .expect("TELEGRAM_CHAT_ID must be set")
                .parse::<i64>()
                .unwrap();

            if let Err(e) =
                bot::broadcast_lobby_winner(&telegram_bot, chat_id, winner_payload).await
            {
                tracing::error!("Failed to broadcast lobby winner: {}", e);
            }
        });
    }

    // Clean up Redis data
    if let Err(e) = clear_lobby_game_state(lobby_id, redis.clone()).await {
        tracing::error!("Failed to clear lobby game state: {}", e);
    }

    tracing::info!("Game ended for lobby {}", lobby_id);
    Ok(())
}

fn create_winner_payload(
    lobby_id: Uuid,
    lobby_info: &LobbyInfo,
    final_standings: &[PlayerStanding],
    connected_players_count: usize,
    tg_msg_id: i32,
) -> BotLobbyWinnerPayload {
    let winner = &final_standings[0];

    let winner_wallet = winner
        .player
        .user
        .as_ref()
        .map(|u| u.wallet_address.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let winner_name = winner
        .player
        .user
        .as_ref()
        .and_then(|u| u.display_name.clone().or_else(|| u.username.clone()));

    // Create runner-ups (2nd and 3rd place if they exist)
    let mut runner_ups = Vec::new();

    if final_standings.len() > 1 {
        let second_place = &final_standings[1];
        let second_wallet = second_place
            .player
            .user
            .as_ref()
            .map(|u| u.wallet_address.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        let second_name = second_place
            .player
            .user
            .as_ref()
            .and_then(|u| u.display_name.clone().or_else(|| u.username.clone()));

        let second_prize = get_prize(lobby_info, connected_players_count, 2);

        runner_ups.push(RunnerUp {
            name: second_name,
            wallet: second_wallet,
            position: "2nd".to_string(),
            prize: second_prize,
        });
    }

    if final_standings.len() > 2 {
        let third_place = &final_standings[2];
        let third_wallet = third_place
            .player
            .user
            .as_ref()
            .map(|u| u.wallet_address.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        let third_name = third_place
            .player
            .user
            .as_ref()
            .and_then(|u| u.display_name.clone().or_else(|| u.username.clone()));

        let third_prize = get_prize(lobby_info, connected_players_count, 3);

        runner_ups.push(RunnerUp {
            name: third_name,
            wallet: third_wallet,
            position: "3rd".to_string(),
            prize: third_prize,
        });
    }

    BotLobbyWinnerPayload {
        lobby_id,
        lobby_name: lobby_info.name.clone(),
        game: lobby_info.game.clone(),
        winner_name,
        winner_wallet,
        winner_prize: winner.player.prize,
        entry_amount: lobby_info.entry_amount,
        runner_ups,
        tg_msg_id,
    }
}
