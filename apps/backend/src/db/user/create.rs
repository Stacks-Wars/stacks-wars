use crate::{
    auth::generate_jwt,
    errors::AppError,
    models::{User, WalletAddress},
};
use email_address::EmailAddress;
use std::str::FromStr;

use super::UserRepository;

impl UserRepository {
    /// Create a new user or return an existing user with JWT token.
    pub async fn create_user(
        &self,
        wallet_address: &str,
        email_address: Option<&str>,
        jwt_secret: &str,
    ) -> Result<(User, String), AppError> {
        let wallet_address = WalletAddress::new(wallet_address)?;

        // Handle email address
        let (email, email_verified) = match email_address {
            None => {
                // Construct default email from wallet address
                let default_email = format!("{}@stackswars.com", wallet_address.as_str());
                // Validate the email format
                EmailAddress::from_str(&default_email).map_err(|e| {
                    tracing::error!("Failed to create default email address: {}", default_email);
                    AppError::EmailAddressError(format!("Failed to create default email: {}", e))
                })?;
                (default_email, false)
            }
            Some(email_str) => {
                // Validate provided email
                let email = EmailAddress::from_str(email_str).map_err(|e| {
                    tracing::error!("Invalid email address provided: {}", email_str);
                    AppError::EmailAddressError(format!("Invalid email address: {}", e))
                })?;
                (email.to_string(), true)
            }
        };

        // Try to insert user
        let result = sqlx::query_as::<_, User>(
            "INSERT INTO users (wallet_address, email, email_verified)
            VALUES ($1, $2, $3)
            RETURNING id, wallet_address, username, display_name, email, email_verified, trust_rating, created_at, updated_at",
        )
        .bind(&wallet_address)
        .bind(&email)
        .bind(email_verified)
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
        Ok((user, token))
    }

    /// Create a new user (returns the created `User`).
    pub async fn create_user_with_details(
        &self,
        wallet_address: &str,
        username: Option<&str>,
        display_name: Option<&str>,
    ) -> Result<User, AppError> {
        let wallet_address = WalletAddress::new(wallet_address)?;

        // Validate username if provided
        let username = if let Some(uname) = username {
            Some(crate::models::Username::new(uname)?)
        } else {
            None
        };

        let trust_rating = 10.0;

        // Construct default email from wallet address
        let default_email = format!("{}@stackswars.com", wallet_address.as_str());
        // Validate the email format
        EmailAddress::from_str(&default_email).map_err(|e| {
            AppError::EmailAddressError(format!("Failed to create default email: {}", e))
        })?;

        // Try to insert user with optional fields
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (wallet_address, username, display_name, email, email_verified, trust_rating)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, wallet_address, username, display_name, email, email_verified, trust_rating, created_at, updated_at",
        )
        .bind(&wallet_address)
        .bind(username.as_ref())
        .bind(display_name)
        .bind(&default_email)
        .bind(false)
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
