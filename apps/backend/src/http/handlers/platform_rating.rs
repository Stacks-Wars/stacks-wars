use axum::{Json, extract::Path, extract::Query, extract::State, http::StatusCode};
use serde::Deserialize;
use uuid::Uuid;

use crate::{auth::AuthClaims, db::platform_rating::PlatformRatingRepository, state::AppState};

// Request/Response types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlatformRatingRequest {
    pub rating: i16,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlatformRatingRequest {
    pub rating: i16,
    pub comment: Option<String>,
}

// Create or replace rating for authenticated user
pub async fn create_rating(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(payload): Json<CreatePlatformRatingRequest>,
) -> Result<(StatusCode, Json<()>), (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    let repo = PlatformRatingRepository::new(state.postgres.clone());

    repo.create_rating(user_id, payload.rating, payload.comment.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create platform rating: {}", e);
            e.to_response()
        })?;

    Ok((StatusCode::CREATED, Json(())))
}

// Get rating by user id (public)
pub async fn get_rating(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<crate::models::db::PlatformRating>, (StatusCode, String)> {
    let repo = PlatformRatingRepository::new(state.postgres.clone());

    match repo.get_by_user(user_id).await.map_err(|e| {
        tracing::error!("Failed to fetch platform rating: {}", e);
        e.to_response()
    })? {
        Some(r) => Ok(Json(r)),
        None => Err((StatusCode::NOT_FOUND, "Not found".to_string())),
    }
}

// List ratings (public) with optional `?rating=1..5` filter
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRatingsQuery {
    pub rating: Option<i16>,
}

pub async fn list_ratings(
    State(state): State<AppState>,
    Query(query): Query<ListRatingsQuery>,
) -> Result<Json<Vec<crate::models::db::PlatformRating>>, (StatusCode, String)> {
    if let Some(r) = query.rating {
        if !(1..=5).contains(&r) {
            return Err((
                StatusCode::BAD_REQUEST,
                "rating must be between 1 and 5".to_string(),
            ));
        }
    }

    let repo = PlatformRatingRepository::new(state.postgres.clone());

    let list = repo.list(query.rating).await.map_err(|e| {
        tracing::error!("Failed to list platform ratings: {}", e);
        e.to_response()
    })?;

    Ok(Json(list))
}

// Update rating for authenticated user
pub async fn update_rating(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(payload): Json<UpdatePlatformRatingRequest>,
) -> Result<Json<crate::models::db::PlatformRating>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    let repo = PlatformRatingRepository::new(state.postgres.clone());

    let updated = repo
        .update_rating(user_id, payload.rating, payload.comment.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to update platform rating: {}", e);
            e.to_response()
        })?;

    Ok(Json(updated))
}

// Delete rating for authenticated user
pub async fn delete_rating(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    let repo = PlatformRatingRepository::new(state.postgres.clone());

    repo.delete_by_user(user_id).await.map_err(|e| {
        tracing::error!("Failed to delete platform rating: {}", e);
        e.to_response()
    })?;

    Ok(StatusCode::NO_CONTENT)
}
