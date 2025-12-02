use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    errors::AppError,
    games::lexi_wars::rules::RuleContext,
    models::redis::{KeyPart, RedisKey},
    state::RedisClient,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyGameState {
    pub rule_context: RuleContext,
    pub rule_index: usize,
    pub current_turn_id: Uuid,
    pub eliminated_players: Vec<Uuid>,
    pub game_started: bool,
    pub current_rule: Option<String>,
}

impl LobbyGameState {
    pub fn _new(rule_context: RuleContext, rule_index: usize, current_turn_id: Uuid) -> Self {
        Self {
            rule_context,
            rule_index,
            current_turn_id,
            eliminated_players: Vec::new(),
            game_started: false,
            current_rule: None,
        }
    }
}

pub async fn set_rule_context(
    lobby_id: Uuid,
    rule_context: &RuleContext,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let rule_context_key = RedisKey::lobby_rule_context(KeyPart::Id(lobby_id));
    let serialized = serde_json::to_string(rule_context)
        .map_err(|e| AppError::Serialization(format!("Failed to serialize rule context: {}", e)))?;

    let _: () = conn
        .set(&rule_context_key, serialized)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn get_rule_context(
    lobby_id: Uuid,
    redis: RedisClient,
) -> Result<Option<RuleContext>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let rule_context_key = RedisKey::lobby_rule_context(KeyPart::Id(lobby_id));
    let serialized: Option<String> = conn
        .get(&rule_context_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    match serialized {
        Some(data) => {
            let rule_context = serde_json::from_str(&data).map_err(|e| {
                AppError::Deserialization(format!("Failed to deserialize rule context: {}", e))
            })?;
            Ok(Some(rule_context))
        }
        None => Ok(None),
    }
}

pub async fn set_rule_index(
    lobby_id: Uuid,
    rule_index: usize,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let rule_index_key = RedisKey::lobby_rule_index(KeyPart::Id(lobby_id));
    let _: () = conn
        .set(&rule_index_key, rule_index)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

/// Get the rule index for a lobby
pub async fn get_rule_index(lobby_id: Uuid, redis: RedisClient) -> Result<Option<usize>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let rule_index_key = RedisKey::lobby_rule_index(KeyPart::Id(lobby_id));
    let rule_index: Option<usize> = conn
        .get(&rule_index_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(rule_index)
}

pub async fn set_current_turn(
    lobby_id: Uuid,
    player_id: Uuid,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let current_turn_key = RedisKey::lobby_current_turn(KeyPart::Id(lobby_id));
    let _: () = conn
        .set(&current_turn_key, player_id.to_string())
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn get_current_turn(
    lobby_id: Uuid,
    redis: RedisClient,
) -> Result<Option<Uuid>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let current_turn_key = RedisKey::lobby_current_turn(KeyPart::Id(lobby_id));
    let player_id: Option<String> = conn
        .get(&current_turn_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    match player_id {
        Some(id_str) => {
            let uuid = Uuid::parse_str(&id_str).map_err(|e| {
                AppError::Deserialization(format!("Invalid UUID for current turn: {}", e))
            })?;
            Ok(Some(uuid))
        }
        None => Ok(None),
    }
}

pub async fn add_eliminated_player(
    lobby_id: Uuid,
    player_id: Uuid,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let eliminated_key = RedisKey::lobby_eliminated_players(KeyPart::Id(lobby_id));
    let _: () = conn
        .sadd(&eliminated_key, player_id.to_string())
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn get_eliminated_players(
    lobby_id: Uuid,
    redis: RedisClient,
) -> Result<Vec<Uuid>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let eliminated_key = RedisKey::lobby_eliminated_players(KeyPart::Id(lobby_id));
    let player_ids: Vec<String> = conn
        .smembers(&eliminated_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    let mut uuids = Vec::new();
    for id_str in player_ids {
        let uuid = Uuid::parse_str(&id_str).map_err(|e| {
            AppError::Deserialization(format!("Invalid UUID for eliminated player: {}", e))
        })?;
        uuids.push(uuid);
    }

    Ok(uuids)
}

pub async fn set_game_started(
    lobby_id: Uuid,
    started: bool,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let game_started_key = RedisKey::lobby_game_started(KeyPart::Id(lobby_id));
    let _: () = conn
        .set(&game_started_key, started)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn get_game_started(lobby_id: Uuid, redis: RedisClient) -> Result<bool, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let game_started_key = RedisKey::lobby_game_started(KeyPart::Id(lobby_id));
    let started: Option<bool> = conn
        .get(&game_started_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(started.unwrap_or(false))
}

pub async fn set_current_rule(
    lobby_id: Uuid,
    rule: Option<String>,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let current_rule_key = RedisKey::lobby_current_rule(KeyPart::Id(lobby_id));

    match rule {
        Some(rule_text) => {
            let _: () = conn
                .set(&current_rule_key, rule_text)
                .await
                .map_err(AppError::RedisCommandError)?;
        }
        None => {
            let _: () = conn
                .del(&current_rule_key)
                .await
                .map_err(AppError::RedisCommandError)?;
        }
    }

    Ok(())
}

pub async fn get_current_rule(
    lobby_id: Uuid,
    redis: RedisClient,
) -> Result<Option<String>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let current_rule_key = RedisKey::lobby_current_rule(KeyPart::Id(lobby_id));
    let rule: Option<String> = conn
        .get(&current_rule_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(rule)
}

pub async fn clear_lobby_game_state(lobby_id: Uuid, redis: RedisClient) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let keys = vec![
        RedisKey::lobby_rule_context(KeyPart::Id(lobby_id)),
        RedisKey::lobby_rule_index(KeyPart::Id(lobby_id)),
        RedisKey::lobby_current_turn(KeyPart::Id(lobby_id)),
        RedisKey::lobby_eliminated_players(KeyPart::Id(lobby_id)),
        RedisKey::lobby_game_started(KeyPart::Id(lobby_id)),
        RedisKey::lobby_current_rule(KeyPart::Id(lobby_id)),
        RedisKey::lobby_used_words(KeyPart::Id(lobby_id)),
        RedisKey::lobby_current_players(KeyPart::Id(lobby_id)),
        RedisKey::lobby_spectators(KeyPart::Id(lobby_id)),
    ];

    let _: () = conn.del(&keys).await.map_err(AppError::RedisCommandError)?;

    Ok(())
}
