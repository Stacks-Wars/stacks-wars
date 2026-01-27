// Lobby management handlers: create/join/manage lobbies

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::handlers::stacks::has_joined;
use crate::models::WalletAddress;
use crate::{auth::AuthClaims, db::lobby::LobbyRepository, models::Lobby, state::AppState};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLobbyRequest {
    pub name: String,
    pub description: Option<String>,
    pub entry_amount: Option<f64>,
    pub current_amount: Option<f64>,
    pub token_symbol: Option<String>,
    pub token_contract_id: Option<String>,
    pub contract_address: Option<String>,
    pub is_private: Option<bool>,
    #[serde(default)]
    pub is_sponsored: bool,
    pub game_id: Uuid,
    pub game_path: String,
}

#[derive(Debug, Deserialize)]
pub struct LobbyQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

// ============================================================================
// Handlers
// ============================================================================

/// Create a new lobby. Authenticated endpoint that returns the full `Lobby`.
pub async fn create_lobby(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(payload): Json<CreateLobbyRequest>,
) -> Result<(StatusCode, Json<Lobby>), (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Invalid user ID in token");
        (StatusCode::UNAUTHORIZED, "Invalid token".to_string())
    })?;

    // Get user's wallet address from JWT claims
    let wallet_address = WalletAddress::try_from(claims.wallet.as_str()).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid wallet address in token".to_string(),
        )
    })?;

    // Confirm join if contract_address is provided
    if let Some(ref contract_addr) = payload.contract_address {
        let contract_wallet = WalletAddress::try_from(contract_addr.as_str()).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid contract address".to_string(),
            )
        })?;
        let has_joined = has_joined(&contract_wallet, &wallet_address, &state)
            .await
            .map_err(|e| e.to_response())?;
        if !has_joined {
            return Err((
                StatusCode::BAD_REQUEST,
                "Player has not joined the vault contract".to_string(),
            ));
        }
    }

    // For non-sponsored lobbies, default current_amount to entry_amount if not provided
    let current_amount = if !payload.is_sponsored {
        payload.current_amount.or(payload.entry_amount)
    } else {
        payload.current_amount
    };

    let repo = LobbyRepository::new(state.postgres.clone());

    let lobby = repo
        .create_lobby(
            &payload.name,
            payload.description.as_deref(),
            user_id,
            payload.game_id,
            &payload.game_path,
            payload.entry_amount,
            current_amount,
            payload.token_symbol.as_deref(),
            payload.token_contract_id.as_deref(),
            payload.contract_address.as_deref(),
            payload.is_private.unwrap_or(false),
            payload.is_sponsored,
            state.redis.clone(),
            state.clone(),
        )
        .await
        .map_err(|e| e.to_response())?;

    Ok((StatusCode::CREATED, Json(lobby)))
}

/// Get lobby details by UUID. Public endpoint returning `Lobby`.
pub async fn get_lobby(
    State(state): State<AppState>,
    Path(lobby_id): Path<Uuid>,
) -> Result<Json<Lobby>, (StatusCode, String)> {
    let repo = LobbyRepository::new(state.postgres);
    let lobby = repo
        .find_by_id(lobby_id)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(lobby))
}

/// Get lobby details by path. Public endpoint returning `Lobby`.
pub async fn get_lobby_by_path(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Json<Lobby>, (StatusCode, String)> {
    let repo = LobbyRepository::new(state.postgres);
    let lobby = repo
        .find_by_path(&path)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(lobby))
}
/// List lobbies for a game with optional pagination. Public endpoint.
pub async fn list_lobbies_by_game(
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
    Query(query): Query<LobbyQuery>,
) -> Result<Json<PaginatedResponse<Lobby>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(20).min(100) as usize;
    let offset = query.offset.unwrap_or(0).max(0) as usize;

    let repo = LobbyRepository::new(state.postgres);
    let (lobbies, total) = repo
        .find_by_game_id(game_id, offset, limit)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(PaginatedResponse {
        data: lobbies,
        total,
        limit: limit as i64,
        offset: offset as i64,
    }))
}

/// List lobbies created by the authenticated user. Requires JWT.
pub async fn list_my_lobbies(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Query(query): Query<LobbyQuery>,
) -> Result<Json<PaginatedResponse<Lobby>>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    let limit = query.limit.unwrap_or(20).min(100) as usize;
    let offset = query.offset.unwrap_or(0).max(0) as usize;

    let repo = LobbyRepository::new(state.postgres);
    let (lobbies, total) = repo
        .find_by_creator(user_id, offset, limit)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(PaginatedResponse {
        data: lobbies,
        total,
        limit: limit as i64,
        offset: offset as i64,
    }))
}

/// List all lobbies with pagination. Public endpoint.
pub async fn get_all_lobbies(
    State(state): State<AppState>,
    Query(query): Query<LobbyQuery>,
) -> Result<Json<PaginatedResponse<Lobby>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0).max(0);

    let repo = LobbyRepository::new(state.postgres);
    let (lobbies, total) = repo
        .get_all_lobbies(limit, offset)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(PaginatedResponse {
        data: lobbies,
        total,
        limit,
        offset,
    }))
}
