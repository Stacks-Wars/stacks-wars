// Create operations for PlayerState

use crate::db::player_state::PlayerStateRepository;
use crate::errors::AppError;
use crate::models::{PlayerState, RedisKey};
use crate::state::AppState;
use crate::ws::broadcast_lobby_update;
use redis::AsyncCommands;

impl PlayerStateRepository {
    /// Upsert (create or update) a player's state in Redis.
    pub async fn upsert_state(
        &self,
        state: PlayerState,
        app_state: Option<AppState>,
    ) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_player(state.lobby_id, state.user_id);
        // Convert to hash and store (overwrites if exists)
        let hash = state.to_redis_hash();
        let hash_pairs: Vec<(&String, &String)> = hash.iter().collect();
        let _: () = conn
            .hset_multiple(&key, &hash_pairs)
            .await
            .map_err(AppError::RedisCommandError)?;

        // Broadcast lobby update if AppState provided
        if let Some(app_state) = app_state {
            broadcast_lobby_update(app_state, state.lobby_id).await;
        }

        Ok(())
    }

    /// Create a new player state in Redis.
    pub async fn create_state(
        &self,
        state: PlayerState,
        app_state: Option<AppState>,
    ) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_player(state.lobby_id, state.user_id);

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

        self.upsert_state(state, app_state).await?;

        Ok(())
    }
}
