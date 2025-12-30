// Small repository for lobby join-request lifecycle stored in Redis.
use crate::models::keys::RedisKey;
use crate::state::RedisClient;
use chrono::Utc;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum JoinRequestState {
    Pending,
    Accepted,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JoinRequest {
    pub user_id: Uuid,
    pub state: JoinRequestState,
    pub wallet_address: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub trust_rating: f64,
    pub is_creator: bool,
    pub created_at: i64,
}

#[derive(Clone)]
pub struct JoinRequestRepository {
    redis: RedisClient,
}

impl JoinRequestRepository {
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }

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

    /// Remove a join request for a specific user in the lobby.
    pub async fn remove(&self, lobby_id: Uuid, user_id: Uuid) -> redis::RedisResult<()> {
        if let Ok(mut conn) = self.redis.get().await {
            let key = RedisKey::lobby_join_requests(lobby_id);
            let _: redis::RedisResult<i32> = conn.hdel(&key, user_id.to_string()).await;
        }
        Ok(())
    }
}
