// Update operations for LobbyState (Redis)

use crate::db::lobby_state::LobbyStateRepository;
use crate::errors::AppError;
use crate::models::LobbyStatus;
use crate::models::keys::RedisKey;
use chrono::Utc;
use redis::AsyncCommands;
use uuid::Uuid;

impl LobbyStateRepository {
    /// Update lobby status.
    pub async fn update_status(&self, lobby_id: Uuid, status: LobbyStatus) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_state(lobby_id);

        // Check if exists
        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(AppError::RedisCommandError)?;

        if !exists {
            return Err(AppError::NotFound(format!(
                "Lobby state {} not found",
                lobby_id
            )));
        }

        // Update status and updated_at
        let now = Utc::now().timestamp();
        let status_str = format!("{:?}", status);

        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("status", status_str.as_str()),
                    ("updated_at", &now.to_string()),
                ],
            )
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Set the participant count for a lobby.
    pub async fn update_participant_count(
        &self,
        lobby_id: Uuid,
        count: usize,
    ) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_state(lobby_id);

        let now = Utc::now().timestamp();

        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("participant_count", &count.to_string()),
                    ("updated_at", &now.to_string()),
                ],
            )
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Increment participant count by 1 and return the new count.
    pub async fn increment_participants(&self, lobby_id: Uuid) -> Result<usize, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_state(lobby_id);

        let new_count: usize = conn
            .hincr(&key, "participant_count", 1)
            .await
            .map_err(AppError::RedisCommandError)?;

        // Update timestamp
        let now = Utc::now().timestamp();
        let _: () = conn
            .hset(&key, "updated_at", now)
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(new_count)
    }

    /// Decrement participant count by 1 (never negative) and return the new count.
    pub async fn decrement_participants(&self, lobby_id: Uuid) -> Result<usize, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_state(lobby_id);

        let new_count: isize = conn
            .hincr(&key, "participant_count", -1)
            .await
            .map_err(AppError::RedisCommandError)?;

        // Ensure count doesn't go negative
        let new_count = new_count.max(0) as usize;

        // Update timestamp
        let now = Utc::now().timestamp();
        let _: () = conn
            .hset(&key, "updated_at", now)
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(new_count)
    }

    /// Mark the lobby as started and set `started_at`.
    pub async fn mark_started(&self, lobby_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_state(lobby_id);

        let now = Utc::now().timestamp();

        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("status", "InProgress"),
                    ("started_at", &now.to_string()),
                    ("updated_at", &now.to_string()),
                ],
            )
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Mark the lobby as finished and set `finished_at`.
    pub async fn mark_finished(&self, lobby_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_state(lobby_id);

        let now = Utc::now().timestamp();

        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("status", "Finished"),
                    ("finished_at", &now.to_string()),
                    ("updated_at", &now.to_string()),
                ],
            )
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Update the lobby creator's last ping timestamp.
    pub async fn update_creator_ping(&self, lobby_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_state(lobby_id);

        let now_ms = Utc::now().timestamp_millis() as u64;
        let now = Utc::now().timestamp();

        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("creator_last_ping", &now_ms.to_string()),
                    ("updated_at", &now.to_string()),
                ],
            )
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Update the Telegram message ID associated with the lobby.
    pub async fn update_tg_msg_id(&self, lobby_id: Uuid, msg_id: i32) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_state(lobby_id);

        let now = Utc::now().timestamp();

        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("tg_msg_id", &msg_id.to_string()),
                    ("updated_at", &now.to_string()),
                ],
            )
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Touch the lobby (refresh `updated_at`).
    pub async fn touch(&self, lobby_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_state(lobby_id);

        let now = Utc::now().timestamp();

        let _: () = conn
            .hset(&key, "updated_at", now)
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Persist the countdown seconds for a lobby (overwrites) and set a short expiry.
    pub async fn set_countdown(
        &self,
        lobby_id: Uuid,
        seconds_remaining: u8,
    ) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;

        let key = RedisKey::lobby_countdown(lobby_id);

        // Store as integer and set a TTL so cancelled/finished countdowns expire.
        let _: () = conn
            .set(&key, seconds_remaining)
            .await
            .map_err(AppError::RedisCommandError)?;

        // Keep countdown around for a short period (e.g., 60s) so clients can pick it up.
        let _: () = conn
            .expire(&key, 60)
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Remove the countdown key for a lobby.
    pub async fn clear_countdown(&self, lobby_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;

        let key = RedisKey::lobby_countdown(lobby_id);

        let _: () = conn.del(&key).await.map_err(AppError::RedisCommandError)?;

        Ok(())
    }
}
