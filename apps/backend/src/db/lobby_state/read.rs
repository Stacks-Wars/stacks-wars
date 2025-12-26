// Read operations for LobbyState (Redis)

use crate::db::lobby_state::LobbyStateRepository;
use crate::errors::AppError;
use crate::models::keys::{KeyPart, RedisKey};
use crate::models::{LobbyState, LobbyStatus};
use redis::AsyncCommands;
use std::collections::HashMap;
use uuid::Uuid;

impl LobbyStateRepository {
    /// Get the lobby state by UUID from Redis.
    pub async fn get_state(&self, lobby_id: Uuid) -> Result<LobbyState, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_state(lobby_id);

        let map: HashMap<String, String> = conn
            .hgetall(&key)
            .await
            .map_err(AppError::RedisCommandError)?;

        if map.is_empty() {
            return Err(AppError::NotFound(format!(
                "Lobby state {} not found",
                lobby_id
            )));
        }

        LobbyState::from_redis_hash(&map)
    }

    /// Check whether a lobby state exists in Redis.
    pub async fn exists(&self, lobby_id: Uuid) -> Result<bool, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_state(lobby_id);

        conn.exists(&key).await.map_err(AppError::RedisCommandError)
    }

    /// Retrieve a lobby's current `LobbyStatus` from Redis.
    pub async fn get_status(&self, lobby_id: Uuid) -> Result<LobbyStatus, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_state(lobby_id);

        let status: Option<String> = conn
            .hget(&key, "status")
            .await
            .map_err(AppError::RedisCommandError)?;

        match status {
            Some(s) => s
                .parse()
                .map_err(|_| AppError::InvalidInput(format!("Invalid lobby status: {}", s))),
            None => Err(AppError::NotFound(format!(
                "Lobby state {} not found",
                lobby_id
            ))),
        }
    }

    /// Return the participant count for a lobby.
    pub async fn get_participant_count(&self, lobby_id: Uuid) -> Result<usize, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_state(lobby_id);

        let count: Option<usize> = conn
            .hget(&key, "participant_count")
            .await
            .map_err(AppError::RedisCommandError)?;

        count.ok_or_else(|| AppError::NotFound(format!("Lobby state {} not found", lobby_id)))
    }

    /// Fetch all lobby states (optional `limit`).
    pub async fn get_all(&self, limit: Option<usize>) -> Result<Vec<LobbyState>, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let pattern = RedisKey::lobby_state(KeyPart::Wildcard);

        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(AppError::RedisCommandError)?;

        let keys_to_fetch = if let Some(limit) = limit {
            keys.into_iter().take(limit).collect()
        } else {
            keys
        };

        let mut states = Vec::new();

        for key in keys_to_fetch {
            let map: HashMap<String, String> = conn
                .hgetall(&key)
                .await
                .map_err(AppError::RedisCommandError)?;

            if !map.is_empty() {
                if let Ok(state) = LobbyState::from_redis_hash(&map) {
                    states.push(state);
                }
            }
        }

        Ok(states)
    }

    /// Return lobby states filtered by `LobbyStatus`.
    pub async fn get_by_status(&self, status: LobbyStatus) -> Result<Vec<LobbyState>, AppError> {
        let all_states = self.get_all(None).await?;

        Ok(all_states
            .into_iter()
            .filter(|state| state.status == status)
            .collect())
    }

    /// Batch fetch lobby states
    pub async fn get_states_batch(
        &self,
        lobby_ids: &[Uuid],
    ) -> Result<Vec<(Uuid, Option<LobbyState>)>, AppError> {
        if lobby_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;

        // Build pipeline with HGETALL commands for each lobby
        let mut pipe = redis::pipe();
        for lobby_id in lobby_ids {
            let key = RedisKey::lobby_state(*lobby_id);
            pipe.hgetall(&key);
        }

        // Execute pipeline - get all results in one round-trip
        let results: Vec<HashMap<String, String>> = pipe
            .query_async(&mut *conn)
            .await
            .map_err(AppError::RedisCommandError)?;

        // Parse results
        let mut states = Vec::with_capacity(lobby_ids.len());
        for (lobby_id, map) in lobby_ids.iter().zip(results.into_iter()) {
            if map.is_empty() {
                states.push((*lobby_id, None));
            } else {
                match LobbyState::from_redis_hash(&map) {
                    Ok(state) => states.push((*lobby_id, Some(state))),
                    Err(_) => states.push((*lobby_id, None)),
                }
            }
        }

        Ok(states)
    }
}
