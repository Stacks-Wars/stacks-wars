use axum::{
    Router,
    routing::{delete, get, patch, post},
};

use crate::state::AppState;

use super::handlers::{create_rating, delete_rating, get_rating, list_ratings, update_rating};

/// Mounted at `/api/ratings`.
pub fn read_routes() -> Router<AppState> {
    Router::new()
        .route("/platform-rating", get(list_ratings))
        .route("/platform-rating/{user_id}", get(get_rating))
}

/// Mounted at `/api/ratings` (auth rate limit).
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/platform-rating", post(create_rating))
        .route("/platform-rating", patch(update_rating))
        .route("/platform-rating", delete(delete_rating))
}
