//! HTTP routing for the Stacks Wars API.
//!
//! This crate exposes three route groups that are composed into the main
//! application router:
//!
//! - `public` (no auth, no rate limit) - health checks and basic API info
//! - `api` (read-focused, moderate rate limit) - public read endpoints such as
//!   user, game, lobby and season queries. Note: a small number of read routes
//!   may still require authentication (see the route definitions).
//! - `auth` (write-focused, strict rate limit) - authenticated endpoints for
//!   creating/updating resources (users, games, lobbies). All routes under
//!   this group are protected by JWT authentication.
//!
//! Exposed endpoints (high level):
//! - Users: POST `/auth/user`, GET `/api/user/{id}`, PATCH `/auth/user/profile`,
//!   PATCH `/auth/user/username`, PATCH `/auth/user/display-name`.
//! - Games: POST `/auth/game`, GET `/api/game`, GET `/api/game/{id}`.
//! - Lobbies: POST `/auth/lobby`, GET `/api/lobby/{id}`, GET `/api/game/{id}/lobbies`,
//!   GET `/api/lobby/my` (see implementation for auth requirements), DELETE `/auth/lobby/{id}`.
//! - Seasons: GET `/api/season` and GET `/api/season/current`.
//! - Token info: GET `/api/token/{contract_address}` and `/api/token/testnet/{contract_address}`.
//!
//! Notes for contributors:
//! - Prefer the repository pattern in `src/db/*` for data access.
//! - Keep handler-level docs concise and document required auth and inputs/outputs.
//! - Legacy/removed features (e.g. older Redis-only leaderboards) are deprecated
//!   and should not be relied on when adding new endpoints.

use crate::state::AppState;
use axum::Router;

pub mod api;
pub mod auth;
pub mod public;

/// Create the main HTTP router with all routes
///
/// Combines all route groups into a single router with shared state.
pub fn create_http_routes(state: AppState) -> Router {
    Router::new()
        // Public routes (no rate limiting, no auth)
        .merge(public::routes())
        // Read routes (moderate rate limit: 1000/min, mostly no auth)
        .nest("/api", api::routes())
        // Write routes (strict rate limit: 300/min, auth required)
        .nest("/auth", auth::routes())
        .with_state(state)
}
