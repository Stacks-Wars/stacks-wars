// Create operations for LobbyState (Redis)

use crate::db::lobby_state::LobbyStateRepository;
use crate::errors::AppError;
use crate::models::redis::LobbyState;
use redis::AsyncCommands;

impl LobbyStateRepository {
    /// Create a new lobby state in Redis (fails if already exists).
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

    /// Upsert (create or overwrite) a lobby state in Redis.
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
