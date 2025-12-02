use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        chat::ChatMessage,
        redis::{KeyPart, RedisKey},
    },
    state::RedisClient,
};

pub async fn store_chat_message(
    lobby_id: Uuid,
    chat_message: &ChatMessage,
    redis: &RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let key = RedisKey::lobby_chat(KeyPart::Id(lobby_id));
    let serialized_message =
        serde_json::to_string(chat_message).map_err(|e| AppError::Serialization(e.to_string()))?;

    // Add message to the end of the list
    let _: () = redis::cmd("RPUSH")
        .arg(&key)
        .arg(&serialized_message)
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    // Trim to keep only the last 100 messages
    let _: () = redis::cmd("LTRIM")
        .arg(&key)
        .arg(-100) // Keep last 100 messages
        .arg(-1) // To the end
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    // Set TTL to 1 week (604800 seconds)
    let _: () = redis::cmd("EXPIRE")
        .arg(&key)
        .arg(604800)
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    tracing::debug!("Stored chat message in Redis for lobby {}", lobby_id);
    Ok(())
}
