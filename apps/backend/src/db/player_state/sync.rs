// Sync operations for PlayerState - update user data across all lobbies

use crate::db::player_state::PlayerStateRepository;
use crate::errors::AppError;
use crate::models::keys::{KeyPart, RedisKey};
use redis::AsyncCommands;
use uuid::Uuid;

impl PlayerStateRepository {
    /// Update user profile fields across all lobbies where the user is a player.
    ///
    /// This is called when a user updates their profile (username, display_name, trust_rating).
    /// It efficiently updates the denormalized user data in all PlayerState records
    /// to keep the UI data fresh without additional database queries.
    ///
    /// Uses Redis SCAN + HSET to update all matching keys in a single operation.
    pub async fn sync_user_profile_across_lobbies(
        &self,
        user_id: Uuid,
        wallet_address: Option<&str>,
        username: Option<&str>,
        display_name: Option<&str>,
        trust_rating: Option<f64>,
    ) -> Result<usize, AppError> {
        let mut conn =
            self.redis.get().await.map_err(|e| {
                AppError::RedisError(format!("Failed to get Redis connection: {}", e))
            })?;

        // Pattern to match all lobbies where this user is a player
        // lobbies:*:players:{user_id}
        let pattern = RedisKey::lobby_player(KeyPart::Wildcard, user_id);

        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(AppError::RedisCommandError)?;

        if keys.is_empty() {
            return Ok(0);
        }

        // Build updates once to reuse for all keys
        let mut updates: Vec<(&str, String)> = Vec::new();

        if let Some(addr) = wallet_address {
            updates.push(("wallet_address", addr.to_string()));
        }
        if let Some(uname) = username {
            updates.push(("username", uname.to_string()));
        }
        if let Some(dname) = display_name {
            updates.push(("display_name", dname.to_string()));
        }
        if let Some(rating) = trust_rating {
            updates.push(("trust_rating", rating.to_string()));
        }

        if updates.is_empty() {
            return Ok(0);
        }

        // Update timestamp
        updates.push(("updated_at", chrono::Utc::now().timestamp().to_string()));

        // Use pipeline to batch all updates - use atomic() for pooled connections
        let mut pipe = redis::pipe();
        pipe.atomic();
        for key in &keys {
            pipe.hset_multiple(key, &updates);
        }

        let _: Vec<()> = pipe
            .query_async(&mut *conn)
            .await
            .map_err(AppError::RedisCommandError)?;

        let updated_count = keys.len();

        tracing::info!(
            "Synced user {} profile across {} lobbies in single pipeline",
            user_id,
            updated_count
        );

        Ok(updated_count)
    }
}
