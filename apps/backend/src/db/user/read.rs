use std::collections::HashMap;

use crate::{
    errors::AppError,
    models::{LobbyInfo, User, Username, WalletAddress, keys::KeyPart, player_state::{ClaimState, PlayerState}}, state::RedisClient,
};
use crate::db::{game::GameRepository, lobby::LobbyRepository};
use crate::models::{LobbyExtended, LobbyState, keys::RedisKey};
use redis::AsyncCommands;
use uuid::Uuid;

use super::UserRepository;

impl UserRepository {
    /// Find a user by ID (returns user profile data).
    pub async fn find_by_id(&self, user_id: Uuid) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, wallet_address, username, display_name, email, email_verified, trust_rating, profile_image, created_at, updated_at
            FROM users
            WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to query user by id: {}", e);
            AppError::DatabaseError(format!("Failed to query user: {}", e))})?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        tracing::debug!("Found user by id: {}", user.id);

        Ok(user)
    }

    /// Find a user by wallet address.
    pub async fn find_by_wallet(&self, wallet_address: &str) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, wallet_address, username, display_name, email, email_verified, trust_rating, profile_image, created_at, updated_at
            FROM users
            WHERE wallet_address = $1",
        )
        .bind(wallet_address)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to query user by wallet: {}", e);
            AppError::DatabaseError(format!("Failed to query user by wallet: {}", e))
        })?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        tracing::debug!("Found user by wallet: {}", user.id);

        Ok(user)
    }

    /// Find a user by username (case-insensitive).
    pub async fn find_by_username(&self, username: &str) -> Result<User, AppError> {
        let normalized_username = username.to_lowercase();

        let user = sqlx::query_as::<_, User>(
            "SELECT id, wallet_address, username, display_name, email, email_verified, trust_rating, profile_image, created_at, updated_at
            FROM users
            WHERE LOWER(username) = $1",
        )
        .bind(&normalized_username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to query user by username: {}", e);
            AppError::DatabaseError(format!("Failed to query user by username: {}", e))
        })?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        tracing::debug!("Found user by username: {}", user.id);

        Ok(user)
    }

    /// Find a user by UUID, wallet address, or username.
    pub async fn find_user(&self, identifier: &str) -> Result<User, AppError> {
        // Try parsing as UUID first
        if let Ok(user_id) = Uuid::parse_str(identifier) {
            if let Ok(user) = self.find_by_id(user_id).await {
                tracing::debug!("Found user by UUID: {}", user.id);
                return Ok(user);
            }
        }

        // Try wallet address if format is valid
        if let Ok(wallet) = WalletAddress::new(identifier) {
            if let Ok(user) = self.find_by_wallet(wallet.as_str()).await {
                tracing::debug!("Found user by wallet: {}", user.id);
                return Ok(user);
            }
        }

        // Fallback to username lookup if format is valid
        if let Ok(username) = Username::new(identifier) {
            if let Ok(user) = self.find_by_username(username.as_str()).await {
                tracing::debug!("Found user by username: {}", user.id);
                return Ok(user);
            }
        }

        tracing::debug!("User not found for identifier: {}", identifier);
        Err(AppError::NotFound(format!(
            "User not found for identifier: {}",
            identifier
        )))
    }

    /// Check if a user exists by ID (lightweight).
    pub async fn exists_by_id(&self, user_id: Uuid) -> Result<bool, AppError> {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to check user existence: {}", e))
                })?;

        Ok(exists)
    }

    /// Get unclaimed rewards for a user.
    /// Returns a list of (LobbyInfo, prize) for lobbies where the user has unclaimed prizes.
    pub async fn get_unclaimed_rewards(
        &self,
        user_id: Uuid,
        redis: &RedisClient,
    ) -> Result<Vec<(LobbyInfo, f64)>, AppError> {
        let mut conn = redis.get().await.map_err(|e| {
            AppError::RedisError(format!("Failed to get Redis connection: {}", e))
        })?;

        // Scan for player state keys matching lobbies:*:players:{user_id}
        let pattern = RedisKey::lobby_player(KeyPart::Wildcard, KeyPart::Id(user_id));
        let keys: Vec<String> = conn.keys(&pattern).await.map_err(|e| {
            AppError::RedisError(format!("Failed to scan player keys: {}", e))
        })?;

        let mut unclaimed = Vec::new();

        for key in keys {
            // Get player state
            let player_data: HashMap<String, String> = conn.hgetall(&key).await.map_err(|e| {
                AppError::RedisError(format!("Failed to get player state: {}", e))
            })?;

            let player_state = PlayerState::from_redis_hash(&player_data)?;

            // Check if unclaimed and has prize
            if matches!(player_state.claim_state, Some(ClaimState::NotClaimed))
                && player_state.prize.unwrap_or(0.0) > 0.0 {

                let lobby_id = player_state.lobby_id;

                // Fetch lobby info in parallel
                let lobby_repo = LobbyRepository::new(self.pool.clone());
                let game_repo = GameRepository::new(self.pool.clone());

                let lobby = lobby_repo.find_by_id(lobby_id).await?;
                let lobby_state_key = RedisKey::lobby_state(lobby_id);
                let (game, creator, state_data) = tokio::join!(
                    game_repo.find_by_id(lobby.game_id),
                    self.find_by_id(lobby.creator_id),
                    async { conn.hgetall(&lobby_state_key).await.ok() }
                );

                let game = game?;
                let creator = creator?;

                let state = state_data.map(|data| LobbyState::from_redis_hash(&data).unwrap_or_else(|_| LobbyState::new(lobby_id)))
                    .unwrap_or_else(|| LobbyState::new(lobby_id));

                let extended = LobbyExtended::from_parts(lobby, state);
                let lobby_info = LobbyInfo {
                    lobby: extended,
                    game,
                    creator,
                };

                unclaimed.push((lobby_info, player_state.prize.unwrap()));
            }
        }

        Ok(unclaimed)
    }
}
