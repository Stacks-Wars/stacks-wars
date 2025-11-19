// Read operations for SpectatorState (Redis)

use crate::db::spectator_state::SpectatorStateRepository;
use crate::errors::AppError;
use crate::models::redis::keys::{KeyPart, RedisKey};
use crate::models::redis::spectator_state::SpectatorState;
use redis::AsyncCommands;
use std::collections::HashMap;
use uuid::Uuid;

impl SpectatorStateRepository {
    /// Return all spectator states for a lobby.
    pub async fn get_all_in_lobby(&self, lobby_id: Uuid) -> Result<Vec<SpectatorState>, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;

        let pattern = RedisKey::lobby_spectator(lobby_id, KeyPart::Wildcard);

        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(AppError::RedisCommandError)?;

        let mut states = Vec::new();

        for key in keys {
            let map: HashMap<String, String> = conn
                .hgetall(&key)
                .await
                .map_err(AppError::RedisCommandError)?;

            if !map.is_empty() {
                if let Ok(state) = SpectatorState::from_redis_hash(&map) {
                    states.push(state);
                }
            }
        }

        Ok(states)
    }

    /// Count spectators in a lobby.
    pub async fn count_spectators(&self, lobby_id: Uuid) -> Result<usize, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let pattern = RedisKey::lobby_spectator(lobby_id, KeyPart::Wildcard);

        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(keys.len())
    }

    /// Get spectator UUIDs present in a lobby.
    pub async fn get_spectator_ids(&self, lobby_id: Uuid) -> Result<Vec<Uuid>, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let pattern = RedisKey::lobby_spectator(lobby_id, KeyPart::Wildcard);

        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(AppError::RedisCommandError)?;

        let mut ids = Vec::new();

        for key in keys {
            if let Some(part) = key.split(':').last() {
                if let Ok(id) = Uuid::parse_str(part) {
                    ids.push(id);
                }
            }
        }

        Ok(ids)
    }
}
