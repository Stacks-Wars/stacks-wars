use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{patch, post, put},
};

use crate::{
    http::handlers::{
        game::toggle_game_active,
        season::{create_season, update_season},
    },
    middleware::{AuthRateLimit, rate_limit_with_state},
    state::AppState,
};

/// Admin routes - all require authentication + admin wallet
pub fn routes(state_for_layer: AppState) -> Router<AppState> {
    Router::new()
        .route("/season", post(create_season))
        .route("/season/{season_id}", put(update_season))
        .route("/game/{game_id}/active", patch(toggle_game_active))
        .layer(from_fn_with_state(
            state_for_layer.clone(),
            rate_limit_with_state::<AuthRateLimit>,
        ))
}
