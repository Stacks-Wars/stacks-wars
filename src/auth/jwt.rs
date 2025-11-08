//! JWT Token Generation and Validation
//!
//! Provides secure JWT-based authentication for the Stacks Wars API.
//!
//! ## Security Features
//!
//! - **Algorithm**: HS256 (HMAC-SHA256)
//! - **Secret Key**: Minimum 32 characters (256 bits)
//! - **Token Expiration**: 7 days (configurable via TOKEN_EXPIRY_DAYS)
//! - **Standard Claims**: sub, iat, exp, jti
//! - **Token Tracking**: JWT ID (jti) for revocation support
//!
//! ## Environment Variables
//!
//! - `JWT_SECRET` (required): Secret key for signing tokens, minimum 32 characters
//! - `TOKEN_EXPIRY_DAYS` (optional): Token validity period in days, default 7
//!
//! ## Token Claims
//!
//! | Claim | Description | Type |
//! |-------|-------------|------|
//! | `sub` | User ID (UUID) | String |
//! | `wallet` | Stacks wallet address | String |
//! | `iat` | Issued at timestamp | i64 |
//! | `exp` | Expiration timestamp | i64 |
//! | `jti` | JWT ID for tracking | String (optional) |
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use crate::auth::{generate_jwt, validate_jwt_secret};
//!
//! // Validate configuration at startup
//! validate_jwt_secret()?;
//!
//! // Generate token for authenticated user
//! let user = user_repo.find_by_wallet(&wallet).await?;
//! let token = generate_jwt(&user)?;
//!
//! // Token is sent to client and included in Authorization header:
//! // Authorization: Bearer <token>
//! ```
//!
//! ## Security Best Practices
//!
//! 1. **Secret Management**: Store JWT_SECRET securely (use secrets manager in production)
//! 2. **HTTPS Only**: Always transmit tokens over HTTPS
//! 3. **Token Storage**: Store tokens securely on client (httpOnly cookies or secure storage)
//! 4. **Token Rotation**: Consider implementing refresh tokens for long-lived sessions
//! 5. **Revocation**: Use `jti` claim to implement token revocation if needed
//! 6. **Validation**: Always validate tokens on protected endpoints using `AuthClaims` extractor

use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use uuid::Uuid;

use crate::{errors::AppError, models::user::UserV2};

/// JWT Claims structure
///
/// Contains user identification, timestamps, and expiration data.
/// Follows JWT standard claims (RFC 7519).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Wallet address (custom claim)
    pub wallet: String,
    /// Issued at timestamp (seconds since Unix epoch)
    pub iat: i64,
    /// Expiration timestamp (seconds since Unix epoch)
    pub exp: i64,
    /// JWT ID for token tracking/revocation (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
}

impl Claims {
    /// Get user ID as UUID
    pub fn user_id(&self) -> Result<Uuid, AppError> {
        self.sub
            .parse()
            .map_err(|_| AppError::BadRequest("Invalid user ID in token".to_string()))
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Get token age in seconds
    pub fn age_seconds(&self) -> i64 {
        Utc::now().timestamp() - self.iat
    }
}

/// Generate JWT token for user authentication
///
/// Creates a signed JWT token with user identification and expiration.
/// Tokens are valid for 7 days and include issued-at timestamp for tracking.
///
/// # Security Features
/// - HS256 (HMAC-SHA256) signing algorithm
/// - 7-day expiration (configurable via TOKEN_EXPIRY_DAYS env var)
/// - Issued-at timestamp for token age validation
/// - Optional JWT ID for token revocation support
///
/// # Arguments
/// * `user` - User to generate token for
///
/// # Returns
/// JWT token string
///
/// # Errors
/// - `AppError::EnvError` if JWT_SECRET not set or invalid
/// - `AppError::JwtError` if token generation fails
///
/// # Examples
/// ```rust,ignore
/// let user = repo.find_by_wallet(&wallet).await?;
/// let token = generate_jwt(&user)?;
/// // Send token to client for authentication
/// ```
pub fn generate_jwt(user: &UserV2) -> Result<String, AppError> {
    // Validate secret exists and meets minimum requirements
    let secret = validate_jwt_secret_internal()?;

    let now = Utc::now();
    let expiry_days = std::env::var("TOKEN_EXPIRY_DAYS")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(7);

    let claims = Claims {
        sub: user.id.to_string(),
        wallet: user.wallet_address.clone(),
        iat: now.timestamp(),
        exp: (now + Duration::days(expiry_days)).timestamp(),
        jti: Some(Uuid::new_v4().to_string()), // For potential token revocation
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(AppError::JwtError)
}

/// Validate JWT_SECRET meets security requirements
///
/// Internal validation that checks:
/// - Secret is set in environment
/// - Secret is at least 32 characters (256 bits for HS256)
///
/// Returns the secret if valid.
fn validate_jwt_secret_internal() -> Result<String, AppError> {
    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::EnvError("JWT_SECRET must be set".to_string()))?;

    if secret.len() < 32 {
        return Err(AppError::EnvError(
            "JWT_SECRET must be at least 32 characters for security".to_string(),
        ));
    }

    Ok(secret)
}

/// Validate JWT_SECRET is set and meets security requirements at startup
///
/// Should be called during application initialization to fail fast
/// if JWT configuration is invalid.
///
/// # Security Requirements
/// - JWT_SECRET must be set in environment
/// - Must be at least 32 characters (256 bits for HS256 security)
///
/// # Errors
/// Returns AppError::EnvError if validation fails
///
/// # Examples
/// ```rust,ignore
/// // In main() or server startup
/// validate_jwt_secret()?;
/// ```
pub fn validate_jwt_secret() -> Result<(), AppError> {
    validate_jwt_secret_internal().map(|_| ())
}
