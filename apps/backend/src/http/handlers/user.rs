// User HTTP handlers: registration, profile, and updates

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::AuthClaims, db::user::UserRepository, errors::AppError, models::User, state::AppState,
};

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request body for creating a new user
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    /// User's Stacks wallet address (principal)
    pub wallet_address: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserResponse {
    pub user: User,
    pub token: String,
}

/// Request body for updating username
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUsernameRequest {
    /// New username (must be unique)
    pub username: String,
}

/// Request body for updating display name
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDisplayNameRequest {
    /// New display name (for UI)
    pub display_name: String,
}

/// Request body for updating user profile (partial update)
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    /// Optional new username
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Optional new display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Optional new trust rating (admin only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_rating: Option<f64>,
}

// ============================================================================
// User Creation
// ============================================================================

/// Register a new user and return user details with JWT token.
///
/// Public endpoint. Returns a JSON object containing `user` and `token` on success.
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, (StatusCode, String)> {
    let repo = UserRepository::new(state.postgres.clone());

    let (user, token) = repo
        .create_user(&payload.wallet_address, &state.config.jwt_secret)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(CreateUserResponse { user, token }))
}

// ============================================================================
// User Retrieval
// ============================================================================

/// Get a user's public profile by UUID.
///
/// Public endpoint returning `User` or `404` if not found.
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<User>, (StatusCode, String)> {
    let repo = UserRepository::new(state.postgres.clone());

    let user = repo
        .find_by_id(user_id)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(user))
}

// ============================================================================
// User Updates
// ============================================================================

/// Update the authenticated user's username.
///
/// Requires a valid JWT. Returns the updated username on success.
pub async fn update_username(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(payload): Json<UpdateUsernameRequest>,
) -> Result<Json<UpdateUsernameRequest>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Invalid user ID in JWT token");
        AppError::Unauthorized("Invalid token".into()).to_response()
    })?;

    let repo = UserRepository::new(state.postgres.clone());

    repo.update_username(user_id, &payload.username, state.redis.clone())
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(payload))
}

/// Update the authenticated user's display name.
///
/// Requires a valid JWT. Display names are not required to be unique.
pub async fn update_display_name(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(payload): Json<UpdateDisplayNameRequest>,
) -> Result<Json<UpdateDisplayNameRequest>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Invalid user ID in JWT token");
        AppError::Unauthorized("Invalid token".into()).to_response()
    })?;

    let repo = UserRepository::new(state.postgres.clone());

    repo.update_display_name(user_id, &payload.display_name, state.redis.clone())
        .await
        .map_err(|e| {
            tracing::error!("Failed to update display name for {}: {}", user_id, e);
            e.to_response()
        })?;

    Ok(Json(payload))
}

// ============================================================================
// User Profile Update (Partial)
// ============================================================================

/// Partially update the authenticated user's profile fields.
///
/// Accepts optional `username` and `displayName` fields and returns the
/// updated `User` on success. Requires a valid JWT.
pub async fn update_profile(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<User>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Invalid user ID in JWT token");
        AppError::Unauthorized("Invalid token".into()).to_response()
    })?;

    let repo = UserRepository::new(state.postgres.clone());

    let user = repo
        .update_profile(
            user_id,
            payload.username.as_deref(),
            payload.display_name.as_deref(),
            state.redis.clone(),
        )
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(user))
}
