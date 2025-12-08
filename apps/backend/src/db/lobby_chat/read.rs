use crate::models::{ChatMessage, RedisKey};
use bb8_redis::{RedisConnectionManager, bb8::Pool, redis::AsyncCommands};
use uuid::Uuid;

/// Gets the most recent chat messages from a lobby.
///
/// Returns messages in reverse chronological order (newest first).
/// Limit defaults to 50 messages.
pub async fn get_chat_history(
    redis_pool: &Pool<RedisConnectionManager>,
    lobby_id: Uuid,
    limit: Option<usize>,
) -> Result<Vec<ChatMessage>, String> {
    let mut conn = redis_pool
        .get()
        .await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

    let chat_key = RedisKey::lobby_chat(lobby_id);
    let limit = limit.unwrap_or(50);

    // Get message IDs from sorted set in reverse order (newest first)
    let message_ids: Vec<String> = conn
        .zrevrange(&chat_key, 0, (limit - 1) as isize)
        .await
        .map_err(|e| format!("Failed to get message IDs: {}", e))?;

    if message_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Fetch all messages in parallel
    let mut messages = Vec::new();
    for message_id_str in message_ids {
        let message_id =
            Uuid::parse_str(&message_id_str).map_err(|e| format!("Invalid message ID: {}", e))?;

        if let Ok(Some(message)) = get_chat_message(redis_pool, lobby_id, message_id).await {
            messages.push(message);
        }
    }

    Ok(messages)
}

/// Gets a specific chat message by ID.
pub async fn get_chat_message(
    redis_pool: &Pool<RedisConnectionManager>,
    lobby_id: Uuid,
    message_id: Uuid,
) -> Result<Option<ChatMessage>, String> {
    let mut conn = redis_pool
        .get()
        .await
        .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

    let message_key = RedisKey::lobby_chat_message(lobby_id, message_id);

    let message_json: Option<String> = conn
        .get(&message_key)
        .await
        .map_err(|e| format!("Failed to get message: {}", e))?;

    match message_json {
        Some(json) => {
            let message: ChatMessage = serde_json::from_str(&json)
                .map_err(|e| format!("Failed to deserialize message: {}", e))?;
            Ok(Some(message))
        }
        None => Ok(None),
    }
}
