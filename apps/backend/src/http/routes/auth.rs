// Authenticated write routes (mounted under `/api`)
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{delete, get, patch, post},
};

use crate::{
    http::handlers::{
        game::create_game,
        lobby::{create_lobby, delete_lobby},
        platform_rating::{create_rating, delete_rating, update_rating},
        season::create_season,
        user::{get_me, logout, update_display_name, update_profile, update_username},
    },
    middleware::{AuthRateLimit, rate_limit_with_state},
    state::AppState,
};

pub fn routes(state_for_layer: AppState) -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me))
        .route("/user/profile", patch(update_profile))
        .route("/platform-rating", post(create_rating))
        .route("/platform-rating", patch(update_rating))
        .route("/platform-rating", delete(delete_rating))
        .route("/user/username", patch(update_username))
        .route("/user/display-name", patch(update_display_name))
        .route("/game", post(create_game))
        .route("/lobby", post(create_lobby))
        .route("/lobby/{lobby_id}", delete(delete_lobby))
        .route("/season", post(create_season))
        .route("/logout", post(logout))
        .layer(from_fn_with_state(
            state_for_layer.clone(),
            rate_limit_with_state::<AuthRateLimit>,
        ))
}
