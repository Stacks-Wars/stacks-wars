use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::AuthClaims,
    db::user::{
        get::get_user_by_id,
        patch::{update_display_name, update_username},
        post::create_user,
    },
    errors::AppError,
    models::User,
    state::AppState,
};

#[derive(Deserialize)]
pub struct CreateUserPayload {
    pub wallet_address: String,
}

pub async fn create_user_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<Json<String>, (StatusCode, String)> {
    match create_user(payload.wallet_address.clone(), state.redis.clone()).await {
        Ok(token) => {
            tracing::info!(
                "User created with wallet address: {}",
                payload.wallet_address
            );
            Ok(Json(token))
        }
        Err(err) => {
            tracing::error!("Error creating user: {}", err);
            Err(err.to_response())
        }
    }
}

pub async fn get_user_handler(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<User>, (StatusCode, String)> {
    let user = get_user_by_id(user_id, state.redis.clone())
        .await
        .map_err(|e| {
            tracing::error!("Error retrieving user: {}", e);
            e.to_response()
        })?;

    Ok(Json(user))
}

#[derive(Deserialize)]
pub struct UsernamePayload {
    pub username: String,
}
pub async fn update_username_handler(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(payload): Json<UsernamePayload>,
) -> Result<Json<String>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Unauthorized access attempt");
        AppError::Unauthorized("Invalid user ID in token".into()).to_response()
    })?;

    let username = payload.username;

    update_username(user_id, username.clone(), state.redis)
        .await
        .map_err(|e| {
            tracing::error!("Error updating username: {}", e);
            e.to_response()
        })?;

    tracing::info!("Username updated for user ID: {}", user_id);
    Ok(Json(username))
}

#[derive(Deserialize)]
pub struct DisplayNamePayload {
    pub display_name: String,
}
pub async fn update_display_name_handler(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(payload): Json<DisplayNamePayload>,
) -> Result<Json<String>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Unauthorized access attempt");
        AppError::Unauthorized("Invalid user ID in token".into()).to_response()
    })?;

    let display_name = payload.display_name;

    update_display_name(user_id, display_name.clone(), state.redis)
        .await
        .map_err(|e| {
            tracing::error!("Error updating display name: {}", e);
            e.to_response()
        })?;

    tracing::info!("Display name updated for user ID: {}", user_id);
    Ok(Json(display_name))
}
