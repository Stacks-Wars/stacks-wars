use crate::models::{ChatMessage, RedisKey};
use bb8_redis::{RedisConnectionManager, bb8::Pool, redis::AsyncCommands};
use uuid::Uuid;

/// Creates a new chat message in Redis.
///
/// Uses both sorted set (for ordering) and hash (for message data):
/// - Sorted set: `lobbies:{lobby_id}:chat` with timestamp as score
/// - Hash: `lobbies:{lobby_id}:chat:messages:{message_id}` with message data
pub async fn create_chat_message(
    redis_pool: &Pool<RedisConnectionManager>,
    lobby_id: Uuid,
    user_id: Uuid,
    content: &str,
    reply_to: Option<Uuid>,
) -> Result<ChatMessage, String> {
    let message = ChatMessage::new(lobby_id, user_id, content, reply_to)
        .map_err(|e| format!("Invalid chat message: {}", e))?;

    let mut conn = redis_pool
        .get()
        .await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

    let chat_key = RedisKey::lobby_chat(lobby_id);
    let message_key = RedisKey::lobby_chat_message(lobby_id, message.message_id);
    let timestamp = message.created_at.timestamp();

    // Serialize message to JSON
    let message_json = serde_json::to_string(&message)
        .map_err(|e| format!("Failed to serialize message: {}", e))?;

    // Add to sorted set and store message data
    let _: () = conn
        .zadd(&chat_key, message.message_id.to_string(), timestamp)
        .await
        .map_err(|e| format!("Failed to add message to sorted set: {}", e))?;

    let _: () = conn
        .set(&message_key, message_json)
        .await
        .map_err(|e| format!("Failed to store message data: {}", e))?;

    // Set expiration to 24 hours (chat is temporary)
    let _: () = conn
        .expire(&message_key, 86400)
        .await
        .map_err(|e| format!("Failed to set message expiration: {}", e))?;

    Ok(message)
}
