use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{post, put},
};

use crate::{
    http::handlers::season::{create_season, update_season},
    middleware::{AuthRateLimit, rate_limit_with_state},
    state::AppState,
};

/// Admin routes - all require authentication + admin wallet
pub fn routes(state_for_layer: AppState) -> Router<AppState> {
    Router::new()
        .route("/season", post(create_season))
        .route("/season/{season_id}", put(update_season))
        .layer(from_fn_with_state(
            state_for_layer.clone(),
            rate_limit_with_state::<AuthRateLimit>,
        ))
}
