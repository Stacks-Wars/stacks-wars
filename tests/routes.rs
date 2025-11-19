// Convenience test entry exposing per-route integration tests
// Run with: `cargo test --test routes`

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
