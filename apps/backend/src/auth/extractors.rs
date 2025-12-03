use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use axum_extra::TypedHeader;
use headers::{Authorization, authorization::Bearer};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

use super::jwt::Claims;

/// WebSocket auth extractor: optional. If the `Authorization: Bearer <token>` header
/// is present and valid, returns `WsAuth(Some(AuthClaims))`. If the header is absent
/// returns `WsAuth(None)`. If the header is present but invalid the extractor
/// rejects with `UNAUTHORIZED`.
pub struct WsAuth(pub Option<AuthClaims>);

impl FromRequestParts<crate::state::AppState> for WsAuth {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::state::AppState,
    ) -> Result<Self, Self::Rejection> {
        // Look for Authorization header manually so we can distinguish missing vs present-but-invalid
        if let Some(hv) = parts.headers.get("authorization") {
            let hv_str = hv.to_str().map_err(|_| {
                (StatusCode::BAD_REQUEST, "Invalid Authorization header encoding".to_string())
            })?;

            // Expect format: "Bearer <token>"
            let parts: Vec<&str> = hv_str.splitn(2, ' ').collect();
            if parts.len() != 2 || !parts[0].eq_ignore_ascii_case("bearer") {
                return Err((StatusCode::BAD_REQUEST, "Invalid Authorization header".to_string()));
            }

            let token = parts[1];
            let secret = state.config.jwt_secret.clone();
            let claims = AuthClaims::from_token_with_secret(token, &secret)?;
            return Ok(WsAuth(Some(claims)));
        }

        // No header -> anonymous websocket connection
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
        // Extract Authorization header
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| {
                    (
                        StatusCode::UNAUTHORIZED,
                        "Missing or invalid Authorization header".into(),
                    )
                })?;

        // Validate token using secret from AppState config
        let secret = state.config.jwt_secret.clone();
        AuthClaims::from_token_with_secret(bearer.token(), &secret)
    }
}

impl AuthClaims {
    /// Create AuthClaims from a JWT token string
    pub fn from_token(token: &str) -> Result<Self, (StatusCode, String)> {
        let secret = std::env::var("JWT_SECRET").map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "JWT_SECRET not configured".into(),
            )
        })?;

        AuthClaims::from_token_with_secret(token, &secret)
    }

    pub fn from_token_with_secret(token: &str, secret: &str) -> Result<Self, (StatusCode, String)> {
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
