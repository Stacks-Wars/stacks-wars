// Create operations for PlayerState

use crate::db::player_state::PlayerStateRepository;
use crate::errors::AppError;
use crate::models::redis::PlayerState;
use redis::AsyncCommands;

impl PlayerStateRepository {
    /// Create a new player state in Redis.
    pub async fn create_state(&self, state: PlayerState) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:players:{}", state.lobby_id, state.user_id);

        // Check if state already exists
        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(AppError::RedisCommandError)?;

        if exists {
            return Err(AppError::AlreadyExists(format!(
                "Player state for user {} in lobby {} already exists",
                state.user_id, state.lobby_id
            )));
        }

        // Convert to hash and store
        let hash = state.to_redis_hash();
        let hash_pairs: Vec<(&String, &String)> = hash.iter().collect();
        let _: () = conn
            .hset_multiple(&key, &hash_pairs)
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Upsert (create or update) a player's state in Redis.
    pub async fn upsert_state(&self, state: PlayerState) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:players:{}", state.lobby_id, state.user_id);
        // Convert to hash and store (overwrites if exists)
        let hash = state.to_redis_hash();
        let hash_pairs: Vec<(&String, &String)> = hash.iter().collect();
        let _: () = conn
            .hset_multiple(&key, &hash_pairs)
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }
}
