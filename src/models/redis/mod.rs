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

pub mod keys;
pub mod lobby_state;
pub mod player_state;
pub mod spectator_state; // Spectator state model (like PlayerState but for spectators)

pub use lobby_state::LobbyState;
pub use lobby_state::LobbyStatus;
pub use player_state::PlayerState;
