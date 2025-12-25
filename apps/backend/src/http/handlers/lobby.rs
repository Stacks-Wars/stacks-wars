// Lobby management handlers: create/join/manage lobbies

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::AuthClaims,
    db::{lobby::LobbyRepository, lobby_state::LobbyStateRepository},
    models::{Lobby, LobbyExtended},
    state::AppState,
    ws::lobby::LobbyServerMessage,
};

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
        )
        .await
        .map_err(|e| e.to_response())?;

    // Broadcast lobby creation to lobby list subscribers
    tokio::spawn({
        let state_clone = state.clone();
        let lobby_id = lobby.id();
        async move {
            use crate::ws::broadcast;

            // Fetch lobby with joins to construct LobbyExtended for broadcast
            let lobby_repo = LobbyRepository::new(state_clone.postgres.clone());
            let lobby_state_repo = LobbyStateRepository::new(state_clone.redis.clone());

            if let Ok(lobby_with_joins) = lobby_repo.find_by_id_with_joins(lobby_id).await {
                if let Ok(lobby_state) = lobby_state_repo.get_state(lobby_id).await {
                    let (creator_wallet, creator_username, creator_display_name) =
                        lobby_with_joins.creator_info();
                    let (game_image_url, game_min_players, game_max_players) =
                        lobby_with_joins.game_info();

                    let lobby_extended = LobbyExtended::from_parts(
                        lobby_with_joins.to_lobby(),
                        lobby_state,
                        creator_wallet,
                        creator_username,
                        creator_display_name,
                        game_image_url,
                        game_min_players,
                        game_max_players,
                    );

                    let _ = broadcast::broadcast_lobby_list(
                        &state_clone,
                        &LobbyServerMessage::LobbyCreated {
                            lobby: lobby_extended,
                        },
                    )
                    .await;
                }
            }
        }
    });

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
) -> Result<Json<Vec<Lobby>>, (StatusCode, String)> {
    let repo = LobbyRepository::new(state.postgres);
    let lobbies = repo
        .find_by_game_id(game_id)
        .await
        .map_err(|e| e.to_response())?;

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
    let lobbies = repo
        .find_by_creator(user_id)
        .await
        .map_err(|e| e.to_response())?;

    // Apply pagination
    let limit = query.limit.unwrap_or(20).min(100) as usize;
    let offset = query.offset.unwrap_or(0).max(0) as usize;
    let paginated: Vec<Lobby> = lobbies.into_iter().skip(offset).take(limit).collect();

    Ok(Json(paginated))
}

/// List all lobbies with pagination. Public endpoint.
pub async fn get_all_lobbies(
    State(state): State<AppState>,
    Query(query): Query<LobbyQuery>,
) -> Result<Json<Vec<Lobby>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0).max(0);

    let repo = LobbyRepository::new(state.postgres);
    let lobbies = repo
        .get_all_lobbies(limit, offset)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(lobbies))
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
    let lobby = repo
        .find_by_id(lobby_id)
        .await
        .map_err(|e| e.to_response())?;

    if lobby.creator_id != user_id {
        return Err((
            StatusCode::FORBIDDEN,
            "Only the creator can delete this lobby".to_string(),
        ));
    }

    repo.delete_lobby(lobby_id)
        .await
        .map_err(|e| e.to_response())?;

    Ok(StatusCode::NO_CONTENT)
}
