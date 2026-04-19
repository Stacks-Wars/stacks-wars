//! Classic Ludo game crate.

pub mod board;
pub mod engine;
pub mod message;

pub use engine::create_ludo;

pub const LUDO_GAME_ID: uuid::Uuid = uuid::Uuid::from_bytes([
    208, 78, 218, 252, 201, 247, 66, 204, 191, 174, 104, 253, 147, 66, 46, 67,
]);
