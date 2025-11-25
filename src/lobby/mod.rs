// Lobby domain module: managers and domain logic for lobbies
pub mod error;
pub mod messages;
pub mod websocket;
pub mod manager;

pub use error::LobbyError;
pub use messages::{LobbyClientMessage, LobbyServerMessage};
pub use manager::*;
