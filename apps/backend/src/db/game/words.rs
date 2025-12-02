use redis::AsyncCommands;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::redis::{KeyPart, RedisKey},
    state::RedisClient,
};

pub async fn add_word_set(redis: RedisClient) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let words_key = RedisKey::words_set();

    // Check if word set already exists
    let exists: bool = conn
        .exists(&words_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    if exists {
        tracing::info!("Word set already exists in Redis");
        return Ok(());
    }

    tracing::info!("Loading words from JSON file...");

    // Read and parse the words.json file
    let words_json = include_str!("../../assets/words.json");
    let words: Vec<String> = serde_json::from_str(words_json)
        .map_err(|e| AppError::Deserialization(format!("Failed to parse words.json: {}", e)))?;

    tracing::info!("Loaded {} words from JSON file", words.len());

    // Add all words to Redis set
    if !words.is_empty() {
        let _: () = conn
            .sadd(&words_key, words)
            .await
            .map_err(AppError::RedisCommandError)?;
    }

    tracing::info!("Successfully added word set to Redis");
    Ok(())
}

pub async fn is_valid_word(word: &str, redis: RedisClient) -> Result<bool, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let words_key = RedisKey::words_set();
    let is_member: bool = conn
        .sismember(&words_key, word.to_lowercase())
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(is_member)
}

pub async fn _get_random_words(count: usize, redis: RedisClient) -> Result<Vec<String>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let words_key = RedisKey::words_set();
    let words: Vec<String> = conn
        .srandmember_multiple(&words_key, count)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(words)
}

pub async fn add_used_word(lobby_id: Uuid, word: &str, redis: RedisClient) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let used_words_key = RedisKey::lobby_used_words(KeyPart::Id(lobby_id));
    let _: () = conn
        .sadd(&used_words_key, word.to_lowercase())
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn is_word_used_in_lobby(
    lobby_id: Uuid,
    word: &str,
    redis: RedisClient,
) -> Result<bool, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let used_words_key = RedisKey::lobby_used_words(KeyPart::Id(lobby_id));
    let is_member: bool = conn
        .sismember(&used_words_key, word.to_lowercase())
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(is_member)
}
