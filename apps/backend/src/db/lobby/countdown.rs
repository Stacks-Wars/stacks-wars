use redis::AsyncCommands;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::redis::{KeyPart, RedisKey},
    state::RedisClient,
};

pub async fn set_lobby_countdown(
    lobby_id: Uuid,
    time: u32,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let key = RedisKey::lobby_countdown(KeyPart::Id(lobby_id));

    // Set countdown with 30 second expiration to auto-cleanup
    let _: () = conn
        .set_ex(&key, time, 30)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}

pub async fn get_lobby_countdown(
    lobby_id: Uuid,
    redis: RedisClient,
) -> Result<Option<u32>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let key = RedisKey::lobby_countdown(KeyPart::Id(lobby_id));

    let time: Option<u32> = conn.get(&key).await.map_err(AppError::RedisCommandError)?;

    Ok(time)
}

pub async fn clear_lobby_countdown(lobby_id: Uuid, redis: RedisClient) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let key = RedisKey::lobby_countdown(KeyPart::Id(lobby_id));

    let _: () = conn.del(&key).await.map_err(AppError::RedisCommandError)?;

    Ok(())
}
