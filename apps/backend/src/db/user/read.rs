use crate::{errors::AppError, models::User};
use uuid::Uuid;

use super::UserRepository;

impl UserRepository {
    /// Find a user by ID (returns user profile data).
    pub async fn find_by_id(&self, user_id: Uuid) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, wallet_address, username, display_name, trust_rating, created_at, updated_at
            FROM users
            WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to query user: {}", e)))?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        Ok(user)
    }

    /// Find a user by wallet address.
    pub async fn find_by_wallet(&self, wallet_address: &str) -> Result<User, AppError> {
        let user_id =
            sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE wallet_address = $1")
                .bind(wallet_address)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to query user by wallet: {}", e))
                })?
                .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        self.find_by_id(user_id).await
    }

    /// Find a user by username (case-insensitive).
    pub async fn find_by_username(&self, username: &str) -> Result<User, AppError> {
        let normalized_username = username.to_lowercase();

        let user_id =
            sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE LOWER(username) = $1")
                .bind(&normalized_username)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to query user by username: {}", e))
                })?
                .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        self.find_by_id(user_id).await
    }

    /// Find a user ID by wallet address or username.
    pub async fn find_user_id(&self, identifier: &str) -> Result<Uuid, AppError> {
        // Try wallet address first
        if let Ok(user_id) =
            sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE wallet_address = $1")
                .bind(identifier)
                .fetch_optional(&self.pool)
                .await
        {
            if let Some(id) = user_id {
                tracing::debug!("Found user by wallet: {}", id);
                return Ok(id);
            }
        }

        // Fallback to username lookup
        let normalized_username = identifier.to_lowercase();
        let user_id =
            sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE LOWER(username) = $1")
                .bind(&normalized_username)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to query user by username: {}", e))
                })?;

        match user_id {
            Some(id) => {
                tracing::debug!("Found user by username: {}", id);
                Ok(id)
            }
            None => {
                tracing::debug!("User not found for identifier: {}", identifier);
                Err(AppError::NotFound(format!(
                    "User not found for identifier: {}",
                    identifier
                )))
            }
        }
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

    /// Check if a wallet address is already registered.
    pub async fn exists_by_wallet(&self, wallet_address: &str) -> Result<bool, AppError> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM users WHERE wallet_address = $1)",
        )
        .bind(wallet_address)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to check wallet existence: {}", e)))?;

        Ok(exists)
    }

    /// Check if a username is already taken (case-insensitive).
    pub async fn exists_by_username(&self, username: &str) -> Result<bool, AppError> {
        let normalized_username = username.to_lowercase();

        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM users WHERE LOWER(username) = $1)",
        )
        .bind(&normalized_username)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to check username existence: {}", e))
        })?;

        Ok(exists)
    }
}
