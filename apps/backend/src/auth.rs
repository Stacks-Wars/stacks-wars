use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use axum_extra::TypedHeader;
use chrono::{Duration, Utc};
use headers::{Authorization, authorization::Bearer};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};

use crate::{
    errors::AppError,
    models::{User, user::Claims},
};
pub struct AuthClaims(pub Claims);

impl<S> FromRequestParts<S> for AuthClaims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state)
                .await
                .map_err(|_| {
                    (
                        StatusCode::UNAUTHORIZED,
                        "Missing or invalid Authorization header".into(),
                    )
                })?;

        AuthClaims::from_token(bearer.token())
    }
}

impl AuthClaims {
    pub fn from_token(token: &str) -> Result<Self, (StatusCode, String)> {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token".into()))?;

        Ok(Self(token_data.claims))
    }
}

pub fn generate_jwt(user: &User) -> Result<String, AppError> {
    let expiration = (Utc::now() + Duration::days(7)).timestamp() as usize;
    let claims = Claims {
        sub: user.id.to_string(),
        wallet: user.wallet_address.clone(),
        exp: expiration,
    };

    let secret = std::env::var("JWT_SECRET").map_err(|e| AppError::EnvError(e.to_string()))?;
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(AppError::JwtError)
}
