//! Database repositories and helpers (Postgres-first).
//!
//! The repository layout is intentionally flat and small - each domain has a
//! module that exposes CRUD functions used by HTTP handlers. Prefer these
//! modules for data access instead of touching SQL directly in handlers.
//!
//! Active modules:
//! - `game`, `lobby`, `user`, `season`, `user_wars_points` - Postgres-backed repositories
//! - `lobby_state`, `player_state` - Redis-backed ephemeral state repositories
//! - `hydration` - one-time helpers to migrate Redis state into Postgres
//!
//! Note: `leaderboard` is legacy (Redis-backed) and should be considered
//! deprecated for new features; prefer adding a Postgres-backed leaderboard
//! repository under `db/` if persistent rankings are required.
pub mod chat;
pub mod game;
pub mod hydration;
pub mod lobby;
pub mod lobby_state;
pub mod player_state;
pub mod season;
pub mod spectator_state;
pub mod tx;
pub mod user;
pub mod user_wars_points;
