//! Checkers game crate.

pub mod engine;
pub mod message;

pub use engine::create_checkers;
pub use message::{
    CheckersAction, CheckersBoard, CheckersEvent, CheckersMove, Piece, PieceColor, PieceKind,
    Position,
};

pub const CHECKERS_GAME_ID: uuid::Uuid = uuid::Uuid::from_bytes([
    0x4d, 0x94, 0xac, 0xa3, 0x0b, 0xa2, 0x49, 0x65, 0x8f, 0xa2, 0x46, 0x28, 0xe5, 0xe3, 0xee, 0x27,
]);
