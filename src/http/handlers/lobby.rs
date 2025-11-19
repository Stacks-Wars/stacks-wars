// Lobby management handlers: create/join/manage lobbies

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::AuthClaims, db::lobby::LobbyRepository, models::db::Lobby, state::AppState};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLobbyRequest {
    pub name: String,
    pub description: Option<String>,
    pub entry_amount: Option<f64>,
    pub token_symbol: Option<String>,
    pub token_contract_id: Option<String>,
    pub contract_address: Option<String>,
    pub is_private: Option<bool>,
    pub game_id: Uuid,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLobbyResponse {
    pub lobby_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct LobbyQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Create a new lobby. Authenticated endpoint that returns the new `lobby_id`.
pub async fn create_lobby(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(payload): Json<CreateLobbyRequest>,
) -> Result<(StatusCode, Json<CreateLobbyResponse>), (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Invalid user ID in token");
        (StatusCode::UNAUTHORIZED, "Invalid token".to_string())
    })?;

    let repo = LobbyRepository::new(state.postgres.clone());
    let is_sponsored = payload.entry_amount.map_or(false, |amount| amount == 0.0);

    let lobby = repo
        .create_lobby(
            payload.name,
            payload.description,
            user_id,
            payload.game_id,
            payload.entry_amount,
            payload.token_symbol,
            payload.token_contract_id,
            payload.contract_address,
            payload.is_private.unwrap_or(false),
            is_sponsored,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create lobby: {}", e);
            e.to_response()
        })?;

    Ok((
        StatusCode::CREATED,
        Json(CreateLobbyResponse { lobby_id: lobby.id }),
    ))
}

/// Get lobby details by UUID. Public endpoint returning `Lobby`.
pub async fn get_lobby(
    State(state): State<AppState>,
    Path(lobby_id): Path<Uuid>,
) -> Result<Json<Lobby>, (StatusCode, String)> {
    let repo = LobbyRepository::new(state.postgres);
    let lobby = repo.get_by_id(lobby_id).await.map_err(|e| {
        tracing::error!("Failed to get lobby {}: {}", lobby_id, e);
        e.to_response()
    })?;

    Ok(Json(lobby))
}

/// List lobbies for a game with optional pagination. Public endpoint.
pub async fn list_lobbies_by_game(
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
    Query(query): Query<LobbyQuery>,
) -> Result<Json<Vec<Lobby>>, (StatusCode, String)> {
    let repo = LobbyRepository::new(state.postgres);
    let lobbies = repo.find_by_game_id(game_id).await.map_err(|e| {
        tracing::error!("Failed to list lobbies for game {}: {}", game_id, e);
        e.to_response()
    })?;

    // Apply pagination
    let limit = query.limit.unwrap_or(20).min(100) as usize;
    let offset = query.offset.unwrap_or(0).max(0) as usize;
    let paginated: Vec<Lobby> = lobbies.into_iter().skip(offset).take(limit).collect();

    Ok(Json(paginated))
}

/// List lobbies created by the authenticated user. Requires JWT.
pub async fn list_my_lobbies(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Query(query): Query<LobbyQuery>,
) -> Result<Json<Vec<Lobby>>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    let repo = LobbyRepository::new(state.postgres);
    let lobbies = repo.find_by_creator(user_id).await.map_err(|e| {
        tracing::error!("Failed to list lobbies for user {}: {}", user_id, e);
        e.to_response()
    })?;

    // Apply pagination
    let limit = query.limit.unwrap_or(20).min(100) as usize;
    let offset = query.offset.unwrap_or(0).max(0) as usize;
    let paginated: Vec<Lobby> = lobbies.into_iter().skip(offset).take(limit).collect();

    Ok(Json(paginated))
}

/// Delete a lobby. Only the lobby creator may delete it. Returns `204`.
pub async fn delete_lobby(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(lobby_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    let repo = LobbyRepository::new(state.postgres);

    // Verify user is the creator
    let lobby = repo.get_by_id(lobby_id).await.map_err(|e| {
        tracing::error!("Lobby not found: {}", e);
        e.to_response()
    })?;

    if lobby.creator_id != user_id {
        tracing::warn!(
            "User {} attempted to delete lobby {} owned by {}",
            user_id,
            lobby_id,
            lobby.creator_id
        );
        return Err((
            StatusCode::FORBIDDEN,
            "Only the creator can delete this lobby".to_string(),
        ));
    }

    repo.delete_lobby(lobby_id).await.map_err(|e| {
        tracing::error!("Failed to delete lobby: {}", e);
        e.to_response()
    })?;

    Ok(StatusCode::NO_CONTENT)
}
