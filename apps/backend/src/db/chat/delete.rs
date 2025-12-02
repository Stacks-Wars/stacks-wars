use uuid::Uuid;

use crate::{
    errors::AppError,
    models::redis::{KeyPart, RedisKey},
    state::RedisClient,
};

pub async fn delete_lobby_chat(lobby_id: Uuid, redis: &RedisClient) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let key = RedisKey::lobby_chat(KeyPart::Id(lobby_id));

    let deleted: u32 = redis::cmd("DEL")
        .arg(&key)
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    if deleted > 0 {
        tracing::info!(
            "Deleted lobby chat history for lobby {} (game finished)",
            lobby_id
        );
    } else {
        tracing::debug!("No chat history found for lobby {} to delete", lobby_id);
    }

    Ok(())
}
