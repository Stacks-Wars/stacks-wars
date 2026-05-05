use crate::db::join_request::{JoinRequest, JoinRequestRepository, JoinRequestState};
use crate::models::keys::RedisKey;
use redis::AsyncCommands;
use uuid::Uuid;

impl JoinRequestRepository {
    /// Update the state of a join request (e.g., from Pending to Accepted/Rejected).
    pub async fn set_state(
        &self,
        lobby_id: Uuid,
        user_id: Uuid,
        state: JoinRequestState,
    ) -> redis::RedisResult<()> {
        if let Ok(mut conn) = self.redis.get().await {
            let key = RedisKey::lobby_join_requests(lobby_id);
            let raw_res: redis::RedisResult<String> = conn.hget(&key, user_id.to_string()).await;
            if let Ok(raw) = raw_res {
                if let Ok(mut jr) = serde_json::from_str::<JoinRequest>(&raw) {
                    jr.state = state;
                    let _: redis::RedisResult<i32> = conn
                        .hset(
                            &key,
                            user_id.to_string(),
                            serde_json::to_string(&jr).unwrap(),
                        )
                        .await;
                }
            }
        }
        Ok(())
    }
}
