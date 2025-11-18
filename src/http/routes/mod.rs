//! HTTP routing for the Stacks Wars API.
//!
//! Composes the application's route groups and exposes them under a single
//! router. Sub-routers:
//! - `public`  : health and non-rate-limited endpoints
//! - `api`     : read-focused endpoints (mounted under `/api`)
//! - `auth`    : authenticated write endpoints (mounted under `/api`)
//! - `strict`  : sensitive write endpoints (mounted under `/api`)
//!
//! `create_http_routes` composes these routers and attaches the shared
//! `AppState`.

/// Create the main HTTP router with all routes
use crate::state::AppState;
use axum::Router;

pub mod api;
pub mod auth;
pub mod public;
pub mod strict;

/// Create the main HTTP router with all routes
///
/// Combines all route groups into a single router with shared state.
pub fn create_http_routes(state: AppState) -> Router {
    // clone the state for attaching to the middleware via from_fn_with_state
    let state_for_layer = state.clone();

    // Build sub-routers that will all be exposed under `/api`.
    let api_router = api::routes(state_for_layer.clone());

    let auth_router = auth::routes(state_for_layer.clone());

    let strict_router = strict::routes(state_for_layer.clone());

    Router::new()
        // Public routes (no rate limiting, no auth)
        .merge(public::routes())
        // Expose API, Auth and Strict routers under a single `/api` prefix
        // so clients only need to call `/api/*` paths.
        .nest(
            "/api",
            Router::new()
                .merge(api_router)
                .merge(auth_router)
                .merge(strict_router),
        )
        .with_state(state)
}
