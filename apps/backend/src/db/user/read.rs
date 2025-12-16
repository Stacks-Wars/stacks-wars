use crate::{
    errors::AppError,
    models::{User, Username, WalletAddress},
};
use uuid::Uuid;

use super::UserRepository;

impl UserRepository {
    /// Find a user by ID (returns user profile data).
    pub async fn find_by_id(&self, user_id: Uuid) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, wallet_address, username, display_name, email, email_verified, trust_rating, created_at, updated_at
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

        tracing::info!("Found user by id: {}", user.id);

        Ok(user)
    }

    /// Find a user by wallet address.
    pub async fn find_by_wallet(&self, wallet_address: &str) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, wallet_address, username, display_name, email, email_verified, trust_rating, created_at, updated_at
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

        tracing::info!("Found user by wallet: {}", user.id);

        Ok(user)
    }

    /// Find a user by username (case-insensitive).
    pub async fn find_by_username(&self, username: &str) -> Result<User, AppError> {
        let normalized_username = username.to_lowercase();

        let user = sqlx::query_as::<_, User>(
            "SELECT id, wallet_address, username, display_name, email, email_verified, trust_rating, created_at, updated_at
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

        tracing::info!("Found user by username: {}", user.id);

        Ok(user)
    }

    /// Find a user ID by wallet address or username.
    pub async fn find_user_id(&self, identifier: &str) -> Result<Uuid, AppError> {
        // Validate identifier format first to avoid unnecessary DB queries
        // Try wallet address if format is valid
        if let Ok(wallet) = WalletAddress::new(identifier) {
            if let Ok(Some(user_id)) =
                sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE wallet_address = $1")
                    .bind(wallet.as_str())
                    .fetch_optional(&self.pool)
                    .await
            {
                tracing::debug!("Found user by wallet: {}", user_id);
                return Ok(user_id);
            }
        }

        // Fallback to username lookup if format is valid
        if let Ok(username) = Username::new(identifier) {
            let normalized_username = username.as_str().to_lowercase();
            if let Ok(Some(user_id)) =
                sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE LOWER(username) = $1")
                    .bind(&normalized_username)
                    .fetch_optional(&self.pool)
                    .await
            {
                tracing::debug!("Found user by username: {}", user_id);
                return Ok(user_id);
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
