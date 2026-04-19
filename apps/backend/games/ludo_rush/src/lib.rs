//! Ludo Rush variant game crate.

pub mod board;
pub mod engine;
pub mod message;

pub use engine::create_ludo_rush;

pub const LUDO_RUSH_GAME_ID: uuid::Uuid = uuid::Uuid::from_bytes([
    118, 191, 136, 195, 124, 235, 65, 49, 141, 45, 106, 103, 157, 112, 248, 43,
]);
