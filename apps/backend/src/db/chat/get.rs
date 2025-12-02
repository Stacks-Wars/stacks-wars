use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        chat::ChatMessage,
        redis::{KeyPart, RedisKey},
    },
    state::RedisClient,
};

pub async fn get_chat_history(
    lobby_id: Uuid,
    redis: &RedisClient,
) -> Result<Vec<ChatMessage>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let key = RedisKey::lobby_chat(KeyPart::Id(lobby_id));

    let messages: Vec<String> = redis::cmd("LRANGE")
        .arg(&key)
        .arg(0)
        .arg(-1) // Get all messages
        .query_async(&mut *conn)
        .await
        .map_err(AppError::RedisCommandError)?;

    let mut chat_messages = Vec::new();
    for message_str in messages {
        if let Ok(chat_message) = serde_json::from_str::<ChatMessage>(&message_str) {
            chat_messages.push(chat_message);
        } else {
            tracing::warn!(
                "Failed to deserialize chat message from Redis: {}",
                message_str
            );
        }
    }

    Ok(chat_messages)
}
