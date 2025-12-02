use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::AuthClaims,
    db::lobby::{
        get::{
            get_all_lobbies_extended, get_all_lobbies_info, get_lobbies_by_game_id,
            get_lobby_extended, get_lobby_info, get_lobby_players, get_player_lobbies,
        },
        patch::{
            join_lobby, leave_lobby, update_claim_state, update_lobby_state, update_player_state,
        },
        post::create_lobby,
    },
    errors::AppError,
    models::game::{
        ClaimState, LobbyExtended, LobbyInfo, LobbyPoolInput, LobbyQuery, LobbyState, Player,
        PlayerLobbyInfo, PlayerQuery, PlayerState, parse_lobby_states, parse_player_state,
    },
    state::AppState,
};

#[derive(Deserialize)]
pub struct CreateLobbyPayload {
    pub name: String,
    pub description: Option<String>,
    pub entry_amount: Option<f64>,
    pub current_amount: Option<f64>,
    pub contract_address: Option<String>,
    pub tx_id: String,
    pub token_symbol: Option<String>,
    pub token_id: Option<String>,
    pub game_id: Uuid,
}

pub async fn create_lobby_handler(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(payload): Json<CreateLobbyPayload>,
) -> Result<Json<Uuid>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Unauthorized access attempt");
        AppError::Unauthorized("Invalid user ID in token".into()).to_response()
    })?;

    let pool = match (
        payload.entry_amount,
        payload.current_amount,
        payload.contract_address.clone(),
    ) {
        (Some(entry_amount), Some(current_amount), Some(contract_address)) => {
            Some(LobbyPoolInput {
                entry_amount,
                current_amount,
                contract_address,
                token_symbol: payload.token_symbol.clone().or(Some("STX".to_string())),
                token_id: payload.token_id.clone(),
            })
        }
        _ => None,
    };

    let lobby_id = create_lobby(
        payload.name,
        payload.description,
        user_id,
        payload.game_id,
        pool,
        payload.tx_id,
        state.redis.clone(),
        state.bot.clone(),
    )
    .await
    .map_err(|err| {
        tracing::error!("Error creating lobby: {}", err);
        err.to_response()
    })?;

    tracing::info!("Lobby created with ID: {}", lobby_id);
    Ok(Json(lobby_id))
}

pub async fn get_lobby_extended_handler(
    Path(lobby_id): Path<Uuid>,
    Query(query): Query<LobbyQuery>,
    State(state): State<AppState>,
) -> Result<Json<LobbyExtended>, (StatusCode, String)> {
    let player_filter = parse_player_state(query.player_state);
    let extended = get_lobby_extended(lobby_id, player_filter, state.redis.clone())
        .await
        .map_err(|e| {
            tracing::error!("Error retrieving extended lobby info: {}", e);
            e.to_response()
        })?;

    tracing::info!("Retrieved extended lobby info for lobby ID: {}", lobby_id);
    Ok(Json(extended))
}

pub async fn get_lobbies_by_game_id_handler(
    Path(game_id): Path<Uuid>,
    Query(query): Query<LobbyQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<LobbyInfo>>, (StatusCode, String)> {
    let lobby_filters = parse_lobby_states(query.lobby_state);

    let (page, limit) = match query.page {
        Some(p) => (p.max(1), query.limit.unwrap_or(12).min(100)),
        None => (1, u32::MAX),
    };

    let lobbies = get_lobbies_by_game_id(game_id, lobby_filters, page, limit, state.redis.clone())
        .await
        .map_err(|e| {
            tracing::error!("Error retrieving lobbies by game ID: {}", e);
            e.to_response()
        })?;

    tracing::info!(
        "Retrieved {} lobbies for game ID: {}",
        lobbies.len(),
        game_id
    );
    Ok(Json(lobbies))
}

pub async fn get_lobby_info_handler(
    Path(lobby_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<LobbyInfo>, (StatusCode, String)> {
    let lobby_info = get_lobby_info(lobby_id, state.redis.clone())
        .await
        .map_err(|e| {
            tracing::error!("Error retrieving lobby info: {}", e);
            e.to_response()
        })?;

    tracing::info!("Retrieved lobby info for {}", lobby_id);
    Ok(Json(lobby_info))
}

pub async fn get_all_lobbies_extended_handler(
    Query(query): Query<LobbyQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<LobbyExtended>>, (StatusCode, String)> {
    let lobby_filters = parse_lobby_states(query.lobby_state);
    let players_filter = parse_player_state(query.player_state);

    let (page, limit) = match query.page {
        Some(p) => (p.max(1), query.limit.unwrap_or(12).min(100)),
        None => (1, u32::MAX),
    };

    let lobbies = get_all_lobbies_extended(
        lobby_filters,
        players_filter,
        page,
        limit,
        state.redis.clone(),
    )
    .await
    .map_err(|e| {
        tracing::error!("Error retrieving all lobbies extended: {}", e);
        e.to_response()
    })?;

    tracing::info!("Retrieved {} extended lobbies", lobbies.len());
    Ok(Json(lobbies))
}

pub async fn get_all_lobbies_info_handler(
    Query(query): Query<LobbyQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<LobbyInfo>>, (StatusCode, String)> {
    let lobby_filters = parse_lobby_states(query.lobby_state);

    let (page, limit) = match query.page {
        Some(p) => (p.max(1), query.limit.unwrap_or(12).min(100)),
        None => (1, u32::MAX),
    };

    let lobbies = get_all_lobbies_info(lobby_filters, page, limit, state.redis.clone())
        .await
        .map_err(|e| {
            tracing::error!("Error retrieving lobbies: {}", e);
            e.to_response()
        })?;

    tracing::info!("Retrieved {} lobbies", lobbies.len());
    Ok(Json(lobbies))
}

pub async fn get_players_handler(
    Path(lobby_id): Path<Uuid>,
    Query(query): Query<PlayerQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Player>>, (StatusCode, String)> {
    let players_filter = parse_player_state(query.player_state.clone());

    let players = get_lobby_players(lobby_id, players_filter, state.redis.clone())
        .await
        .map_err(|e| {
            tracing::error!("Error retrieving players in {lobby_id}: {}", e);
            e.to_response()
        })?;

    tracing::info!("Retrieved {} players", players.len());
    Ok(Json(players))
}

#[derive(Deserialize)]
pub struct JoinLobbyPayload {
    pub tx_id: Option<String>,
}

pub async fn join_lobby_handler(
    Path(lobby_id): Path<Uuid>,
    AuthClaims(claims): AuthClaims,
    State(state): State<AppState>,
    Json(payload): Json<JoinLobbyPayload>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Unauthorized access attempt");
        AppError::Unauthorized("Invalid user ID in token".into()).to_response()
    })?;

    join_lobby(
        lobby_id,
        user_id,
        payload.tx_id,
        PlayerState::Joined,
        state.redis.clone(),
    )
    .await
    .map_err(|e| {
        tracing::error!("Error joining lobby {lobby_id}: {}", e);
        e.to_response()
    })?;

    tracing::info!("Success joining lobby {lobby_id}");
    Ok(Json("success"))
}

pub async fn leave_lobby_handler(
    Path(lobby_id): Path<Uuid>,
    AuthClaims(claims): AuthClaims,
    State(state): State<AppState>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Unauthorized access attempt");
        AppError::Unauthorized("Invalid user ID in token".into()).to_response()
    })?;

    leave_lobby(lobby_id, user_id, state.redis.clone(), state.bot.clone())
        .await
        .map_err(|e| {
            tracing::error!("Error leaving lobby {lobby_id}: {}", e);
            e.to_response()
        })?;

    tracing::info!("Success leaving lobby {lobby_id}");
    Ok(Json("success"))
}

#[derive(Deserialize)]
pub struct KickPlayerPayload {
    pub player_id: Uuid,
}

pub async fn kick_player_handler(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(lobby_id): Path<Uuid>,
    Json(payload): Json<KickPlayerPayload>,
) -> Result<Json<String>, (StatusCode, String)> {
    let caller_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Unauthorized access attempt");
        AppError::Unauthorized("Invalid user ID in token".into()).to_response()
    })?;

    let lobby_info = get_lobby_info(lobby_id, state.redis.clone())
        .await
        .map_err(|e| {
            tracing::error!("Error getting lobby info: {}", e);
            e.to_response()
        })?;

    if lobby_info.creator.id != caller_id {
        return Err({
            tracing::error!("Only the creator can kick players");
            AppError::Unauthorized("Only the creator can kick players".into()).to_response()
        });
    }

    if lobby_info.state != LobbyState::Waiting {
        return Err({
            tracing::error!("Cannot kick players when game is in progress or has ended");
            AppError::BadRequest("Cannot kick players when game is in progress or has ended".into())
                .to_response()
        });
    }

    leave_lobby(
        lobby_id,
        payload.player_id,
        state.redis.clone(),
        state.bot.clone(),
    )
    .await
    .map_err(|e| {
        tracing::error!("Error kicking player: {}", e);
        e.to_response()
    })?;

    tracing::info!("Success kicking player");
    Ok(Json("success".to_string()))
}

#[derive(Deserialize)]
pub struct UpdateLobbyStatePayload {
    pub new_state: LobbyState,
}

pub async fn update_lobby_state_handler(
    Path(lobby_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateLobbyStatePayload>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    update_lobby_state(lobby_id, payload.new_state.clone(), state.redis.clone())
        .await
        .map_err(|e| {
            tracing::error!("Error updating game state: {}", e);
            e.to_response()
        })?;

    tracing::info!("Game state update to {:?}", payload.new_state);
    Ok(Json("success"))
}

#[derive(Deserialize)]
pub struct UpdatePlayerStatePayload {
    pub new_state: PlayerState,
}

pub async fn update_player_state_handler(
    Path(lobby_id): Path<Uuid>,
    AuthClaims(claims): AuthClaims,
    State(state): State<AppState>,
    Json(payload): Json<UpdatePlayerStatePayload>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Unauthorized access attempt");
        AppError::Unauthorized("Invalid user ID in token".into()).to_response()
    })?;

    update_player_state(
        lobby_id,
        user_id,
        payload.new_state.clone(),
        state.redis.clone(),
    )
    .await
    .map_err(|e| {
        tracing::error!("Error updating player state: {}", e);
        e.to_response()
    })?;

    tracing::info!("Player state updated to {:?}", payload.new_state);
    Ok(Json("success"))
}

#[derive(Deserialize)]
pub struct UpdateClaimStatePayload {
    pub claim: ClaimState,
}

pub async fn update_claim_state_handler(
    Path(lobby_id): Path<Uuid>,
    AuthClaims(claims): AuthClaims,
    State(state): State<AppState>,
    Json(payload): Json<UpdateClaimStatePayload>,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Unauthorized access attempt");
        AppError::Unauthorized("Invalid user ID in token".into()).to_response()
    })?;

    update_claim_state(lobby_id, user_id, payload.claim, state.redis.clone())
        .await
        .map_err(|e| {
            tracing::error!("Error updating game state: {}", e);
            e.to_response()
        })?;

    tracing::info!("Claim state updated for lobby {lobby_id}");
    Ok(Json("success"))
}

#[derive(Deserialize)]
pub struct PlayerLobbyQuery {
    pub user_id: Option<Uuid>,
    pub identifier: Option<String>,
    pub lobby_state: Option<String>,
    pub claim_state: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[axum::debug_handler]
pub async fn get_player_lobbies_handler(
    Query(query): Query<PlayerLobbyQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<PlayerLobbyInfo>>, (StatusCode, String)> {
    // Determine the user_id from either direct user_id or identifier
    let user_id = match (query.user_id, query.identifier) {
        (Some(id), _) => {
            // If user_id is provided, use it directly
            id
        }
        (None, Some(identifier)) => {
            // If identifier is provided, look up the user_id
            crate::db::user::get::get_user_id(identifier, state.redis.clone())
                .await
                .map_err(|e| {
                    tracing::error!("Failed to get user ID from identifier: {}", e);
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Failed to get user ID from identifier: {}", e),
                    )
                })?
        }
        (None, None) => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Either user_id or identifier must be provided".to_string(),
            ));
        }
    };

    let claim_filter = parse_claim_state(query.claim_state);
    let lobby_filters = parse_lobby_states(query.lobby_state);
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10).min(100);

    let lobbies = get_player_lobbies(
        user_id,
        claim_filter,
        lobby_filters,
        page,
        limit,
        state.redis,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to get player lobbies for {}: {}", user_id, e);
        e.to_response()
    })?;

    Ok(Json(lobbies))
}

fn parse_claim_state(claim_param: Option<String>) -> Option<ClaimState> {
    claim_param.and_then(|s| match s.to_lowercase().as_str() {
        "claimed" => Some(ClaimState::Claimed {
            tx_id: String::new(),
        }), // We'll match any claimed state
        "notclaimed" | "not_claimed" => Some(ClaimState::NotClaimed),
        other => {
            tracing::warn!("Invalid claim_state filter: {}", other);
            None
        }
    })
}
