use redis::AsyncCommands;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::redis::{KeyPart, RedisKey},
    state::RedisClient,
};

pub async fn create_current_players(
    lobby_id: Uuid,
    current_player_ids: Vec<Uuid>,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let current_key = RedisKey::lobby_current_players(KeyPart::Id(lobby_id));

    let _: () = conn
        .del(&current_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    // Add all current player IDs to the set
    if !current_player_ids.is_empty() {
        let player_id_strings: Vec<String> = current_player_ids
            .into_iter()
            .map(|id| id.to_string())
            .collect();

        let _: () = conn
            .sadd(&current_key, player_id_strings)
            .await
            .map_err(AppError::RedisCommandError)?;
    }

    Ok(())
}

pub async fn remove_current_player(
    lobby_id: Uuid,
    player_id: Uuid,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let current_key = RedisKey::lobby_current_players(KeyPart::Id(lobby_id));
    let _: () = conn
        .srem(&current_key, player_id.to_string())
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}
