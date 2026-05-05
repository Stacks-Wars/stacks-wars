// Season management handlers: create/list/get current season

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;

use crate::{
    auth::extractors::AuthClaims, db::season::SeasonRepository, models::Season, state::AppState,
};

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

/// Request payload for updating a season
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSeasonRequest {
    /// New season name (optional)
    pub name: Option<String>,
    /// New description (optional)
    pub description: Option<String>,
    /// New start date in format: "YYYY-MM-DD HH:MM:SS" (optional)
    pub start_date: Option<String>,
    /// New end date in format: "YYYY-MM-DD HH:MM:SS" (optional)
    pub end_date: Option<String>,
}

// ============================================================================
// Helpers
// ============================================================================

/// Check if the authenticated user is an admin
fn require_admin(state: &AppState, auth: &AuthClaims) -> Result<(), (StatusCode, String)> {
    if !state.config.is_admin(auth.wallet_address()) {
        return Err((StatusCode::FORBIDDEN, "Admin access required".to_string()));
    }
    Ok(())
}

// ============================================================================
// Handlers
// ============================================================================

/// Create a new competitive season (admin only)
pub async fn create_season(
    State(state): State<AppState>,
    auth: AuthClaims,
    Json(payload): Json<CreateSeasonRequest>,
) -> Result<Json<Season>, (StatusCode, String)> {
    // Admin check
    require_admin(&state, &auth)?;

    let repo = SeasonRepository::new(state.postgres.clone());
    let season = repo
        .create_season(
            &payload.name,
            payload.description.as_deref(),
            &payload.start_date,
            &payload.end_date,
        )
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(season))
}

/// Update an existing season (admin only)
pub async fn update_season(
    State(state): State<AppState>,
    auth: AuthClaims,
    Path(season_id): Path<i32>,
    Json(payload): Json<UpdateSeasonRequest>,
) -> Result<Json<Season>, (StatusCode, String)> {
    // Admin check
    require_admin(&state, &auth)?;

    let repo = SeasonRepository::new(state.postgres.clone());

    // Parse dates if provided
    let start_date = payload
        .start_date
        .map(|s| {
            chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Invalid start_date format: {}", e),
                )
            })
        })
        .transpose()?;

    let end_date = payload
        .end_date
        .map(|s| {
            chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Invalid end_date format: {}", e),
                )
            })
        })
        .transpose()?;

    let season = repo
        .update_season(
            season_id,
            payload.name,
            payload.description,
            start_date,
            end_date,
        )
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(season))
}

/// Get the current active season (returns 404 if none)
pub async fn get_current_season(
    State(state): State<AppState>,
) -> Result<Json<Season>, (StatusCode, String)> {
    let repo = SeasonRepository::new(state.postgres);
    let season = repo
        .get_current_season()
        .await
        .map_err(|e| e.to_response())?;

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
    Query(query): Query<PaginationQuery>,
) -> Result<Json<Vec<Season>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(10).min(100);
    let offset = query.offset.unwrap_or(0).max(0);

    let repo = SeasonRepository::new(state.postgres);
    let seasons = repo
        .get_all_seasons(limit, offset)
        .await
        .map_err(|e| e.to_response())?;

    Ok(Json(seasons))
}
