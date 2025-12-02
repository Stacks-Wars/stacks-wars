use redis::AsyncCommands;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::redis::{KeyPart, RedisKey},
    state::RedisClient,
};

pub async fn update_game_active_lobby(
    game_id: Uuid,
    increment: bool,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let game_key = RedisKey::game(KeyPart::Id(game_id));

    let exists: bool = conn
        .exists(&game_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    if !exists {
        return Err(AppError::NotFound(format!("Game {} not found", game_id)));
    }

    // Increment or decrement the active_lobbies field
    let delta = if increment { 1 } else { -1 };
    let _: () = conn
        .hincr(&game_key, "active_lobbies", delta)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(())
}
