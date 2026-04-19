use crate::db::join_request::{JoinRequest, JoinRequestRepository};
use crate::models::keys::RedisKey;
use redis::AsyncCommands;
use std::collections::HashMap;
use uuid::Uuid;

impl JoinRequestRepository {
    /// Get a specific join request by user ID.
    pub async fn get(&self, lobby_id: Uuid, user_id: Uuid) -> Option<JoinRequest> {
        if let Ok(mut conn) = self.redis.get().await {
            let key = RedisKey::lobby_join_requests(lobby_id);
            let raw_res: redis::RedisResult<String> = conn.hget(&key, user_id.to_string()).await;
            if let Ok(raw) = raw_res {
                if let Ok(jr) = serde_json::from_str::<JoinRequest>(&raw) {
                    return Some(jr);
                }
            }
        }
        None
    }

    /// List all join requests for a lobby.
    pub async fn list(&self, lobby_id: Uuid) -> redis::RedisResult<Vec<JoinRequest>> {
        let mut out = Vec::new();
        if let Ok(mut conn) = self.redis.get().await {
            let key = RedisKey::lobby_join_requests(lobby_id);
            let map: HashMap<String, String> = conn.hgetall(&key).await?;
            for (_field, raw) in map.into_iter() {
                if let Ok(jr) = serde_json::from_str::<JoinRequest>(&raw) {
                    out.push(jr);
                }
            }
        }
        Ok(out)
    }
}
