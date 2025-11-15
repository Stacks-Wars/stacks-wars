//! WebSocket entry points and helpers
//!
//! Keeps the `ws` surface small: generic core primitives live under
//! `src/ws/core` and per-handler logic under `src/ws/handlers`.
pub mod core;
pub mod handlers;
pub mod routes;

pub use routes::create_ws_routes;
