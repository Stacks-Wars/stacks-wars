use crate::{auth::generate_jwt, errors::AppError, models::db::UserV2};

use super::UserRepository;

impl UserRepository {
    /// Create a new user or return an existing user's JWT token.
    pub async fn create_user(
        &self,
        wallet_address: String,
        jwt_secret: &str,
    ) -> Result<String, AppError> {
        // Check if user already exists
        if let Ok(user) = self.find_by_wallet(&wallet_address).await {
            tracing::info!("User already exists: {}", user.id);
            let token = generate_jwt(&user, jwt_secret)?;
            return Ok(token);
        }

        // Create new user
        let trust_rating = 10.0;

        // Insert user and return generated data
        let user = sqlx::query_as::<_, UserV2>(
            "INSERT INTO users (wallet_address, trust_rating)
            VALUES ($1, $2)
            RETURNING id, wallet_address, username, display_name, trust_rating, created_at, updated_at",
        )
        .bind(&wallet_address)
        .bind(trust_rating)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create user: {}", e)))?;

        let token = generate_jwt(&user, jwt_secret)?;
        tracing::info!("Created new user: {}", user.id);

        Ok(token)
    }

    /// Create a new user (returns the created `UserV2`).
    pub async fn create_user_with_details(
        &self,
        wallet_address: String,
        username: Option<String>,
        display_name: Option<String>,
    ) -> Result<UserV2, AppError> {
        // Check if user already exists
        if self.find_by_wallet(&wallet_address).await.is_ok() {
            return Err(AppError::BadRequest(
                "User with this wallet address already exists".into(),
            ));
        }

        let trust_rating = 10.0;

        // Insert user with optional fields and return generated data
        let user = sqlx::query_as::<_, UserV2>(
            "INSERT INTO users (wallet_address, username, display_name, trust_rating)
            VALUES ($1, $2, $3, $4)
            RETURNING id, wallet_address, username, display_name, trust_rating, created_at, updated_at",
        )
        .bind(&wallet_address)
        .bind(&username)
        .bind(&display_name)
        .bind(trust_rating)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create user: {}", e)))?;

        tracing::info!("Created user with details: {}", user.id);
        Ok(user)
    }
}
