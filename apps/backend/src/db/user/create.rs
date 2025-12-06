use crate::{
    auth::generate_jwt,
    errors::AppError,
    models::db::{User, WalletAddress},
};

use super::UserRepository;

impl UserRepository {
    /// Create a new user or return an existing user's JWT token.
    pub async fn create_user(
        &self,
        wallet_address: &WalletAddress,
        jwt_secret: &str,
    ) -> Result<String, AppError> {
        // Try to insert user
        let result = sqlx::query_as::<_, User>(
            "INSERT INTO users (wallet_address)
            VALUES ($1)
            RETURNING id, wallet_address, username, display_name, trust_rating, created_at, updated_at",
        )
        .bind(&wallet_address)
        .fetch_one(&self.pool)
        .await;

        let user = match result {
            Ok(user) => {
                tracing::info!("Created new user: {}", user.id());
                user
            }
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                // User already exists, fetch and return
                tracing::info!("User already exists, fetching: {}", wallet_address);
                self.find_by_wallet(wallet_address.as_str()).await?
            }
            Err(e) => {
                return Err(AppError::DatabaseError(format!(
                    "Failed to create user: {}",
                    e
                )));
            }
        };

        let token = generate_jwt(&user, jwt_secret)?;
        Ok(token)
    }

    /// Create a new user (returns the created `User`).
    pub async fn create_user_with_details(
        &self,
        wallet_address: &WalletAddress,
        username: Option<&str>,
        display_name: Option<&str>,
    ) -> Result<User, AppError> {
        let trust_rating = 10.0;

        // Try to insert user with optional fields
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (wallet_address, username, display_name, trust_rating)
            VALUES ($1, $2, $3, $4)
            RETURNING id, wallet_address, username, display_name, trust_rating, created_at, updated_at",
        )
        .bind(&wallet_address)
        .bind(username)
        .bind(display_name)
        .bind(trust_rating)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return AppError::BadRequest("User with this wallet address or username already exists".into());
                }
            }
            AppError::DatabaseError(format!("Failed to create user: {}", e))
        })?;

        tracing::info!("Created user with details: {}", user.id());
        Ok(user)
    }
}
