use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use axum_extra::extract::cookie::CookieJar;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use redis::AsyncCommands;

use super::jwt::Claims;
use crate::{models::keys::RedisKey, state::RedisClient};

/// WebSocket auth extractor: optional.
pub struct WsAuth(pub Option<AuthClaims>);

impl FromRequestParts<crate::state::AppState> for WsAuth {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::state::AppState,
    ) -> Result<Self, Self::Rejection> {
        // Try to get token from cookie first
        let jar = match CookieJar::from_request_parts(parts, state).await {
            Ok(jar) => jar,
            Err(_) => return Ok(WsAuth(None)),
        };
        if let Some(cookie) = jar.get("auth_token") {
            let token = cookie.value();
            let secret = state.config.jwt_secret.clone();
            if let Ok(claims) =
                AuthClaims::from_token_with_secret(token, &secret, &state.redis).await
            {
                return Ok(WsAuth(Some(claims)));
            }
        }

        // No auth or invalid -> anonymous websocket connection
        Ok(WsAuth(None))
    }
}

/// Axum extractor for authenticated requests
pub struct AuthClaims(pub Claims);

impl FromRequestParts<crate::state::AppState> for AuthClaims {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::state::AppState,
    ) -> Result<Self, Self::Rejection> {
        // Get token from cookie
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Authentication required".to_string(),
                )
            })?;

        let cookie = jar.get("auth_token").ok_or((
            StatusCode::UNAUTHORIZED,
            "Missing authentication cookie".to_string(),
        ))?;

        let token = cookie.value();
        let secret = state.config.jwt_secret.clone();
        AuthClaims::from_token_with_secret(token, &secret, &state.redis).await
    }
}

impl AuthClaims {
    /// Create AuthClaims from a JWT token string
    pub async fn from_token(
        token: &str,
        redis: &RedisClient,
    ) -> Result<Self, (StatusCode, String)> {
        let secret = std::env::var("JWT_SECRET").map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "JWT_SECRET not configured".into(),
            )
        })?;

        AuthClaims::from_token_with_secret(token, &secret, redis).await
    }

    pub async fn from_token_with_secret(
        token: &str,
        secret: &str,
        redis: &RedisClient,
    ) -> Result<Self, (StatusCode, String)> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|e| {
            tracing::warn!("JWT validation failed: {}", e);
            (StatusCode::UNAUTHORIZED, "Invalid or expired token".into())
        })?;

        let claims = token_data.claims;

        // Check if token is revoked
        let jti = claims.jti();
        let key = RedisKey::revoked_token(jti);

        let mut conn = redis.get().await.map_err(|e| {
            tracing::error!("Failed to get Redis connection for token check: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Authentication check failed".to_string(),
            )
        })?;

        let is_revoked: Option<bool> = conn.get(&key).await.map_err(|e| {
            tracing::error!("Failed to check token revocation status: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Authentication check failed".to_string(),
            )
        })?;

        if is_revoked.is_some() {
            tracing::warn!("Attempted use of revoked token: {}", jti);
            return Err((StatusCode::UNAUTHORIZED, "Token has been revoked".into()));
        }

        Ok(Self(claims))
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
