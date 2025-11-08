//! # User HTTP Handlers
//!
//! HTTP request handlers for user-related operations.
//!
//! ## Endpoints Provided
//! - User creation (registration)
//! - User profile retrieval
//! - Username updates
//! - Display name updates
//!
//! ## Architecture
//! ```text
//! HTTP Request → Handler → UserRepository → PostgreSQL
//!                                ↓
//!                           Response
//! ```

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::AuthClaims, db::user::UserRepository, errors::AppError, models::user::UserV2,
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

// ============================================================================
// User Creation
// ============================================================================

/// Create a new user account
///
/// Registers a new user with their Stacks wallet address and generates
/// a JWT token for authentication.
///
/// ## Authentication
/// None required (public endpoint for registration)
///
/// ## Request
/// ```json
/// {
///   "walletAddress": "SP2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKNRV9EJ7"
/// }
/// ```
///
/// ## Response
/// ```json
/// {
///   "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
/// }
/// ```
///
/// ## Errors
/// - `400 Bad Request` - Invalid wallet address format
/// - `409 Conflict` - Wallet address already registered
/// - `500 Internal Server Error` - Database error
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<String>, (StatusCode, String)> {
    let repo = UserRepository::new(state.postgres.clone());

    // Create user and get JWT token
    let token = repo
        .create_user(payload.wallet_address.clone())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create user: {}", e);
            e.to_response()
        })?;

    tracing::info!(
        "User created successfully: Wallet: {}",
        payload.wallet_address
    );

    Ok(Json(token))
}

// ============================================================================
// User Retrieval
// ============================================================================

/// Get user profile by ID
///
/// Retrieves detailed user information including username, display name,
/// wallet address, and trust rating.
///
/// ## Authentication
/// None required (public endpoint)
///
/// ## Path Parameters
/// - `user_id` - UUID of the user
///
/// ## Response
/// ```json
/// {
///   "id": "550e8400-e29b-41d4-a716-446655440000",
///   "walletAddress": "SP2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKNRV9EJ7",
///   "username": "player123",
///   "displayName": "Pro Gamer",
///   "trustRating": 10.0,
///   "createdAt": "2024-01-01T00:00:00Z",
///   "updatedAt": "2024-01-15T12:30:00Z"
/// }
/// ```
///
/// ## Errors
/// - `404 Not Found` - User doesn't exist
/// - `500 Internal Server Error` - Database error
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserV2>, (StatusCode, String)> {
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

/// Update user's username
///
/// Changes the user's unique username. The username must not already be
/// taken by another user.
///
/// ## Authentication
/// Required (JWT token in Authorization header)
///
/// ## Request
/// ```json
/// {
///   "username": "new_username"
/// }
/// ```
///
/// ## Response
/// ```json
/// {
///   "username": "new_username"
/// }
/// ```
///
/// ## Errors
/// - `400 Bad Request` - Invalid username format
/// - `401 Unauthorized` - Invalid or missing JWT token
/// - `409 Conflict` - Username already taken
/// - `500 Internal Server Error` - Database error
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

    repo.update_username(user_id, &payload.username)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update username for {}: {}", user_id, e);
            e.to_response()
        })?;

    tracing::info!(
        "Username updated for user {} to '{}'",
        user_id,
        payload.username
    );
    Ok(Json(payload))
}

/// Update user's display name
///
/// Changes the user's display name shown in the UI. Unlike username,
/// display names don't need to be unique.
///
/// ## Authentication
/// Required (JWT token in Authorization header)
///
/// ## Request
/// ```json
/// {
///   "displayName": "Cool Player"
/// }
/// ```
///
/// ## Response
/// ```json
/// {
///   "displayName": "Cool Player"
/// }
/// ```
///
/// ## Errors
/// - `401 Unauthorized` - Invalid or missing JWT token
/// - `500 Internal Server Error` - Database error
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
