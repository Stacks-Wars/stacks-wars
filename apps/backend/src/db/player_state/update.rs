// Update operations for PlayerState (Redis)

use crate::db::player_state::PlayerStateRepository;
use crate::errors::AppError;
use crate::models::RedisKey;
use crate::models::player_state::{ClaimState, PlayerStatus};
use chrono::Utc;
use redis::AsyncCommands;
use uuid::Uuid;

impl PlayerStateRepository {
    /// Update a player's status in Redis.
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
        let key = RedisKey::lobby_player(lobby_id, user_id);

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

    /// Set player rank and prize (for winners).
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
        let key = RedisKey::lobby_player(lobby_id, user_id);

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

    /// Set a player's rank only.
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
        let key = RedisKey::lobby_player(lobby_id, user_id);

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

    /// Set a player's prize and mark as unclaimed.
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
        let key = RedisKey::lobby_player(lobby_id, user_id);

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

    /// Mark a player's prize as claimed.
    pub async fn mark_claimed(&self, lobby_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_player(lobby_id, user_id);

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

    /// Update a player's claim state.
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
        let key = RedisKey::lobby_player(lobby_id, user_id);

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

    /// Update a player's last ping timestamp.
    pub async fn update_ping(&self, lobby_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_player(lobby_id, user_id);

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

    /// Touch the player state (refresh updated_at timestamp).
    pub async fn touch(&self, lobby_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;
        let key = RedisKey::lobby_player(lobby_id, user_id);

        let now = Utc::now().timestamp();

        let _: () = conn
            .hset(&key, "updated_at", now)
            .await
            .map_err(AppError::RedisCommandError)?;

        Ok(())
    }
}
