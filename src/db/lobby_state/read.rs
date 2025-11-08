//! Read operations for LobbyState

use crate::db::lobby_state::LobbyStateRepository;
use crate::errors::AppError;
use crate::models::enums::LobbyState as LobbyStatus;
use crate::models::redis::LobbyState;
use redis::AsyncCommands;
use std::collections::HashMap;
use uuid::Uuid;

impl LobbyStateRepository {
    /// Get lobby state by ID
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(LobbyState)` if found
    /// * `Err(AppError::NotFound)` if not found
    pub async fn get_state(&self, lobby_id: Uuid) -> Result<LobbyState, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", lobby_id);

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

    /// Check if lobby state exists
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(true)` if exists
    /// * `Ok(false)` if not found
    pub async fn exists(&self, lobby_id: Uuid) -> Result<bool, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", lobby_id);

        conn.exists(&key).await.map_err(AppError::RedisCommandError)
    }

    /// Get current status of a lobby
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(LobbyStatus)` if found
    /// * `Err(AppError::NotFound)` if not found
    pub async fn get_status(&self, lobby_id: Uuid) -> Result<LobbyStatus, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", lobby_id);

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

    /// Get participant count
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of participants
    /// * `Err(AppError)` if lobby not found
    pub async fn get_participant_count(&self, lobby_id: Uuid) -> Result<usize, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:state", lobby_id);

        let count: Option<usize> = conn
            .hget(&key, "participant_count")
            .await
            .map_err(AppError::RedisCommandError)?;

        count.ok_or_else(|| AppError::NotFound(format!("Lobby state {} not found", lobby_id)))
    }

    /// Get all lobby states (for admin/debugging)
    ///
    /// # Arguments
    /// * `limit` - Maximum number of lobbies to return (None for all)
    ///
    /// # Returns
    /// * `Ok(Vec<LobbyState>)` - List of lobby states
    pub async fn get_all(&self, limit: Option<usize>) -> Result<Vec<LobbyState>, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let pattern = "lobbies:*:state";

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

    /// Get lobbies by status
    ///
    /// # Arguments
    /// * `status` - The status to filter by
    ///
    /// # Returns
    /// * `Ok(Vec<LobbyState>)` - List of lobby states with the given status
    pub async fn get_by_status(&self, status: LobbyStatus) -> Result<Vec<LobbyState>, AppError> {
        let all_states = self.get_all(None).await?;

        Ok(all_states
            .into_iter()
            .filter(|state| state.status == status)
            .collect())
    }
}
