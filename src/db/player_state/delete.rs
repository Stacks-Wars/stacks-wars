//! Delete operations for PlayerState

use crate::db::player_state::PlayerStateRepository;
use crate::errors::AppError;
use redis::AsyncCommands;
use uuid::Uuid;

impl PlayerStateRepository {
    /// Delete player state
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `user_id` - The user UUID
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(AppError::NotFound)` if player state doesn't exist
    pub async fn delete_state(&self, lobby_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:players:{}", lobby_id, user_id);

        let deleted: usize = conn.del(&key).await.map_err(AppError::RedisCommandError)?;

        if deleted == 0 {
            return Err(AppError::NotFound(format!(
                "Player state for user {} in lobby {} not found",
                user_id, lobby_id
            )));
        }

        Ok(())
    }

    /// Delete player state (soft - doesn't error if not found)
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `user_id` - The user UUID
    ///
    /// # Returns
    /// * `Ok(bool)` - true if deleted, false if didn't exist
    pub async fn delete_state_soft(&self, lobby_id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:players:{}", lobby_id, user_id);

        let deleted: usize = conn.del(&key).await.map_err(AppError::RedisCommandError)?;

        Ok(deleted > 0)
    }

    /// Remove player from lobby (alias for delete_state)
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `user_id` - The user UUID
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn remove_from_lobby(&self, lobby_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        self.delete_state(lobby_id, user_id).await
    }

    /// Delete all player states in a lobby
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of players deleted
    pub async fn cleanup_lobby(&self, lobby_id: Uuid) -> Result<usize, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let pattern = format!("lobbies:{}:players:*", lobby_id);

        let keys: Vec<String> = conn
            .keys(&pattern)
            .await
            .map_err(AppError::RedisCommandError)?;

        if keys.is_empty() {
            return Ok(0);
        }

        let deleted: usize = conn.del(&keys).await.map_err(AppError::RedisCommandError)?;

        Ok(deleted)
    }

    /// Delete unclaimed prizes older than a threshold
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `older_than_secs` - Delete unclaimed prizes older than this many seconds
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of player states deleted
    pub async fn cleanup_unclaimed_prizes(
        &self,
        lobby_id: Uuid,
        older_than_secs: i64,
    ) -> Result<usize, AppError> {
        let all_players = self.get_all_in_lobby(lobby_id).await?;

        let now = chrono::Utc::now().timestamp();
        let threshold = now - older_than_secs;

        let mut deleted_count = 0;

        for player in all_players {
            if player.has_prize() && !player.has_claimed() {
                // Check if updated_at is older than threshold
                if player.updated_at < threshold {
                    if self.delete_state_soft(lobby_id, player.user_id).await? {
                        deleted_count += 1;
                    }
                }
            }
        }

        Ok(deleted_count)
    }

    /// Delete players who haven't pinged recently
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `timeout_secs` - Delete players who haven't pinged in this many seconds
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of players deleted
    pub async fn cleanup_inactive_players(
        &self,
        lobby_id: Uuid,
        timeout_secs: u64,
    ) -> Result<usize, AppError> {
        let all_players = self.get_all_in_lobby(lobby_id).await?;

        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        let threshold_ms = now_ms - (timeout_secs * 1000);

        let mut deleted_count = 0;

        for player in all_players {
            if let Some(last_ping) = player.last_ping {
                if last_ping < threshold_ms {
                    if self.delete_state_soft(lobby_id, player.user_id).await? {
                        deleted_count += 1;
                    }
                }
            } else {
                // No ping ever - check joined_at
                let joined_ms = (player.joined_at * 1000) as u64;
                if joined_ms < threshold_ms {
                    if self.delete_state_soft(lobby_id, player.user_id).await? {
                        deleted_count += 1;
                    }
                }
            }
        }

        Ok(deleted_count)
    }
}
