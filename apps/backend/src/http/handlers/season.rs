// Season management handlers: create/list/get current season

use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::{db::season::SeasonRepository, models::db::Season, state::AppState};

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

/// Create a new competitive season (admin only)
pub async fn create_season(
    State(state): State<AppState>,
    Json(payload): Json<CreateSeasonRequest>,
) -> Result<Json<CreateSeasonResponse>, (StatusCode, String)> {
    let repo = SeasonRepository::new(state.postgres.clone());
    let season = repo
        .create_season(
            &payload.name,
            payload.description.as_deref(),
            &payload.start_date,
            &payload.end_date,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create season: {}", e);
            e.to_response()
        })?;

    tracing::info!("Season created: {} (ID: {})", season.name, season.id);
    Ok(Json(CreateSeasonResponse { season }))
}

/// Get the current active season (returns 404 if none)
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
/// Supports `limit` and `offset` query params; returns a vector of `Season`.
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
