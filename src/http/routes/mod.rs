//! # HTTP Routes Module
//!
//! This module organizes all HTTP routes by protection and rate limiting level.
//!
//! ## Route Organization
//!
//! ### Public Routes (`public.rs`)
//! - **Rate Limit**: None
//! - **Auth**: Not required
//! - **Purpose**: Health checks, API info, monitoring
//! - **Examples**: `/health`, `/`
//!
//! ### API Routes (`api.rs`)
//! - **Rate Limit**: 1000 requests/minute per IP (moderate)
//! - **Auth**: Most don't require auth (public read operations)
//! - **Purpose**: Reading platform data
//! - **Examples**: `/api/user/:id`, `/api/game`, `/api/lobby/:id`
//!
//! ### Auth Routes (`auth.rs`)
//! - **Rate Limit**: 300 requests/minute per IP (strict)
//! - **Auth**: Required (JWT token)
//! - **Purpose**: Write operations, authenticated actions
//! - **Examples**: `/user` (POST), `/game` (POST), `/lobby/:id/join`
//!
//! ## Migration Status
//! - ✅ **User endpoints**: Fully refactored to use `UserRepository`
//! - ✅ **Game endpoints**: Fully refactored to use `GameRepository`
//! - ⚠️ **Lobby endpoints**: Legacy implementation (uses old Redis patterns)
//! - ⚠️ **Leaderboard endpoints**: Legacy implementation
//! - ⚠️ **Token info endpoints**: Legacy implementation
//!
//! ## For Contributors
//! When adding new endpoints:
//! 1. Choose the correct route file based on auth/rate limit needs
//! 2. Create handlers in `src/http/handlers/<domain>.rs`
//! 3. Use repository pattern (see `UserRepository`, `GameRepository`)
//! 4. Add comprehensive documentation with examples
//! 5. Follow naming convention (no `_handler` suffix)

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
