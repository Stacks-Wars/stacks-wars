// Convenience test entry exposing per-route integration tests
// Run with: `cargo test --test http_routes`

#[path = "common/mod.rs"]
mod common;

#[path = "http_routes/game.rs"]
mod game;

#[path = "http_routes/lobby.rs"]
mod lobby;

#[path = "http_routes/season.rs"]
mod season;

#[path = "http_routes/user.rs"]
mod user;

#[path = "http_routes/platform_rating.rs"]
mod platform_rating;
