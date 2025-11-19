// Create/update operations for SpectatorState (Redis)

use crate::db::spectator_state::SpectatorStateRepository;
use crate::errors::AppError;
use crate::models::redis::keys::RedisKey;
use crate::models::redis::spectator_state::SpectatorState;
use redis::AsyncCommands;
use uuid::Uuid;

impl SpectatorStateRepository {
    /// Upsert (create or update) a spectator state in Redis.
    pub async fn upsert_state(&self, state: SpectatorState) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;

        let key = RedisKey::lobby_spectator(state.lobby_id, state.user_id);

        let hash = state.to_redis_hash();
        let hash_pairs: Vec<(&String, &String)> = hash.iter().collect();
        let _: () = conn
            .hset_multiple(&key, &hash_pairs)
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Remove a spectator from a lobby in Redis.
    pub async fn remove_from_lobby(&self, lobby_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_spectator(lobby_id, user_id);
        let _: () = conn.del(&key).await.map_err(AppError::RedisCommandError)?;
        Ok(())
    }
}
