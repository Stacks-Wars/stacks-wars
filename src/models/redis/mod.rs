//! Redis models for cached and transient data
//!
//! This module contains models stored in Redis for fast access during gameplay.
//!
//! ## Architecture
//!
//! ### Platform State (Generic - All Games)
//! - `LobbyState` - Runtime lobby state (status, participants, timing)
//! - `PlayerState` - Generic player state (rank, prize, connection)
//!
//! ### Legacy Models (To be deprecated)
//! - `LobbyInfo` - Old mixed model (being replaced by Lobby + LobbyState)
//! - `Player` - Old mixed model with game-specific fields (being replaced by PlayerState + GameState)
//!
//! ### Game-Specific State
//! - See `games` module for game-specific state implementations

pub mod game; // Legacy models
pub mod keys;
pub mod lobby_state; // New state models
pub mod player_state; // New state models

// New state models (Phase 2)
pub use lobby_state::LobbyState;
pub use player_state::PlayerState;

// Legacy exports (will be deprecated after full migration)
pub use game::{GameType, LobbyInfo, Player};
pub use keys::RedisKey;
