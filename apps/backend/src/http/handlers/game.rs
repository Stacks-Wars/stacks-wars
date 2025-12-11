// Game HTTP handlers: create, retrieve, and list game types

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::AuthClaims,
    db::game::GameRepository,
    errors::AppError,
    models::game::{Game, Order, Pagination},
    state::AppState,
};

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request body for creating a new game type
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGameRequest {
    /// Game name (must be unique)
    pub name: String,
    /// URL-friendly path (must be unique, lowercase alphanumeric + hyphens)
    pub path: String,
    /// Game description
    pub description: String,
    /// URL to game thumbnail/icon
    pub image_url: String,
    /// Minimum players required
    pub min_players: u8,
    /// Maximum players allowed
    pub max_players: u8,
    /// Game category/genre (e.g., "Word Games", "Strategy")
    pub category: Option<String>,
}

/// Query parameters for listing games
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListGamesQuery {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Items per page
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// Sort order: "asc" or "desc"
    #[serde(default)]
    pub order: Option<String>,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    20
}

// ============================================================================
// Game Creation (Admin)
// ============================================================================

/// Create a new game type (admin only).
///
/// Requires a valid admin JWT; returns the created `Game` on success.
pub async fn create_game(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(payload): Json<CreateGameRequest>,
) -> Result<Json<Game>, (StatusCode, String)> {
    let creator_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Invalid user ID in JWT token");
        AppError::Unauthorized("Invalid token".into()).to_response()
    })?;

    let repo = GameRepository::new(state.postgres.clone());

    let game = repo
        .create_game(
            &payload.name,
            &payload.path,
            &payload.description,
            &payload.image_url,
            payload.min_players as i16,
            payload.max_players as i16,
            payload.category.as_deref(),
            creator_id,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create game: {}", e);
            e.to_response()
        })?;

    Ok(Json(game))
}

// ============================================================================
// Game Retrieval
// ============================================================================

/// Get a game by UUID. Returns `Game` or `404` if not found.
pub async fn get_game(
    Path(game_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Game>, (StatusCode, String)> {
    let repo = GameRepository::new(state.postgres.clone());

    let game = repo
        .find_by_id(game_id)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(game))
}

/// Get a game by path. Returns `Game` or `404` if not found.
pub async fn get_game_by_path(
    Path(path): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Game>, (StatusCode, String)> {
    let repo = GameRepository::new(state.postgres.clone());

    let game = repo
        .find_by_path(&path)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(game))
}

/// Get games by creator ID. Returns array of `Game`.
pub async fn get_games_by_creator(
    Path(creator_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Game>>, (StatusCode, String)> {
    let repo = GameRepository::new(state.postgres.clone());

    let games = repo
        .get_by_creator(creator_id, 100)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(games))
}

/// List games with pagination. Public endpoint returning an array of `Game`.
pub async fn list_games(
    State(state): State<AppState>,
    Query(query): Query<ListGamesQuery>,
) -> Result<Json<Vec<Game>>, (StatusCode, String)> {
    let pagination = Pagination {
        page: query.page as i64,
        limit: query.limit as i64,
    };

    let order = query
        .order
        .as_deref()
        .and_then(|s| s.parse::<Order>().ok())
        .unwrap_or(Order::Descending);

    let repo = GameRepository::new(state.postgres.clone());

    let games = repo
        .get_all_games(pagination, order)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(games))
}
