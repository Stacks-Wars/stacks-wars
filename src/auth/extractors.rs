use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use axum_extra::TypedHeader;
use headers::{Authorization, authorization::Bearer};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

use super::jwt::Claims;

/// Axum extractor for authenticated requests
///
/// Extracts and validates JWT token from Authorization header.
///
/// # Usage
/// ```rust
/// async fn protected_handler(
///     AuthClaims(claims): AuthClaims,
/// ) -> Result<Json<Response>, AppError> {
///     let user_id = claims.user_id()?;
///     // ... handler logic
/// }
/// ```
///
/// # Authentication Flow
/// 1. Extract Bearer token from Authorization header
/// 2. Decode and validate JWT
/// 3. Return claims if valid, error if not
pub struct AuthClaims(pub Claims);

impl<S> FromRequestParts<S> for AuthClaims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state)
                .await
                .map_err(|_| {
                    (
                        StatusCode::UNAUTHORIZED,
                        "Missing or invalid Authorization header".into(),
                    )
                })?;

        // Validate token
        AuthClaims::from_token(bearer.token())
    }
}

impl AuthClaims {
    /// Create AuthClaims from a JWT token string
    ///
    /// # Arguments
    /// * `token` - JWT token string
    ///
    /// # Returns
    /// Validated claims if token is valid
    ///
    /// # Errors
    /// Returns (StatusCode, String) tuple on validation failure
    pub fn from_token(token: &str) -> Result<Self, (StatusCode, String)> {
        let secret = std::env::var("JWT_SECRET").map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "JWT_SECRET not configured".into(),
            )
        })?;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|e| {
            tracing::warn!("JWT validation failed: {}", e);
            (StatusCode::UNAUTHORIZED, "Invalid or expired token".into())
        })?;

        Ok(Self(token_data.claims))
    }

    /// Get the user ID from claims
    pub fn user_id(&self) -> Result<uuid::Uuid, (StatusCode, String)> {
        self.0
            .user_id()
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID in token".into()))
    }

    /// Get the wallet address from claims
    pub fn wallet_address(&self) -> &str {
        &self.0.wallet
    }
}
