// User HTTP handlers: registration, profile, and updates

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::AuthClaims,
    db::user::UserRepository,
    errors::AppError,
    models::db::{User, Username, WalletAddress},
    state::AppState,
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

/// Register a new user and return a JWT token.
///
/// Public endpoint. Returns a JSON object containing `token` on success.
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<String>, (StatusCode, String)> {
    let wallet_address = WalletAddress::new(&payload.wallet_address).map_err(|e| {
        tracing::warn!("Invalid wallet address: {}", e);
        AppError::from(e).to_response()
    })?;

    let repo = UserRepository::new(state.postgres.clone());

    // Create user and get JWT token (pass by reference, no clone)
    let token = repo
        .create_user(&wallet_address, &state.config.jwt_secret)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create user: {}", e);
            e.to_response()
        })?;

    tracing::info!("User created successfully: Wallet: {}", wallet_address);
    Ok(Json(token))
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

    let user = repo.find_by_id(user_id).await.map_err(|e| {
        tracing::error!("Failed to fetch user {}: {}", user_id, e);
        e.to_response()
    })?;

    tracing::debug!("Retrieved user profile for {}", user_id);
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

    let username = Username::new(&payload.username).map_err(|e| {
        tracing::warn!("Invalid username: {}", e);
        AppError::from(e).to_response()
    })?;

    let repo = UserRepository::new(state.postgres.clone());

    repo.update_username(user_id, &username)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update username for {}: {}", user_id, e);
            e.to_response()
        })?;

    tracing::info!("Username updated for user {} to '{}'", user_id, username);
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

    repo.update_display_name(user_id, &payload.display_name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update display name for {}: {}", user_id, e);
            e.to_response()
        })?;

    tracing::info!(
        "Display name updated for user {} to '{}'",
        user_id,
        payload.display_name
    );
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

    let username = if let Some(ref uname) = payload.username {
        Some(Username::new(uname).map_err(|e| {
            tracing::warn!("Invalid username: {}", e);
            AppError::from(e).to_response()
        })?)
    } else {
        None
    };

    let repo = UserRepository::new(state.postgres.clone());

    let user = repo
        .update_profile(user_id, username.as_ref(), payload.display_name.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to update profile for {}: {}", user_id, e);
            e.to_response()
        })?;

    tracing::info!(
        "Profile updated for user {}: username={:?}, display_name={:?}",
        user_id,
        username,
        payload.display_name
    );

    Ok(Json(user))
}
