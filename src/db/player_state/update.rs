//! Update operations for PlayerState

use crate::db::player_state::PlayerStateRepository;
use crate::errors::AppError;
use crate::models::redis::player_state::{ClaimState, PlayerStatus};
use chrono::Utc;
use redis::AsyncCommands;
use uuid::Uuid;

impl PlayerStateRepository {
    /// Update player status
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `user_id` - The user UUID
    /// * `status` - The new status
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn update_status(
        &self,
        lobby_id: Uuid,
        user_id: Uuid,
        status: PlayerStatus,
    ) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:players:{}", lobby_id, user_id);

        // Check if exists
        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(AppError::RedisCommandError)?;

        if !exists {
            return Err(AppError::NotFound(format!(
                "Player state for user {} in lobby {} not found",
                user_id, lobby_id
            )));
        }

        // Update status and updated_at
        let now = Utc::now().timestamp();
        let status_str = format!("{:?}", status);

        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("status", status_str.as_str()),
                    ("updated_at", &now.to_string()),
                ],
            )
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Set player rank and prize (for winners)
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `user_id` - The user UUID
    /// * `rank` - The player's rank (1st, 2nd, 3rd, etc.)
    /// * `prize` - The prize amount (if any)
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn set_result(
        &self,
        lobby_id: Uuid,
        user_id: Uuid,
        rank: usize,
        prize: Option<f64>,
    ) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:players:{}", lobby_id, user_id);

        let now = Utc::now().timestamp();

        let mut fields = vec![("rank", rank.to_string()), ("updated_at", now.to_string())];

        if let Some(prize_amount) = prize {
            fields.push(("prize", prize_amount.to_string()));
            fields.push(("claim_state", "Unclaimed".to_string()));
        }

        let fields_ref: Vec<(&str, &str)> = fields
            .iter()
            .map(|(k, v)| (k.as_ref(), v.as_ref()))
            .collect();

        let _: () = conn
            .hset_multiple(&key, &fields_ref)
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Set player rank only
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `user_id` - The user UUID
    /// * `rank` - The player's rank
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn set_rank(
        &self,
        lobby_id: Uuid,
        user_id: Uuid,
        rank: usize,
    ) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:players:{}", lobby_id, user_id);

        let now = Utc::now().timestamp();

        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("rank", &rank.to_string()),
                    ("updated_at", &now.to_string()),
                ],
            )
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Set player prize
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `user_id` - The user UUID
    /// * `prize` - The prize amount
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn set_prize(
        &self,
        lobby_id: Uuid,
        user_id: Uuid,
        prize: f64,
    ) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:players:{}", lobby_id, user_id);

        let now = Utc::now().timestamp();

        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("prize", prize.to_string().as_str()),
                    ("claim_state", "Unclaimed"),
                    ("updated_at", now.to_string().as_str()),
                ],
            )
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Mark prize as claimed
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `user_id` - The user UUID
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn mark_claimed(&self, lobby_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:players:{}", lobby_id, user_id);

        let now = Utc::now().timestamp();

        let _: () = conn
            .hset_multiple(
                &key,
                &[("claim_state", "Claimed"), ("updated_at", &now.to_string())],
            )
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Update claim state
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `user_id` - The user UUID
    /// * `claim_state` - The new claim state
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn update_claim_state(
        &self,
        lobby_id: Uuid,
        user_id: Uuid,
        claim_state: ClaimState,
    ) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:players:{}", lobby_id, user_id);

        let now = Utc::now().timestamp();
        let claim_str = format!("{:?}", claim_state);

        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("claim_state", claim_str.as_str()),
                    ("updated_at", &now.to_string()),
                ],
            )
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Update player's last ping timestamp
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `user_id` - The user UUID
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn update_ping(&self, lobby_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:players:{}", lobby_id, user_id);

        let now_ms = Utc::now().timestamp_millis() as u64;
        let now = Utc::now().timestamp();

        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("last_ping", &now_ms.to_string()),
                    ("updated_at", &now.to_string()),
                ],
            )
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }

    /// Touch the player state (update updated_at timestamp)
    ///
    /// # Arguments
    /// * `lobby_id` - The lobby UUID
    /// * `user_id` - The user UUID
    ///
    /// # Returns
    /// * `Ok(())` if successful
    pub async fn touch(&self, lobby_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = format!("lobbies:{}:players:{}", lobby_id, user_id);

        let now = Utc::now().timestamp();

        let _: () = conn
            .hset(&key, "updated_at", now)
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }
}
