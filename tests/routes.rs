//! Convenience test entry that exposes the per-route integration tests
//! located in `tests/routes/` so they can be run independently.
//!
//! Run only these with:
//!
//!     cargo test --test routes

#[path = "common/mod.rs"]
mod common;

#[path = "routes/game.rs"]
mod game;

#[path = "routes/lobby.rs"]
mod lobby;

#[path = "routes/season.rs"]
mod season;

#[path = "routes/user.rs"]
mod user;
