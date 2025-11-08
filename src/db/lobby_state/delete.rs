//! Delete operations for LobbyState

use crate::db::lobby_state::LobbyStateRepository;
use crate::errors::AppError;
use crate::models::enums::LobbyState as LobbyStatus;
use redis::AsyncCommands;
use uuid::Uuid;

impl LobbyStateRepository {
    /// Delete lobby state
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(AppError::NotFound)` if lobby state doesn't exist
    pub async fn delete_state(&self, lobby_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", lobby_id);

        let deleted: usize = conn.del(&key).await.map_err(AppError::RedisCommandError)?;

        if deleted == 0 {
            return Err(AppError::NotFound(format!(
                "Lobby state {} not found",
                lobby_id
            )));
        }

        Ok(())
    }

    /// Delete lobby state (soft - doesn't error if not found)
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(bool)` - true if deleted, false if didn't exist
    pub async fn delete_state_soft(&self, lobby_id: Uuid) -> Result<bool, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", lobby_id);

        let deleted: usize = conn.del(&key).await.map_err(AppError::RedisCommandError)?;

        Ok(deleted > 0)
    }

    /// Cleanup finished lobbies older than a threshold
    ///
    /// # Arguments
    /// * `older_than_secs` - Delete lobbies finished more than this many seconds ago
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of lobbies deleted
    pub async fn cleanup_finished(&self, older_than_secs: i64) -> Result<usize, AppError> {
        let states = self.get_by_status(LobbyStatus::Finished).await?;

        let now = chrono::Utc::now().timestamp();
        let threshold = now - older_than_secs;

        let mut deleted_count = 0;

        for state in states {
            if let Some(finished_at) = state.finished_at {
                if finished_at < threshold {
                    if self.delete_state_soft(state.lobby_id).await? {
                        deleted_count += 1;
                    }
                }
            }
        }

        Ok(deleted_count)
    }

    /// Cleanup abandoned lobbies (waiting/starting but creator hasn't pinged)
    ///
    /// # Arguments
    /// * `timeout_secs` - Delete lobbies where creator hasn't pinged in this many seconds
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of lobbies deleted
    pub async fn cleanup_abandoned(&self, timeout_secs: u64) -> Result<usize, AppError> {
        let waiting = self.get_by_status(LobbyStatus::Waiting).await?;
        let starting = self.get_by_status(LobbyStatus::Starting).await?;

        let all_lobbies = waiting.into_iter().chain(starting);

        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        let threshold_ms = now_ms - (timeout_secs * 1000);

        let mut deleted_count = 0;

        for state in all_lobbies {
            if let Some(last_ping) = state.creator_last_ping {
                if last_ping < threshold_ms {
                    if self.delete_state_soft(state.lobby_id).await? {
                        deleted_count += 1;
                    }
                }
            } else {
                // No ping ever - check created_at
                let created_ms = (state.created_at * 1000) as u64;
                if created_ms < threshold_ms {
                    if self.delete_state_soft(state.lobby_id).await? {
                        deleted_count += 1;
                    }
                }
            }
        }

        Ok(deleted_count)
    }
}
