//! Season Management Handlers
//!
//! Handles season (competitive periods) management operations.
//! Seasons define time-bound competitive periods with leaderboards and rewards.

use axum::{Json, extract::State, http::StatusCode};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{db::season::SeasonRepository, models::season::Season, state::AppState};

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request payload for creating a new season
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSeasonRequest {
    /// Season name (e.g., "Season 1: Winter Wars")
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Start date in format: "YYYY-MM-DD HH:MM:SS"
    pub start_date: String,
    /// End date in format: "YYYY-MM-DD HH:MM:SS"
    pub end_date: String,
}

/// Response after creating a season
#[derive(Debug, Serialize)]
pub struct CreateSeasonResponse {
    /// The created season
    #[serde(flatten)]
    pub season: Season,
}

// ============================================================================
// Handlers
// ============================================================================

/// Create a new competitive season
///
/// Creates a new time-bound season for competitive play. Seasons help organize
/// leaderboards and competitive periods for the platform.
///
/// # Authentication
/// - **Required**: Yes (admin/platform access only)
///
/// # Request Body
/// ```json
/// {
///   "name": "Season 1: Winter Championship",
///   "description": "First competitive season with $10k prize pool",
///   "startDate": "2024-01-01 00:00:00",
///   "endDate": "2024-03-31 23:59:59"
/// }
/// ```
///
/// # Response
/// - **200 OK**: Season created successfully
/// ```json
/// {
///   "id": 1,
///   "name": "Season 1: Winter Championship",
///   "description": "First competitive season with $10k prize pool",
///   "startDate": "2024-01-01T00:00:00",
///   "endDate": "2024-03-31T23:59:59",
///   "createdAt": "2024-01-15T10:30:00Z"
/// }
/// ```
///
/// # Errors
/// - **400 Bad Request**: Invalid date format or dates
/// - **401 Unauthorized**: Not authenticated or insufficient permissions
/// - **500 Internal Server Error**: Database error
pub async fn create_season(
    State(state): State<AppState>,
    Json(payload): Json<CreateSeasonRequest>,
) -> Result<Json<CreateSeasonResponse>, (StatusCode, String)> {
    // Parse and validate dates
    let start_date = NaiveDateTime::parse_from_str(&payload.start_date, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| {
            tracing::error!("Invalid start_date format: {}", e);
            (
                StatusCode::BAD_REQUEST,
                format!(
                    "Invalid start_date format: {}. Expected: YYYY-MM-DD HH:MM:SS",
                    e
                ),
            )
        })?;

    let end_date =
        NaiveDateTime::parse_from_str(&payload.end_date, "%Y-%m-%d %H:%M:%SS").map_err(|e| {
            tracing::error!("Invalid end_date format: {}", e);
            (
                StatusCode::BAD_REQUEST,
                format!(
                    "Invalid end_date format: {}. Expected: YYYY-MM-DD HH:MM:SS",
                    e
                ),
            )
        })?;

    // Validate date logic
    if end_date <= start_date {
        tracing::error!("End date must be after start date");
        return Err((
            StatusCode::BAD_REQUEST,
            "End date must be after start date".to_string(),
        ));
    }

    // Create season using repository
    let repo = SeasonRepository::new(state.postgres.clone());
    let season = repo
        .create_season(
            payload.name.clone(),
            payload.description,
            start_date,
            end_date,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create season: {}", e);
            e.to_response()
        })?;

    tracing::info!("Season created: {} (ID: {})", season.name, season.id);
    Ok(Json(CreateSeasonResponse { season }))
}

/// Get the current active season
///
/// Returns the currently active competitive season based on current timestamp.
///
/// # Authentication
/// - **Required**: No
///
/// # Response
/// - **200 OK**: Current season found
/// ```json
/// {
///   "id": 1,
///   "name": "Season 1: Winter Championship",
///   "startDate": "2024-01-01T00:00:00",
///   "endDate": "2024-03-31T23:59:59"
/// }
/// ```
///
/// # Errors
/// - **404 Not Found**: No active season
/// - **500 Internal Server Error**: Database error
pub async fn get_current_season(
    State(state): State<AppState>,
) -> Result<Json<Season>, (StatusCode, String)> {
    let repo = SeasonRepository::new(state.postgres);
    let season = repo.get_current_season_full().await.map_err(|e| {
        tracing::error!("Failed to get current season: {}", e);
        e.to_response()
    })?;

    tracing::info!("Retrieved current season: {}", season.name);
    Ok(Json(season))
}

/// List all seasons with pagination
///
/// Returns paginated list of all seasons (past, current, and future).
///
/// # Authentication
/// - **Required**: No
///
/// # Query Parameters
/// - `limit` (optional): Number of results (default: 10, max: 100)
/// - `offset` (optional): Pagination offset (default: 0)
///
/// # Response
/// - **200 OK**: List of seasons
/// ```json
/// [
///   {
///     "id": 1,
///     "name": "Season 1",
///     "startDate": "2024-01-01T00:00:00",
///     "endDate": "2024-03-31T23:59:59"
///   }
/// ]
/// ```
///
/// # Errors
/// - **500 Internal Server Error**: Database error
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_seasons(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<PaginationQuery>,
) -> Result<Json<Vec<Season>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(10).min(100);
    let offset = query.offset.unwrap_or(0).max(0);

    let repo = SeasonRepository::new(state.postgres);
    let seasons = repo.get_all_seasons(limit, offset).await.map_err(|e| {
        tracing::error!("Failed to list seasons: {}", e);
        e.to_response()
    })?;

    tracing::info!("Retrieved {} seasons", seasons.len());
    Ok(Json(seasons))
}
