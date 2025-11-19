// Authenticated write routes (mounted under `/api`)
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{delete, patch, post},
};

use crate::{
    http::handlers::{
        game::create_game,
        lobby::{create_lobby, delete_lobby},
        season::create_season,
        user::{update_display_name, update_profile, update_username},
    },
    middleware::{AuthRateLimit, rate_limit_with_state},
    state::AppState,
};

pub fn routes(state_for_layer: AppState) -> Router<AppState> {
    Router::new()
        .route("/user/profile", patch(update_profile))
        .route("/user/username", patch(update_username))
        .route("/user/display-name", patch(update_display_name))
        .route("/game", post(create_game))
        .route("/lobby", post(create_lobby))
        .route("/lobby/{lobby_id}", delete(delete_lobby))
        .route("/season", post(create_season))
        .layer(from_fn_with_state(
            state_for_layer.clone(),
            rate_limit_with_state::<AuthRateLimit>,
        ))
}
