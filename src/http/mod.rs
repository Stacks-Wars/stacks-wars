//! HTTP layer for the Stacks Wars backend.
//!
//! This module groups the HTTP request handlers and route definitions used to
//! build the Axum router. It intentionally keeps wiring simple:
//!
//! - `handlers` contains domain-specific request handlers (users, games, lobbies,
//!   seasons, token info, etc.). Handlers are small adapters that call into the
//!   repository layer under `src/db/`.
//! - `routes` composes handler functions into route groups (public, api, auth)
//!   and exposes `create_http_routes(state)` which returns a ready-to-use
//!   `axum::Router<AppState>` for the application.
//!
//! Export the top-level router constructor so binaries can start the server
//! with a single import:
//!
//! ```rust
//! use stacks_wars_be::http::create_http_routes;
//! ```
pub mod handlers;
pub mod routes;

pub use routes::create_http_routes;
