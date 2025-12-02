use redis::AsyncCommands;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::redis::{KeyPart, RedisKey},
    state::RedisClient,
};

pub async fn add_player_used_word(
    lobby_id: Uuid,
    player_id: Uuid,
    word: &str,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let player_key = RedisKey::lobby_player(KeyPart::Id(lobby_id), KeyPart::Id(player_id));

    let current_words: Option<String> = conn
        .hget(&player_key, "used_words")
        .await
        .map_err(AppError::RedisCommandError)?;

    let mut used_words: Vec<String> = match current_words {
        Some(words_json) if !words_json.is_empty() => {
            serde_json::from_str(&words_json).unwrap_or_else(|_| Vec::new())
        }
        _ => Vec::new(),
    };

    let word_lower = word.to_lowercase();
    if !used_words.contains(&word_lower) {
        used_words.push(word_lower);
    }

    let words_json = serde_json::to_string(&used_words)
        .map_err(|e| AppError::Serialization(format!("Failed to serialize used words: {}", e)))?;

    let _: () = conn
        .hset(&player_key, "used_words", words_json)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn _get_player_used_words(
    lobby_id: Uuid,
    player_id: Uuid,
    redis: RedisClient,
) -> Result<Vec<String>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let player_key = RedisKey::lobby_player(KeyPart::Id(lobby_id), KeyPart::Id(player_id));
    let words_json: Option<String> = conn
        .hget(&player_key, "used_words")
        .await
        .map_err(AppError::RedisCommandError)?;

    match words_json {
        Some(json) if !json.is_empty() => serde_json::from_str(&json).map_err(|e| {
            AppError::Deserialization(format!("Failed to deserialize used words: {}", e))
        }),
        _ => Ok(Vec::new()),
    }
}

pub async fn _has_player_used_word(
    lobby_id: Uuid,
    player_id: Uuid,
    word: &str,
    redis: RedisClient,
) -> Result<bool, AppError> {
    let used_words = _get_player_used_words(lobby_id, player_id, redis).await?;
    Ok(used_words.contains(&word.to_lowercase()))
}
