use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::state::AppState;

use super::handlers::{
    create_user, get_player_lobbies, get_user, update_display_name, update_profile, update_username,
};

/// Mounted at `/api/users`.
pub fn read_routes() -> Router<AppState> {
    Router::new()
        .route("/user/{user_id}", get(get_user))
        .route("/player-lobby/{user_id}", get(get_player_lobbies))
}

/// Mounted at `/api/users` (auth rate limit).
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/user/profile", patch(update_profile))
        .route("/user/username", patch(update_username))
        .route("/user/display-name", patch(update_display_name))
}

/// Mounted at `/api/users` (strict rate limit).
pub fn strict_routes() -> Router<AppState> {
    Router::new().route("/register", post(create_user))
}
