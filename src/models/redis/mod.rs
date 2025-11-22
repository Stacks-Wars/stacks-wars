// Redis models for cached/transient gameplay data (LobbyState, PlayerState, ...)

pub mod keys;
pub mod lobby_state;
pub mod player_state;
pub mod spectator_state; // Spectator state model (like PlayerState but for spectators)

pub use lobby_state::LobbyState;
pub use lobby_state::LobbyStatus;
pub use player_state::PlayerState;
pub use keys::{RedisKey, KeyPart};
