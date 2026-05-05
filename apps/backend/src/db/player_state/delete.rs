// Delete operations for PlayerState (Redis)

use crate::db::player_state::PlayerStateRepository;
use crate::errors::AppError;
use crate::models::{KeyPart, RedisKey};
use crate::state::AppState;
use redis::AsyncCommands;
use uuid::Uuid;

impl PlayerStateRepository {
    /// Delete a player's state from Redis.
    pub async fn delete_state(
        &self,
        lobby_id: Uuid,
        user_id: Uuid,
        app_state: Option<AppState>,
    ) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_player(lobby_id, user_id);

        let deleted: usize = conn.del(&key).await.map_err(AppError::RedisCommandError)?;

        if deleted == 0 {
            return Err(AppError::NotFound(format!(
                "Player state for user {} in lobby {} not found",
                user_id, lobby_id
            )));
        }

        if let Some(app_state) = app_state {
            crate::ws::broadcast::broadcast_lobby_update(app_state, lobby_id).await;
        }

        Ok(())
    }

    /// Delete all player states in a lobby; returns number deleted.
    pub async fn cleanup_lobby(&self, lobby_id: Uuid) -> Result<usize, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let pattern = RedisKey::lobby_player(lobby_id, KeyPart::Wildcard);

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
}
