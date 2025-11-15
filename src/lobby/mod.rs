//! Lobby domain module
//!
//! Contains lobby-specific managers and domain logic (socket-level manager,
//! lifecycle, state transitions). This keeps `src/ws` for generic WebSocket
//! primitives while `src/lobby` holds game-agnostic lobby behavior.
pub mod handler;
pub mod manager;
pub use manager::*;
