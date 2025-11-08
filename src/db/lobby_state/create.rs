//! Create operations for LobbyState

use crate::db::lobby_state::LobbyStateRepository;
use crate::errors::AppError;
use crate::models::redis::LobbyState;
use redis::AsyncCommands;

impl LobbyStateRepository {
    /// Create a new lobby state in Redis
    ///
    /// # Arguments
    /// * `state` - The LobbyState to create
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(AppError)` if the state already exists or Redis operation fails
    pub async fn create_state(&self, state: LobbyState) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", state.lobby_id);

        // Check if state already exists
        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(AppError::RedisCommandError)?;

        if exists {
            return Err(AppError::AlreadyExists(format!(
                "Lobby state {} already exists",
                state.lobby_id
            )));
        }

        // Convert to hash and store
        let hash = state.to_redis_hash();
        let _: () = conn
            .hset_multiple(&key, &hash)
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Create or update lobby state (upsert)
    ///
    /// # Arguments
    /// * `state` - The LobbyState to create or update
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn upsert_state(&self, state: LobbyState) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", state.lobby_id);

        // Convert to hash and store (overwrites if exists)
        let hash = state.to_redis_hash();
        let _: () = conn
            .hset_multiple(&key, &hash)
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }
}
