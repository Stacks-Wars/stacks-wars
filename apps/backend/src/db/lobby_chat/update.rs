use crate::models::{ChatMessage, RedisKey};
use bb8_redis::{bb8::Pool, redis::AsyncCommands, RedisConnectionManager};
use uuid::Uuid;

/// Adds a reaction to a chat message.
pub async fn add_reaction(
    redis_pool: &Pool<RedisConnectionManager>,
    lobby_id: Uuid,
    message_id: Uuid,
    user_id: Uuid,
    emoji: &str,
) -> Result<ChatMessage, String> {
    let mut conn = redis_pool
        .get()
        .await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

    let message_key = RedisKey::lobby_chat_message(lobby_id, message_id);

    // Get current message
    let message_json: Option<String> = conn
        .get(&message_key)
        .await
        .map_err(|e| format!("Failed to get message: {}", e))?;

    let message_json = message_json.ok_or_else(|| "Message not found".to_string())?;

    let mut message: ChatMessage = serde_json::from_str(&message_json)
        .map_err(|e| format!("Failed to deserialize message: {}", e))?;

    // Add reaction
    message.add_reaction(user_id, emoji);

    // Save updated message
    let updated_json = serde_json::to_string(&message)
        .map_err(|e| format!("Failed to serialize message: {}", e))?;

    let _: () = conn
        .set(&message_key, updated_json)
        .await
        .map_err(|e| format!("Failed to update message: {}", e))?;

    Ok(message)
}

/// Removes a reaction from a chat message.
pub async fn remove_reaction(
    redis_pool: &Pool<RedisConnectionManager>,
    lobby_id: Uuid,
    message_id: Uuid,
    user_id: Uuid,
    emoji: &str,
) -> Result<ChatMessage, String> {
    let mut conn = redis_pool
        .get()
        .await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

    let message_key = RedisKey::lobby_chat_message(lobby_id, message_id);

    // Get current message
    let message_json: Option<String> = conn
        .get(&message_key)
        .await
        .map_err(|e| format!("Failed to get message: {}", e))?;

    let message_json = message_json.ok_or_else(|| "Message not found".to_string())?;

    let mut message: ChatMessage = serde_json::from_str(&message_json)
        .map_err(|e| format!("Failed to deserialize message: {}", e))?;

    // Remove reaction
    message.remove_reaction(user_id, emoji);

    // Save updated message
    let updated_json = serde_json::to_string(&message)
        .map_err(|e| format!("Failed to serialize message: {}", e))?;

    let _: () = conn
        .set(&message_key, updated_json)
        .await
        .map_err(|e| format!("Failed to update message: {}", e))?;

    Ok(message)
}
