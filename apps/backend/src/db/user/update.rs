use crate::{
    errors::AppError,
    models::db::{User, Username},
};
use uuid::Uuid;

use super::UserRepository;

impl UserRepository {
    /// Update a user's username.
    /// DB constraint (CITEXT UNIQUE) enforces uniqueness automatically.
    pub async fn update_username(
        &self,
        user_id: Uuid,
        username: &Username,
    ) -> Result<User, AppError> {
        sqlx::query(
            "UPDATE users
            SET username = $1, updated_at = NOW()
            WHERE id = $2",
        )
        .bind(username)
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

        tracing::info!("Updated username for user {}: {}", user_id, username);
        self.find_by_id(user_id).await
    }

    /// Update a user's display name.
    pub async fn update_display_name(
        &self,
        user_id: Uuid,
        display_name: &str,
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
        self.find_by_id(user_id).await
    }

    /// Update a user's trust rating.
    pub async fn update_trust_rating(
        &self,
        user_id: Uuid,
        trust_rating: f64,
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
        self.find_by_id(user_id).await
    }

    /// Partially update a user's profile (only provided fields are changed).
    pub async fn update_profile(
        &self,
        user_id: Uuid,
        username: Option<&Username>,
        display_name: Option<&str>,
    ) -> Result<User, AppError> {
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

        if let Some(uname) = username {
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
        self.find_by_id(user_id).await
    }

    /// Increment a user's trust rating.
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

    /// Decrement a user's trust rating.
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
