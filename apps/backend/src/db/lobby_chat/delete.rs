use crate::db::lobby_chat::LobbyChatRepository;
use crate::models::RedisKey;
use redis::AsyncCommands;
use uuid::Uuid;

impl LobbyChatRepository {
    /// Deletes a chat message from Redis.
    ///
    /// Removes both the sorted set entry and the message data.
    pub async fn delete_message(&self, lobby_id: Uuid, message_id: Uuid) -> Result<(), String> {
        let mut conn = self
            .redis
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
}
