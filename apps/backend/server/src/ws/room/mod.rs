// Room WebSocket module - handles lobby room connections (game + chat)
pub mod engine;
pub mod error;
pub mod handler;
pub mod messages;

pub use engine::handle_room_message;
pub use error::RoomError;
pub use handler::room_handler;
pub use messages::{RoomClientMessage, RoomServerMessage};
