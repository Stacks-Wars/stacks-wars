//! # Game HTTP Handlers
//!
//! HTTP request handlers for game type management.
//!
//! ## What is a Game?
//! A game represents a game mode/type available on the platform (e.g., "Lexi Wars", "Word Battle").
//! Each game defines:
//! - Rules and mechanics
//! - Player limits (min/max)
//! - Category/genre
//! - Active status
//!
//! ## Endpoints Provided
//! - Game creation (admin)
//! - Game retrieval (single/list)
//! - Game filtering by category
//!
//! ## Architecture
//! ```text
//! HTTP Request → Handler → GameRepository → PostgreSQL
//!                                ↓
//!                           Response
//! ```

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
    models::db::game::{Game, Order, Pagination},
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

/// Create a new game type
///
/// Adds a new game mode to the platform. This endpoint should be restricted
/// to admin users in production.
///
/// ## Authentication
/// Required (JWT token in Authorization header)
///
/// ## Request
/// ```json
/// {
///   "name": "Lexi Wars",
///   "description": "Fast-paced word battle game",
///   "imageUrl": "https://example.com/lexi-wars.png",
///   "minPlayers": 2,
///   "maxPlayers": 10,
///   "category": "Word Games"
/// }
/// ```
///
/// ## Response
/// Returns the created game with generated ID and timestamps.
///
/// ## Errors
/// - `400 Bad Request` - Invalid input (e.g., min_players > max_players)
/// - `401 Unauthorized` - Invalid or missing JWT token
/// - `409 Conflict` - Game name already exists
/// - `500 Internal Server Error` - Database error
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
            payload.name,
            payload.description,
            payload.image_url,
            payload.min_players as i16,
            payload.max_players as i16,
            payload.category,
            creator_id,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create game: {}", e);
            e.to_response()
        })?;

    tracing::info!("Game created - ID: {}, Name: '{}'", game.id, game.name);
    Ok(Json(game))
}

// ============================================================================
// Game Retrieval
// ============================================================================

/// Get a single game by ID
///
/// Retrieves detailed information about a specific game type.
///
/// ## Authentication
/// None required (public endpoint)
///
/// ## Path Parameters
/// - `game_id` - UUID of the game
///
/// ## Response
/// ```json
/// {
///   "id": "550e8400-e29b-41d4-a716-446655440000",
///   "name": "Lexi Wars",
///   "description": "Fast-paced word battle game",
///   "imageUrl": "https://example.com/lexi-wars.png",
///   "minPlayers": 2,
///   "maxPlayers": 10,
///   "category": "Word Games",
///   "creatorId": "...",
///   "isActive": true,
///   "createdAt": "2024-01-01T00:00:00Z",
///   "updatedAt": "2024-01-15T12:30:00Z"
/// }
/// ```
///
/// ## Errors
/// - `404 Not Found` - Game doesn't exist
/// - `500 Internal Server Error` - Database error
pub async fn get_game(
    Path(game_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Game>, (StatusCode, String)> {
    let repo = GameRepository::new(state.postgres.clone());

    let game = repo.find_by_id(game_id).await.map_err(|e| {
        tracing::error!("Failed to fetch game {}: {}", game_id, e);
        e.to_response()
    })?;

    tracing::debug!("Retrieved game: {}", game_id);
    Ok(Json(game))
}

/// Get list of games with pagination
///
/// Retrieves all game types available on the platform.
///
/// ## Authentication
/// None required (public endpoint)
///
/// ## Query Parameters
/// - `page` - Page number (default: 1)
/// - `limit` - Items per page (default: 20)
/// - `order` - Sort order: "asc" or "desc" (default: "desc")
///
/// ## Response
/// Returns array of games, sorted by creation date.
///
/// ## Example
/// ```
/// GET /api/game?page=1&limit=10&order=desc
/// ```
///
/// ## Errors
/// - `400 Bad Request` - Invalid query parameters
/// - `500 Internal Server Error` - Database error
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

    let games = repo.get_all_games(pagination, order).await.map_err(|e| {
        tracing::error!("Failed to fetch games: {}", e);
        e.to_response()
    })?;

    tracing::debug!(
        "Retrieved {} games (page {}, limit {})",
        games.len(),
        query.page,
        query.limit
    );
    Ok(Json(games))
}
