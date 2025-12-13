use crate::{
    errors::AppError,
    models::{User, Username},
};
use uuid::Uuid;

use super::UserRepository;
use crate::db::player_state::PlayerStateRepository;
use crate::state::RedisClient;

impl UserRepository {
    /// Update a user's username.
    /// Validates username internally. DB constraint (CITEXT UNIQUE) enforces uniqueness.
    pub async fn update_username(
        &self,
        user_id: Uuid,
        username: &str,
        redis: RedisClient,
    ) -> Result<User, AppError> {
        let username = Username::new(username)?;
        sqlx::query(
            "UPDATE users
            SET username = $1, updated_at = NOW()
            WHERE id = $2",
        )
        .bind(&username)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return AppError::BadRequest("Username already taken".into());
                }
            }
            AppError::DatabaseError(format!("Failed to update username: {}", e))
        })?;

        // Sync username across all lobbies in Redis
        let player_repo = PlayerStateRepository::new(redis);
        let uname = username.as_ref().to_string();
        tokio::spawn(async move {
            if let Err(e) = player_repo
                .sync_user_profile_across_lobbies(user_id, None, Some(&uname), None, None)
                .await
            {
                tracing::warn!(
                    "Failed to sync username for user {} across lobbies: {}",
                    user_id,
                    e
                );
            }
        });

        self.find_by_id(user_id).await
    }

    /// Update a user's display name.
    pub async fn update_display_name(
        &self,
        user_id: Uuid,
        display_name: &str,
        redis: RedisClient,
    ) -> Result<User, AppError> {
        sqlx::query(
            "UPDATE users
            SET display_name = $1, updated_at = NOW()
            WHERE id = $2",
        )
        .bind(display_name)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update display name: {}", e)))?;

        tracing::info!(
            "Updated display name for user {}: {}",
            user_id,
            display_name
        );

        // Sync display name across all lobbies in Redis
        let player_repo = PlayerStateRepository::new(redis);
        let dname = display_name.to_string();
        tokio::spawn(async move {
            if let Err(e) = player_repo
                .sync_user_profile_across_lobbies(user_id, None, None, Some(&dname), None)
                .await
            {
                tracing::warn!(
                    "Failed to sync display name for user {} across lobbies: {}",
                    user_id,
                    e
                );
            }
        });

        self.find_by_id(user_id).await
    }

    /// Update a user's trust rating.
    pub async fn update_trust_rating(
        &self,
        user_id: Uuid,
        trust_rating: f64,
        redis: RedisClient,
    ) -> Result<User, AppError> {
        sqlx::query(
            "UPDATE users
            SET trust_rating = $1, updated_at = NOW()
            WHERE id = $2",
        )
        .bind(trust_rating)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update trust rating: {}", e)))?;

        tracing::info!(
            "Updated trust rating for user {}: {}",
            user_id,
            trust_rating
        );

        // Sync trust rating across all lobbies in Redis
        let player_repo = PlayerStateRepository::new(redis);
        tokio::spawn(async move {
            if let Err(e) = player_repo
                .sync_user_profile_across_lobbies(user_id, None, None, None, Some(trust_rating))
                .await
            {
                tracing::warn!(
                    "Failed to sync trust rating for user {} across lobbies: {}",
                    user_id,
                    e
                );
            }
        });

        self.find_by_id(user_id).await
    }

    /// Partially update a user's profile (only provided fields are changed).
    pub async fn update_profile(
        &self,
        user_id: Uuid,
        username: Option<&str>,
        display_name: Option<&str>,
        redis: RedisClient,
    ) -> Result<User, AppError> {
        // Validate username if provided
        let username = if let Some(uname) = username {
            Some(Username::new(uname)?)
        } else {
            None
        };

        // Build dynamic update query
        let mut query = String::from("UPDATE users SET updated_at = NOW()");
        let mut param_count = 1;

        if username.is_some() {
            query.push_str(&format!(", username = ${}", param_count));
            param_count += 1;
        }
        if display_name.is_some() {
            query.push_str(&format!(", display_name = ${}", param_count));
            param_count += 1;
        }

        query.push_str(&format!(" WHERE id = ${}", param_count));

        let mut query_builder = sqlx::query(&query);

        if let Some(ref uname) = username {
            query_builder = query_builder.bind(uname);
        }
        if let Some(dname) = display_name {
            query_builder = query_builder.bind(dname);
        }

        query_builder = query_builder.bind(user_id);

        query_builder.execute(&self.pool).await.map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return AppError::BadRequest("Username already taken".into());
                }
            }
            AppError::DatabaseError(format!("Failed to update profile: {}", e))
        })?;

        tracing::info!("Updated profile for user {}", user_id);

        // Sync updated fields across all lobbies in Redis
        let player_repo = PlayerStateRepository::new(redis);
        let uname_owned = username.as_ref().map(|u| u.as_ref().to_string());
        let dname_owned = display_name.map(|d| d.to_string());
        tokio::spawn(async move {
            if let Err(e) = player_repo
                .sync_user_profile_across_lobbies(
                    user_id,
                    None,
                    uname_owned.as_deref(),
                    dname_owned.as_deref(),
                    None,
                )
                .await
            {
                tracing::warn!(
                    "Failed to sync profile for user {} across lobbies: {}",
                    user_id,
                    e
                );
            }
        });

        self.find_by_id(user_id).await
    }

    /// Increment a user's trust rating.
    pub async fn increment_trust_rating(
        &self,
        user_id: Uuid,
        amount: f64,
        redis: RedisClient,
    ) -> Result<f64, AppError> {
        let new_rating = sqlx::query_scalar::<_, f64>(
            "UPDATE users
            SET trust_rating = trust_rating + $1, updated_at = NOW()
            WHERE id = $2
            RETURNING trust_rating",
        )
        .bind(amount)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to increment trust rating: {}", e)))?;

        tracing::info!(
            "Incremented trust rating for user {} by {} to {}",
            user_id,
            amount,
            new_rating
        );

        // Sync trust rating across all lobbies in Redis
        let player_repo = PlayerStateRepository::new(redis);
        tokio::spawn(async move {
            if let Err(e) = player_repo
                .sync_user_profile_across_lobbies(user_id, None, None, None, Some(new_rating))
                .await
            {
                tracing::warn!(
                    "Failed to sync trust rating for user {} across lobbies: {}",
                    user_id,
                    e
                );
            }
        });

        Ok(new_rating)
    }

    /// Decrement a user's trust rating.
    pub async fn decrement_trust_rating(
        &self,
        user_id: Uuid,
        amount: f64,
        redis: RedisClient,
    ) -> Result<f64, AppError> {
        let new_rating = sqlx::query_scalar::<_, f64>(
            "UPDATE users
            SET trust_rating = trust_rating - $1, updated_at = NOW()
            WHERE id = $2
            RETURNING trust_rating",
        )
        .bind(amount)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to decrement trust rating: {}", e)))?;

        tracing::info!(
            "Decremented trust rating for user {} by {} to {}",
            user_id,
            amount,
            new_rating
        );

        // Sync trust rating across all lobbies in Redis
        let player_repo = PlayerStateRepository::new(redis);
        tokio::spawn(async move {
            if let Err(e) = player_repo
                .sync_user_profile_across_lobbies(user_id, None, None, None, Some(new_rating))
                .await
            {
                tracing::warn!(
                    "Failed to sync trust rating for user {} across lobbies: {}",
                    user_id,
                    e
                );
            }
        });

        Ok(new_rating)
    }
}
