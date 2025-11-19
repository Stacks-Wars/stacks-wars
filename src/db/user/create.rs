use crate::{auth::generate_jwt, errors::AppError, models::db::UserV2};
use uuid::Uuid;

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
        let user_id = Uuid::new_v4();
        let trust_rating = 10.0;

        // Start a transaction
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to start transaction: {}", e))
            })?;

        // Insert user
        sqlx::query(
            "INSERT INTO users (id, wallet_address, trust_rating)
            VALUES ($1, $2, $3)",
        )
        .bind(user_id)
        .bind(&wallet_address)
        .bind(trust_rating) // Default trust rating
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create user: {}", e)))?;

        // Commit transaction
        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        // Construct user object
        let user = UserV2 {
            id: user_id,
            wallet_address,
            username: None,
            display_name: None,
            trust_rating,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        };

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

        let user_id = Uuid::new_v4();
        let trust_rating = 10.0;

        // Start transaction
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to start transaction: {}", e))
            })?;

        // Insert user with optional fields
        sqlx::query(
            "INSERT INTO users (id, wallet_address, username, display_name, trust_rating)
            VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(user_id)
        .bind(&wallet_address)
        .bind(&username)
        .bind(&display_name)
        .bind(trust_rating)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create user: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        let user = UserV2 {
            id: user_id,
            wallet_address,
            username,
            display_name,
            trust_rating,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        };

        tracing::info!("Created user with details: {}", user.id);
        Ok(user)
    }
}
