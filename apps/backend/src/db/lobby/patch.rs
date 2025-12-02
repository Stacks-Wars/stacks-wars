use chrono::Utc;
use redis::AsyncCommands;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    db::{
        chat::delete::delete_lobby_chat, lobby::join_requests::remove_all_lobby_join_requests,
        tx::validate_payment_tx, user::get::get_user_by_id,
    },
    errors::AppError,
    models::{
        game::{ClaimState, LobbyInfo, LobbyState, Player, PlayerState},
        redis::{KeyPart, RedisKey},
    },
    state::RedisClient,
};

pub async fn join_lobby(
    lobby_id: Uuid,
    user_id: Uuid,
    tx_id: Option<String>,
    player_state: PlayerState,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let lobby_key = RedisKey::lobby(KeyPart::Id(lobby_id));
    let lobby_map: HashMap<String, String> = conn
        .hgetall(&lobby_key)
        .await
        .map_err(AppError::RedisCommandError)?;
    if lobby_map.is_empty() {
        return Err(AppError::NotFound(format!("Lobby {} not found", lobby_id)));
    }
    let (lobby, _creator_id, _game_id) = LobbyInfo::from_redis_hash_partial(&lobby_map)?;

    let player_key = RedisKey::lobby_player(KeyPart::Id(lobby_id), KeyPart::Id(user_id));
    let mut existing_player_state: Option<PlayerState> = None;

    if conn
        .exists(&player_key)
        .await
        .map_err(AppError::RedisCommandError)?
    {
        // Check existing player state
        let player_map: HashMap<String, String> = conn
            .hgetall(&player_key)
            .await
            .map_err(AppError::RedisCommandError)?;

        if let Ok(existing_player) = Player::from_redis_hash(&player_map) {
            existing_player_state = Some(existing_player.state.clone());

            if existing_player.state == PlayerState::Joined {
                return Err(AppError::BadRequest("User already in lobby".into()));
            }
        }
    }

    if let Some(addr) = &lobby.contract_address {
        let entry_amount = lobby.entry_amount.unwrap_or(0.0);

        if entry_amount > 0.0 && player_state != PlayerState::NotJoined {
            let tx = tx_id.clone().ok_or_else(|| {
                AppError::BadRequest("Missing transaction ID for paid lobby".into())
            })?;

            let user = get_user_by_id(user_id, redis.clone()).await?;
            validate_payment_tx(&tx, &user.wallet_address, addr, entry_amount).await?;

            // Increment pool current amount
            let _: () = conn
                .hincr(&lobby_key, "current_amount", entry_amount as i64)
                .await
                .map_err(AppError::RedisCommandError)?;
        }
    }

    let new_player = Player::new(user_id, tx_id, player_state.clone());
    let player_hash = new_player.to_redis_hash();
    let player_fields: Vec<(&str, &str)> = player_hash
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let _: () = conn
        .hset_multiple(&player_key, &player_fields)
        .await
        .map_err(AppError::RedisCommandError)?;

    let should_increment_participants = player_state == PlayerState::Joined
        && (existing_player_state.is_none()
            || existing_player_state == Some(PlayerState::NotJoined));

    if should_increment_participants {
        let _: () = conn
            .hincr(&lobby_key, "participants", 1)
            .await
            .map_err(AppError::RedisCommandError)?;
    }

    Ok(())
}

pub async fn leave_lobby(
    lobby_id: Uuid,
    user_id: Uuid,
    redis: RedisClient,
    bot: teloxide::Bot,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let lobby_key = RedisKey::lobby(KeyPart::Id(lobby_id));
    let m: HashMap<String, String> = conn
        .hgetall(&lobby_key)
        .await
        .map_err(AppError::RedisCommandError)?;
    if m.is_empty() {
        return Err(AppError::NotFound(format!("Lobby {} not found", lobby_id)));
    }
    let (info, creator_id, game_id) = LobbyInfo::from_redis_hash_partial(&m)?;

    if creator_id == user_id {
        // Creator leaving - delete entire lobby
        let pattern = RedisKey::lobby_player(KeyPart::Id(lobby_id), KeyPart::Wildcard);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut *conn)
            .await
            .map_err(AppError::RedisCommandError)?;

        if keys.len() == 1 {
            // Only creator left - delete lobby and clean up all references
            let _: () = conn
                .del(&lobby_key)
                .await
                .map_err(AppError::RedisCommandError)?;

            // Delete all player hashes
            for key in keys {
                let _: () = conn.del(key).await.map_err(AppError::RedisCommandError)?;
            }

            // Clean up sorted sets - remove lobby from all relevant sets
            let lobby_id_str = lobby_id.to_string();

            // Remove from lobbies:all
            let _: () = conn
                .zrem(RedisKey::lobbies_all(), &lobby_id_str)
                .await
                .map_err(AppError::RedisCommandError)?;

            // Remove from lobbies state set
            let _: () = conn
                .zrem(RedisKey::lobbies_state(&info.state), &lobby_id_str)
                .await
                .map_err(AppError::RedisCommandError)?;

            // Remove from game lobbies set
            let _: () = conn
                .zrem(RedisKey::game_lobbies(KeyPart::Id(game_id)), &lobby_id_str)
                .await
                .map_err(AppError::RedisCommandError)?;

            // Update game active lobby count
            //update_game_active_lobby(game_id, false, redis.clone()).await?;

            // Delete Telegram lobby creation message if bot is available and tg_msg_id exists
            if let Some(tg_msg_id) = info.tg_msg_id {
                tokio::spawn(async move {
                    let chat_id = std::env::var("TELEGRAM_CHAT_ID")
                        .expect("TELEGRAM_CHAT_ID must be set")
                        .parse::<i64>()
                        .unwrap();

                    if let Err(e) =
                        crate::http::bot::delete_lobby_creation_message(&bot, chat_id, tg_msg_id)
                            .await
                    {
                        tracing::error!("Failed to delete lobby creation message: {}", e);
                    }
                });
            }
        } else {
            return Err(AppError::BadRequest(
                "Creator cannot leave lobby with players".into(),
            ));
        }

        return Ok(());
    }

    // Regular player leaving
    let player_key = RedisKey::lobby_player(KeyPart::Id(lobby_id), KeyPart::Id(user_id));
    if !conn
        .exists(&player_key)
        .await
        .map_err(AppError::RedisCommandError)?
    {
        return Err(AppError::BadRequest("User not in lobby".into()));
    }

    // Get player state before deletion to check if we need to decrement participant count and pool
    let player_map: HashMap<String, String> = conn
        .hgetall(&player_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    let joined = if let Ok(player) = Player::from_redis_hash(&player_map) {
        player.state == PlayerState::Joined
    } else {
        false
    };

    let _: () = conn
        .del(&player_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    if joined {
        let _: () = conn
            .hincr(&lobby_key, "participants", -1)
            .await
            .map_err(AppError::RedisCommandError)?;
    }

    // Only update pool amount for paid lobbies (entry_amount > 0) where player actually paid (not idle)
    if let Some(_addr) = &info.contract_address {
        let entry_amount = info.entry_amount.unwrap_or(0.0);

        if entry_amount > 0.0 && joined {
            // Regular paid lobby - refund player by decreasing pool (only if they weren't idle)
            let _: () = conn
                .hincr(&lobby_key, "current_amount", -(entry_amount as i64))
                .await
                .map_err(AppError::RedisCommandError)?;
        }
    }

    Ok(())
}

pub async fn update_lobby_state(
    lobby_id: Uuid,
    new_state: LobbyState,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let lobby_key = RedisKey::lobby(KeyPart::Id(lobby_id));

    // Read the existing state and game_id
    let old_state_str: String = conn
        .hget(&lobby_key, "state")
        .await
        .map_err(AppError::RedisCommandError)?;
    //let game_id_str: String = conn
    //    .hget(&lobby_key, "game_id")
    //    .await
    //    .map_err(AppError::RedisCommandError)?;

    let old_state = old_state_str
        .parse::<LobbyState>()
        .map_err(|_| AppError::Deserialization("Invalid old state".into()))?;
    //let game_id = game_id_str
    //    .parse::<Uuid>()
    //    .map_err(|_| AppError::Deserialization("Invalid game_id".into()))?;

    // No-op if state is unchanged
    if old_state == new_state {
        return Ok(());
    }

    let _: () = conn
        .hset(&lobby_key, "state", format!("{:?}", new_state))
        .await
        .map_err(AppError::RedisCommandError)?;

    // Move the lobby ID between the old & new state ZSETs
    let score = Utc::now().timestamp();
    let old_z = RedisKey::lobbies_state(&old_state);
    let new_z = RedisKey::lobbies_state(&new_state);

    let _: () = conn
        .zrem(&old_z, lobby_id.to_string())
        .await
        .map_err(AppError::RedisCommandError)?;
    let _: () = conn
        .zadd(&new_z, lobby_id.to_string(), score)
        .await
        .map_err(AppError::RedisCommandError)?;

    //if new_state == LobbyState::Finished
    //    && (old_state == LobbyState::Waiting
    //        || old_state == LobbyState::InProgress
    //        || old_state == LobbyState::Starting)
    //{
    //    update_game_active_lobby(game_id, false, redis.clone()).await?;
    //} else if (new_state == LobbyState::InProgress
    //    || new_state == LobbyState::Waiting
    //    || new_state == LobbyState::Starting)
    //    && old_state == LobbyState::Finished
    //{
    //    update_game_active_lobby(game_id, true, redis.clone()).await?;
    //}

    // clean up chat history if transitioning to Finished
    if new_state == LobbyState::Finished {
        if let Err(e) = delete_lobby_chat(lobby_id, &redis).await {
            tracing::error!(
                "Failed to delete lobby chat history when setting state to Finished: {}",
                e
            );
        }

        if let Err(e) = remove_all_lobby_join_requests(lobby_id, redis.clone()).await {
            tracing::error!(
                "Failed to delete lobby join requests when setting state to Finished: {}",
                e
            );
        } else {
            tracing::info!(
                "Cleaned up all join requests for finished lobby {}",
                lobby_id
            );
        }
    }

    Ok(())
}

pub async fn update_player_state(
    lobby_id: Uuid,
    user_id: Uuid,
    new_state: PlayerState,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    // Build the player hash key
    let player_key = RedisKey::lobby_player(KeyPart::Id(lobby_id), KeyPart::Id(user_id));

    // Fetch the existing hash
    let map: HashMap<String, String> = conn
        .hgetall(&player_key)
        .await
        .map_err(AppError::RedisCommandError)?;
    if map.is_empty() {
        return Err(AppError::NotFound(format!(
            "Player {} not found in lobby {}",
            user_id, lobby_id
        )));
    }

    let player = Player::from_redis_hash(&map)?;

    // Short‐circuit if no change
    if player.state == new_state {
        return Ok(());
    }

    // Update only the "state" field in Redis
    let _: () = conn
        .hset(&player_key, "state", format!("{:?}", new_state))
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn update_claim_state(
    lobby_id: Uuid,
    user_id: Uuid,
    new_claim: ClaimState,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let player_key = RedisKey::lobby_player(KeyPart::Id(lobby_id), KeyPart::Id(user_id));
    let map: HashMap<String, String> = conn
        .hgetall(&player_key)
        .await
        .map_err(AppError::RedisCommandError)?;
    if map.is_empty() {
        return Err(AppError::NotFound(format!(
            "Player {} not found in lobby {}",
            user_id, lobby_id
        )));
    }

    let current = Player::from_redis_hash(&map)?;
    if current.claim.as_ref() == Some(&new_claim) {
        return Ok(());
    }

    let claim_json =
        serde_json::to_string(&new_claim).map_err(|e| AppError::Serialization(e.to_string()))?;
    let _: () = conn
        .hset(&player_key, "claim", claim_json)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn add_connected_player(
    lobby_id: Uuid,
    player_id: Uuid,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let connected_key = RedisKey::lobby_connected_players(KeyPart::Id(lobby_id));
    let _: () = conn
        .sadd(&connected_key, player_id.to_string())
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn remove_connected_player(
    lobby_id: Uuid,
    player_id: Uuid,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let connected_key = RedisKey::lobby_connected_players(KeyPart::Id(lobby_id));
    let _: () = conn
        .srem(&connected_key, player_id.to_string())
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn _update_player_prize(
    lobby_id: Uuid,
    player_id: Uuid,
    prize: Option<f64>,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let player_key = RedisKey::lobby_player(KeyPart::Id(lobby_id), KeyPart::Id(player_id));

    let map: HashMap<String, String> = conn
        .hgetall(&player_key)
        .await
        .map_err(AppError::RedisCommandError)?;
    if map.is_empty() {
        return Err(AppError::NotFound(format!(
            "Player {} not found in lobby {}",
            player_id, lobby_id
        )));
    }

    let player = Player::from_redis_hash(&map)?;

    // Check if prize is already set to the same value
    if player.prize == prize {
        return Ok(());
    }

    let mut updates = Vec::new();

    // Handle prize update
    if let Some(amount) = prize {
        updates.push(("prize", amount.to_string()));

        // Set claim state to NotClaimed when prize is set
        let claim_json = serde_json::to_string(&ClaimState::NotClaimed)
            .map_err(|e| AppError::Serialization(e.to_string()))?;
        updates.push(("claim", claim_json));
    }

    if updates.is_empty() {
        return Ok(()); // No updates needed
    }

    let hset_args: Vec<(&str, &str)> = updates.iter().map(|(k, v)| (*k, v.as_str())).collect();

    let _: () = conn
        .hset_multiple(&player_key, &hset_args)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn _update_player_rank(
    lobby_id: Uuid,
    player_id: Uuid,
    rank: usize,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let player_key = RedisKey::lobby_player(KeyPart::Id(lobby_id), KeyPart::Id(player_id));

    let map: HashMap<String, String> = conn
        .hgetall(&player_key)
        .await
        .map_err(AppError::RedisCommandError)?;
    if map.is_empty() {
        return Err(AppError::NotFound(format!(
            "Player {} not found in lobby {}",
            player_id, lobby_id
        )));
    }

    let player = Player::from_redis_hash(&map)?;

    // Check if rank is already set to the same value
    if player.rank == Some(rank) {
        return Ok(());
    }

    // Update rank
    let _: () = conn
        .hset(&player_key, "rank", rank.to_string())
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn _update_player_used_words(
    lobby_id: Uuid,
    player_id: Uuid,
    used_words: Vec<String>,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let player_key = RedisKey::lobby_player(KeyPart::Id(lobby_id), KeyPart::Id(player_id));

    let map: HashMap<String, String> = conn
        .hgetall(&player_key)
        .await
        .map_err(AppError::RedisCommandError)?;
    if map.is_empty() {
        return Err(AppError::NotFound(format!(
            "Player {} not found in lobby {}",
            player_id, lobby_id
        )));
    }

    let player = Player::from_redis_hash(&map)?;

    // Check if used_words is already set to the same value
    if player.used_words == Some(used_words.clone()) {
        return Ok(());
    }

    // Update used_words
    let used_words_json =
        serde_json::to_string(&used_words).map_err(|e| AppError::Serialization(e.to_string()))?;

    let _: () = conn
        .hset(&player_key, "used_words", used_words_json)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn _update_lexi_wars_player(
    lobby_id: Uuid,
    player_id: Uuid,
    rank: usize,
    prize: Option<f64>,
    used_words: Vec<String>,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let player_key = RedisKey::lobby_player(KeyPart::Id(lobby_id), KeyPart::Id(player_id));
    let map: HashMap<String, String> = conn
        .hgetall(&player_key)
        .await
        .map_err(AppError::RedisCommandError)?;
    if map.is_empty() {
        return Err(AppError::NotFound(format!(
            "Player {} not found in lobby {}",
            player_id, lobby_id
        )));
    }

    let mut player = Player::from_redis_hash(&map)?;
    player.rank = Some(rank);
    player.used_words = Some(used_words.clone());
    if let Some(amount) = prize {
        player.prize = Some(amount);
        player.claim = Some(ClaimState::NotClaimed);
    }

    let mut updates = Vec::new();
    updates.push(("rank", rank.to_string()));
    updates.push((
        "used_words",
        serde_json::to_string(&used_words).map_err(|e| AppError::Serialization(e.to_string()))?,
    ));
    if let Some(amount) = player.prize {
        updates.push(("prize", amount.to_string()));
    }
    if let Some(ref c) = player.claim {
        updates.push((
            "claim",
            serde_json::to_string(c).map_err(|e| AppError::Serialization(e.to_string()))?,
        ));
    }

    let hset_args: Vec<(&str, &str)> = updates
        .iter()
        .map(|(k, v)| (k.as_ref(), v.as_str()))
        .collect();
    let _: () = conn
        .hset_multiple(&player_key, &hset_args)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn add_spectator(
    lobby_id: Uuid,
    user_id: Uuid,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let spectators_key = RedisKey::lobby_spectators(KeyPart::Id(lobby_id));
    let _: () = conn
        .sadd(&spectators_key, user_id.to_string())
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn remove_spectator(
    lobby_id: Uuid,
    user_id: Uuid,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let spectators_key = RedisKey::lobby_spectators(KeyPart::Id(lobby_id));
    let _: () = conn
        .srem(&spectators_key, user_id.to_string())
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}
