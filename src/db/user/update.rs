use crate::{errors::AppError, models::db::UserV2};
use uuid::Uuid;

use super::UserRepository;

impl UserRepository {
    /// Update a user's username
    ///
    /// Username must be unique (case-insensitive check).
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user
    /// * `username` - New username
    ///
    /// # Returns
    /// * `Ok(UserV2)` - Updated user
    /// * `Err(AppError::BadRequest)` - Username already taken
    /// * `Err(AppError::NotFound)` - User doesn't exist
    pub async fn update_username(
        &self,
        user_id: Uuid,
        username: &String,
    ) -> Result<UserV2, AppError> {
        // Check if username is already taken by another user
        if let Ok(existing_id) = self.find_user_id(&username).await {
            if existing_id != user_id {
                return Err(AppError::BadRequest("Username already taken".into()));
            }
        }

        sqlx::query(
            "UPDATE users
            SET username = $1, updated_at = NOW()
            WHERE id = $2",
        )
        .bind(username)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update username: {}", e)))?;

        tracing::info!("Updated username for user {}: {}", user_id, username);
        self.find_by_id(user_id).await
    }

    /// Update a user's display name
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user
    /// * `display_name` - New display name
    ///
    /// # Returns
    /// * `Ok(UserV2)` - Updated user
    /// * `Err(AppError::NotFound)` - User doesn't exist
    pub async fn update_display_name(
        &self,
        user_id: Uuid,
        display_name: &String,
    ) -> Result<UserV2, AppError> {
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
        self.find_by_id(user_id).await
    }

    /// Update a user's trust rating
    ///
    /// Trust rating affects matchmaking and platform privileges.
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user
    /// * `trust_rating` - New trust rating (typically 0.0 - 100.0)
    ///
    /// # Returns
    /// * `Ok(UserV2)` - Updated user
    /// * `Err(AppError::NotFound)` - User doesn't exist
    pub async fn update_trust_rating(
        &self,
        user_id: Uuid,
        trust_rating: f64,
    ) -> Result<UserV2, AppError> {
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
        self.find_by_id(user_id).await
    }

    /// Update a user's profile (username, display_name, trust_rating)
    ///
    /// Only updates fields that are Some(). Use this for partial updates.
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user
    /// * `username` - Optional new username
    /// * `display_name` - Optional new display name
    /// * `trust_rating` - Optional new trust rating
    ///
    /// # Returns
    /// * `Ok(UserV2)` - Updated user
    /// * `Err(AppError::BadRequest)` - Username already taken
    /// * `Err(AppError::NotFound)` - User doesn't exist
    ///
    /// # Examples
    /// ```rust,ignore
    /// // Update only username
    /// let user = repo.update_profile(user_id, Some("alice".into()), None, None).await?;
    ///
    /// // Update multiple fields
    /// let user = repo.update_profile(
    ///     user_id,
    ///     Some("alice".into()),
    ///     Some("Alice Wonderland".into()),
    ///     Some(95.5)
    /// ).await?;
    /// ```
    pub async fn update_profile(
        &self,
        user_id: Uuid,
        username: Option<String>,
        display_name: Option<String>,
        trust_rating: Option<f64>,
    ) -> Result<UserV2, AppError> {
        // Validate username uniqueness if provided
        if let Some(ref uname) = username {
            if let Ok(existing_id) = self.find_user_id(uname).await {
                if existing_id != user_id {
                    return Err(AppError::BadRequest("Username already taken".into()));
                }
            }
        }

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
        if trust_rating.is_some() {
            query.push_str(&format!(", trust_rating = ${}", param_count));
            param_count += 1;
        }

        query.push_str(&format!(" WHERE id = ${}", param_count));

        let mut query_builder = sqlx::query(&query);

        if let Some(uname) = username {
            query_builder = query_builder.bind(uname);
        }
        if let Some(dname) = display_name {
            query_builder = query_builder.bind(dname);
        }
        if let Some(rating) = trust_rating {
            query_builder = query_builder.bind(rating);
        }

        query_builder = query_builder.bind(user_id);

        query_builder
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to update profile: {}", e)))?;

        tracing::info!("Updated profile for user {}", user_id);
        self.find_by_id(user_id).await
    }

    /// Increment a user's trust rating
    ///
    /// Useful for positive feedback systems.
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user
    /// * `amount` - Amount to add to trust rating
    ///
    /// # Returns
    /// * `Ok(f64)` - New trust rating value
    pub async fn increment_trust_rating(
        &self,
        user_id: Uuid,
        amount: f64,
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
        Ok(new_rating)
    }

    /// Decrement a user's trust rating
    ///
    /// Useful for negative feedback systems.
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user
    /// * `amount` - Amount to subtract from trust rating
    ///
    /// # Returns
    /// * `Ok(f64)` - New trust rating value
    pub async fn decrement_trust_rating(
        &self,
        user_id: Uuid,
        amount: f64,
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
        Ok(new_rating)
    }
}
