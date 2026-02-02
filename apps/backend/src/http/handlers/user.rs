// User HTTP handlers: registration, profile, and updates

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use time::Duration;
use uuid::Uuid;

use crate::{
    auth::AuthClaims,
    db::user::UserRepository,
    errors::AppError,
    models::{User, LobbyInfo, LobbyStatus, keys::RedisKey},
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
    pub email_address: Option<String>,
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
}

/// Response for unclaimed rewards
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnclaimedReward {
    pub lobby_info: LobbyInfo,
    pub prize: f64,
}

/// Query parameters for player lobbies
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerLobbiesQuery {
    /// Optional comma-separated lobby statuses to filter by
    pub status: Option<String>,
    /// Maximum number of lobbies to return (default: 3)
    pub limit: Option<i64>,
    /// Number of lobbies to skip (default: 0)
    pub offset: Option<i64>,
}

// ============================================================================
// User Creation
// ============================================================================

/// Register a new user and return user details with JWT token.
///
/// Public endpoint. Returns the created User and sets an httpOnly cookie with the auth token.
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Response, (StatusCode, String)> {
    let repo = UserRepository::new(state.postgres.clone());

    let (user, token) = repo
        .create_user(
            &payload.wallet_address,
            payload.email_address.as_deref(),
            &state.config.jwt_secret,
        )
        .await
        .map_err(|e| e.to_response())?;

    // Create httpOnly cookie for the token
    let cookie = Cookie::build(("auth_token", token.clone()))
        .path("/")
        .max_age(Duration::days(14)) // 14 days in seconds
        .same_site(SameSite::Lax) // More permissive than Strict for cross-domain compatibility
        .http_only(true)
        .secure(state.config.is_production())
        .build();

    let mut response = Json(user).into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok(response)
}

// ============================================================================
// User Retrieval
// ============================================================================

/// Get the authenticated user's profile.
///
/// Requires a valid JWT. Returns the authenticated `User` or `401` if not authenticated.
pub async fn get_me(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<User>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token".into()).to_response())?;

    let repo = UserRepository::new(state.postgres.clone());

    let user = repo
        .find_by_id(user_id)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(user))
}

/// Get a user's public profile by UUID, wallet address, or username.
///
/// Public endpoint returning `User` or `404` if not found.
/// Accepts any of:
/// - UUID (e.g., "550e8400-e29b-41d4-a716-446655440000")
/// - Wallet address (e.g., "SP2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKNRV9EJ7")
/// - Username (case-insensitive)
pub async fn get_user(
    State(state): State<AppState>,
    Path(identifier): Path<String>,
) -> Result<Json<User>, (StatusCode, String)> {
    let repo = UserRepository::new(state.postgres.clone());

    let user = repo
        .find_user(&identifier)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(user))
}

/// Get all lobbies a player is part of, filtered by status.
///
/// Public endpoint returning `(Vec<LobbyInfo>, total_count)` or `404` if user not found.
/// Defaults to active lobbies (waiting, starting, in_progress) if no status filter provided.
/// Supports pagination with limit (default: 3) and offset (default: 0).
pub async fn get_player_lobbies(
    State(state): State<AppState>,
    Path(user_id_str): Path<String>,
    Query(params): Query<PlayerLobbiesQuery>,
) -> Result<Json<(Vec<LobbyInfo>, i64)>, (StatusCode, String)> {
    // Parse user ID
    let user_id = Uuid::parse_str(&user_id_str).map_err(|_| {
        AppError::BadRequest("Invalid user ID format".into()).to_response()
    })?;

    // Parse status filter
    let status_filter = params.status.as_ref().map(|s| {
        s.split(',')
            .filter_map(|status_str| status_str.trim().parse::<LobbyStatus>().ok())
            .collect::<Vec<_>>()
    }).unwrap_or_else(|| vec![LobbyStatus::Waiting, LobbyStatus::Starting, LobbyStatus::InProgress]);

    // Get pagination params
    let limit = params.limit.unwrap_or(3);
    let offset = params.offset.unwrap_or(0);

    let repo = UserRepository::new(state.postgres.clone());

    let (lobbies, total) = repo
        .get_player_lobbies(user_id, &state.redis, &status_filter, limit, offset)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json((lobbies, total)))
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

// ============================================================================
// Unclaimed Rewards
// ============================================================================

/// Get the authenticated user's unclaimed rewards.
///
/// Requires a valid JWT. Returns a list of unclaimed rewards with lobby info and prize amounts.
pub async fn get_unclaimed_rewards(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Vec<UnclaimedReward>>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Invalid user ID in JWT token");
        AppError::Unauthorized("Invalid token".into()).to_response()
    })?;

    let repo = UserRepository::new(state.postgres.clone());

    let rewards = repo
        .get_unclaimed_rewards(user_id, &state.redis)
        .await
        .map_err(|e| e.to_response())?;

    let response = rewards
        .into_iter()
        .map(|(lobby_info, prize)| UnclaimedReward { lobby_info, prize })
        .collect();

    Ok(Json(response))
}

// ============================================================================
// User Logout
// ============================================================================

/// Logout the authenticated user by revoking their JWT token.
///
/// Revokes the token by storing its JTI in Redis with the remaining TTL.
/// Also clears the auth_token cookie.
pub async fn logout(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Response, (StatusCode, String)> {
    // Get the JTI and remaining TTL from the token
    let jti = claims.jti();
    let ttl = claims.remaining_ttl();

    // Store revoked token in Redis with remaining TTL
    let key = RedisKey::revoked_token(jti);

    let mut conn = state.redis.get().await.map_err(|e| {
        tracing::error!("Failed to get Redis connection for logout: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Logout failed".to_string(),
        )
    })?;

    // Set the revoked token with expiration matching the token's remaining lifetime
    let _: () = conn.set_ex(&key, true, ttl as u64).await.map_err(|e| {
        tracing::error!("Failed to revoke token in Redis: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Logout failed".to_string(),
        )
    })?;

    // Create cookie with max-age=0 to clear it
    let cookie = Cookie::build(("auth_token", ""))
        .path("/")
        .max_age(Duration::seconds(0))
        .same_site(SameSite::Lax)
        .http_only(true)
        .secure(state.config.is_production())
        .build();

    let mut response = StatusCode::NO_CONTENT.into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());

    tracing::info!(
        "User {} logged out successfully",
        claims.user_id().unwrap_or_default()
    );

    Ok(response)
}
