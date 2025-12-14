use crate::models::RedisKey;
use bb8_redis::{RedisConnectionManager, bb8::Pool, redis::AsyncCommands};
use uuid::Uuid;

/// Deletes a chat message from Redis.
///
/// Removes both the sorted set entry and the message data.
pub async fn delete_chat_message(
    redis_pool: &Pool<RedisConnectionManager>,
    lobby_id: Uuid,
    message_id: Uuid,
) -> Result<(), String> {
    let mut conn = redis_pool
        .get()
        .await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

    let chat_key = RedisKey::lobby_chat(lobby_id);
    let message_key = RedisKey::lobby_chat_message(lobby_id, message_id);

    // Remove from sorted set
    let _: () = conn
        .zrem(&chat_key, message_id.to_string())
        .await
        .map_err(|e| format!("Failed to remove message from sorted set: {}", e))?;

    // Delete message data
    let _: () = conn
        .del(&message_key)
        .await
        .map_err(|e| format!("Failed to delete message data: {}", e))?;

    Ok(())
}
