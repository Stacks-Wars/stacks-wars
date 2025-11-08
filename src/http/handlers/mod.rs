//! HTTP Handlers
//!
//! Production-ready HTTP handlers using repository pattern.
//!
//! ## Modules
//! - `user` - User management (`UserRepository`)
//! - `game` - Game types (`GameRepository`)
//! - `lobby` - Multiplayer lobbies (`LobbyRepository`)
//! - `season` - Competitive seasons (`SeasonRepository`)
//! - `token_info` - Token pricing (external API)

pub mod game;
pub mod lobby;
pub mod season;
pub mod token_info;
pub mod user;
