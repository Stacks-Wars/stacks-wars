//! Read operations for PlayerState

use crate::db::player_state::PlayerStateRepository;
use crate::errors::AppError;
use crate::models::redis::PlayerState;
use redis::AsyncCommands;
use std::collections::HashMap;
use uuid::Uuid;

impl PlayerStateRepository {
    /// Get player state by user ID and lobby ID
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `user_id` - The user UUID
    ///
    /// # Returns
    /// * `Ok(PlayerState)` if found
    /// * `Err(AppError::NotFound)` if not found
    pub async fn get_state(&self, lobby_id: Uuid, user_id: Uuid) -> Result<PlayerState, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:players:{}", lobby_id, user_id);

        let map: HashMap<String, String> = conn
            .hgetall(&key)
            .await
            .map_err(AppError::RedisCommandError)?;

        if map.is_empty() {
            return Err(AppError::NotFound(format!(
                "Player state for user {} in lobby {} not found",
                user_id, lobby_id
            )));
        }

        PlayerState::from_redis_hash(&map)
    }

    /// Check if player state exists
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `user_id` - The user UUID
    ///
    /// # Returns
    /// * `Ok(true)` if exists
    /// * `Ok(false)` if not found
    pub async fn exists(&self, lobby_id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:players:{}", lobby_id, user_id);

        conn.exists(&key).await.map_err(AppError::RedisCommandError)
    }

    /// Get all player states in a lobby
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(Vec<PlayerState>)` - List of player states (empty if lobby has no players)
    pub async fn get_all_in_lobby(&self, lobby_id: Uuid) -> Result<Vec<PlayerState>, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let pattern = format!("lobbies:{}:players:*", lobby_id);

        let keys: Vec<String> = conn
            .keys(&pattern)
            .await
            .map_err(AppError::RedisCommandError)?;

        let mut states = Vec::new();

        for key in keys {
            let map: HashMap<String, String> = conn
                .hgetall(&key)
                .await
                .map_err(AppError::RedisCommandError)?;

            if !map.is_empty() {
                if let Ok(state) = PlayerState::from_redis_hash(&map) {
                    states.push(state);
                }
            }
        }

        Ok(states)
    }

    /// Count players in a lobby
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of players
    pub async fn count_players(&self, lobby_id: Uuid) -> Result<usize, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let pattern = format!("lobbies:{}:players:*", lobby_id);

        let keys: Vec<String> = conn
            .keys(&pattern)
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(keys.len())
    }

    /// Get players with prizes (winners)
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(Vec<PlayerState>)` - List of player states with prizes
    pub async fn get_winners(&self, lobby_id: Uuid) -> Result<Vec<PlayerState>, AppError> {
        let all_players = self.get_all_in_lobby(lobby_id).await?;

        Ok(all_players.into_iter().filter(|p| p.has_prize()).collect())
    }

    /// Get players sorted by rank
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(Vec<PlayerState>)` - List of player states sorted by rank (1st, 2nd, 3rd...)
    pub async fn get_ranked_players(&self, lobby_id: Uuid) -> Result<Vec<PlayerState>, AppError> {
        let mut all_players = self.get_all_in_lobby(lobby_id).await?;

        // Sort by rank (None values go to the end)
        all_players.sort_by(|a, b| match (a.rank, b.rank) {
            (Some(rank_a), Some(rank_b)) => rank_a.cmp(&rank_b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });

        Ok(all_players)
    }

    /// Get player IDs in a lobby (lightweight - just IDs)
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    ///
    /// # Returns
    /// * `Ok(Vec<Uuid>)` - List of player UUIDs
    pub async fn get_player_ids(&self, lobby_id: Uuid) -> Result<Vec<Uuid>, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let pattern = format!("lobbies:{}:players:*", lobby_id);

        let keys: Vec<String> = conn
            .keys(&pattern)
            .await
            .map_err(AppError::RedisCommandError)?;

        let mut player_ids = Vec::new();

        for key in keys {
            // Extract user_id from key: lobbies:{lobby_id}:players:{user_id}
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() == 4 {
                if let Ok(user_id) = Uuid::parse_str(parts[3]) {
                    player_ids.push(user_id);
                }
            }
        }

        Ok(player_ids)
    }
}
