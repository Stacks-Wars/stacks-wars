// Update operations for SpectatorState (Redis)

use crate::db::spectator_state::SpectatorStateRepository;
use crate::errors::AppError;
use crate::models::redis::keys::RedisKey;
use chrono::Utc;
use redis::AsyncCommands;
use uuid::Uuid;

impl SpectatorStateRepository {
    /// Update a spectator's last ping timestamp and refresh TTL.
    pub async fn update_ping(&self, lobby_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;

        let key = RedisKey::lobby_spectator(lobby_id, user_id);

        let now_ms = Utc::now().timestamp_millis() as u64;
        let now = Utc::now().timestamp();

        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("last_ping", &now_ms.to_string()),
                    ("updated_at", &now.to_string()),
                ],
            )
            .await
            .map_err(AppError::RedisCommandError)?;

        // Refresh TTL for spectator entries (75s)
        let _: redis::RedisResult<bool> = conn.expire(&key, 75).await;

        Ok(())
    }

    /// Touch the spectator state (refresh updated_at timestamp and TTL).
    pub async fn touch(&self, lobby_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;

        let key = RedisKey::lobby_spectator(lobby_id, user_id);

        let now = Utc::now().timestamp();

        let _: () = conn
            .hset(&key, "updated_at", now)
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }
}
