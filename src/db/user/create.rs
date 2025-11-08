use crate::{auth::generate_jwt, errors::AppError, models::user::UserV2};
use uuid::Uuid;

use super::UserRepository;

impl UserRepository {
    /// Create a new user or return existing user with JWT token
    ///
    /// This is the primary method for user registration/authentication.
    /// If a user with the given wallet address exists, returns their token.
    /// Otherwise, creates a new user with default values and initializes
    /// their wars points for the current season.
    ///
    /// # Arguments
    /// * `wallet_address` - Stacks wallet address (e.g., "SP2...")
    ///
    /// # Returns
    /// * `Ok(String)` - JWT token for authentication
    /// * `Err(AppError::DatabaseError)` - Database operation failed
    /// * `Err(AppError::BadRequest)` - Invalid wallet format
    ///
    /// # Examples
    /// ```rust,ignore
    /// let token = repo.create_user("SP2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKNRV9EJ7".to_string()).await?;
    /// ```
    pub async fn create_user(&self, wallet_address: String) -> Result<String, AppError> {
        // Check if user already exists
        if let Ok(user) = self.find_by_wallet(&wallet_address).await {
            tracing::info!("User already exists: {}", user.id);
            let token = generate_jwt(&user)?;
            return Ok(token);
        }

        // Create new user
        let user_id = Uuid::new_v4();

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
        .bind(10.0) // Default trust rating
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
            trust_rating: 10.0,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        };

        let token = generate_jwt(&user)?;
        tracing::info!("Created new user: {}", user.id);

        Ok(token)
    }

    /// Create a new user without returning a token
    ///
    /// Useful for administrative user creation or bulk operations.
    ///
    /// # Arguments
    /// * `wallet_address` - Stacks wallet address
    /// * `username` - Optional username
    /// * `display_name` - Optional display name
    /// * `trust_rating` - Initial trust rating (defaults to 10.0)
    ///
    /// # Returns
    /// * `Ok(UserV2)` - Created user with wars points
    /// * `Err(AppError::DatabaseError)` - Database operation failed
    /// * `Err(AppError::BadRequest)` - User already exists
    pub async fn create_user_with_details(
        &self,
        wallet_address: String,
        username: Option<String>,
        display_name: Option<String>,
        trust_rating: Option<f64>,
    ) -> Result<UserV2, AppError> {
        // Check if user already exists
        if self.find_by_wallet(&wallet_address).await.is_ok() {
            return Err(AppError::BadRequest(
                "User with this wallet address already exists".into(),
            ));
        }

        let user_id = Uuid::new_v4();
        let trust_rating = trust_rating.unwrap_or(10.0);

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
