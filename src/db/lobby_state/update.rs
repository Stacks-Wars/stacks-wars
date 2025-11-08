//! Update operations for LobbyState

use crate::db::lobby_state::LobbyStateRepository;
use crate::errors::AppError;
use crate::models::enums::LobbyState as LobbyStatus;
use chrono::Utc;
use redis::AsyncCommands;
use uuid::Uuid;

impl LobbyStateRepository {
    /// Update lobby status
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `status` - The new status
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn update_status(&self, lobby_id: Uuid, status: LobbyStatus) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", lobby_id);

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

    /// Update participant count
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `count` - The new participant count
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn update_participant_count(
        &self,
        lobby_id: Uuid,
        count: usize,
    ) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", lobby_id);

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

    /// Increment participant count by 1
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(usize)` - The new count
    pub async fn increment_participants(&self, lobby_id: Uuid) -> Result<usize, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", lobby_id);

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

    /// Decrement participant count by 1
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(usize)` - The new count
    pub async fn decrement_participants(&self, lobby_id: Uuid) -> Result<usize, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", lobby_id);

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

    /// Mark lobby as started (sets started_at timestamp)
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn mark_started(&self, lobby_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", lobby_id);

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

    /// Mark lobby as finished (sets finished_at timestamp)
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn mark_finished(&self, lobby_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", lobby_id);

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

    /// Update creator's last ping timestamp
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn update_creator_ping(&self, lobby_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", lobby_id);

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

    /// Update Telegram message ID
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `msg_id` - The Telegram message ID
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn update_tg_msg_id(&self, lobby_id: Uuid, msg_id: i32) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", lobby_id);

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

    /// Touch the lobby (update updated_at timestamp)
    ///
    /// Useful for keeping track of activity without changing other fields.
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn touch(&self, lobby_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", lobby_id);

        let now = Utc::now().timestamp();

        let _: () = conn
            .hset(&key, "updated_at", now)
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }
}
