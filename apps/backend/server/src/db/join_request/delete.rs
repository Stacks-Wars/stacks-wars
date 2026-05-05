use crate::db::join_request::JoinRequestRepository;
use crate::models::keys::RedisKey;
use redis::AsyncCommands;
use uuid::Uuid;

impl JoinRequestRepository {
    /// Remove a join request for a specific user in the lobby.
    pub async fn remove(&self, lobby_id: Uuid, user_id: Uuid) -> redis::RedisResult<()> {
        if let Ok(mut conn) = self.redis.get().await {
            let key = RedisKey::lobby_join_requests(lobby_id);
            let _: redis::RedisResult<i32> = conn.hdel(&key, user_id.to_string()).await;
        }
        Ok(())
    }
}
