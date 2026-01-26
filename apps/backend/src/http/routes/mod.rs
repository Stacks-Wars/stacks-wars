// Main HTTP routing: compose and mount sub-routers under `/api`.
use crate::state::AppState;
use axum::Router;

pub mod admin;
pub mod api;
pub mod auth;
pub mod public;
pub mod strict;

/// Build the top-level router and nest `api`, `auth`, `strict`, and `admin` under `/api`.
pub fn create_http_routes(state: AppState) -> Router {
    // clone the state for attaching to the middleware via from_fn_with_state
    let state_for_layer = state.clone();

    // Build sub-routers that will all be exposed under `/api`.
    let api_router = api::routes(state_for_layer.clone());

    let auth_router = auth::routes(state_for_layer.clone());

    let strict_router = strict::routes(state_for_layer.clone());

    let admin_router = admin::routes(state_for_layer.clone());

    Router::new()
        // Public routes (no rate limiting, no auth)
        .merge(public::routes())
        // Expose API, Auth, Strict, and Admin routers under a single `/api` prefix
        // so clients only need to call `/api/*` paths.
        .nest(
            "/api",
            Router::new()
                .merge(api_router)
                .merge(auth_router)
                .merge(strict_router)
                .merge(admin_router),
        )
        .with_state(state)
}
