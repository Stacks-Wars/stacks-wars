use crate::db::join_request::{JoinRequest, JoinRequestRepository, JoinRequestState};
use crate::models::keys::RedisKey;
use chrono::Utc;
use redis::AsyncCommands;
use uuid::Uuid;

impl JoinRequestRepository {
    /// Create a new pending join request for a user in a lobby.
    pub async fn create_pending(
        &self,
        lobby_id: Uuid,
        user_id: Uuid,
        wallet_address: String,
        username: Option<String>,
        display_name: Option<String>,
        trust_rating: f64,
        ttl_seconds: usize,
    ) -> redis::RedisResult<()> {
        if let Ok(mut conn) = self.redis.get().await {
            let key = RedisKey::lobby_join_requests(lobby_id);
            let jr = JoinRequest {
                user_id,
                state: JoinRequestState::Pending,
                wallet_address,
                username,
                display_name,
                trust_rating,
                is_creator: false,
                created_at: Utc::now().timestamp(),
            };
            let _: redis::RedisResult<i32> = conn
                .hset(
                    &key,
                    user_id.to_string(),
                    serde_json::to_string(&jr).unwrap(),
                )
                .await;
            let _: redis::RedisResult<bool> = conn.expire(&key, ttl_seconds as i64).await;
        }
        Ok(())
    }
}
