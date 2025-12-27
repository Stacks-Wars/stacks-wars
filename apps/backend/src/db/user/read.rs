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
}
