// Lobby WebSocket module - handles lobby list browsing
pub mod error;
pub mod handler;
pub mod messages;

pub use error::LobbyError;
pub use handler::lobby_handler;
pub use messages::{LobbyClientMessage, LobbyServerMessage};
