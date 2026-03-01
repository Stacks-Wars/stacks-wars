// Ludo Rush Game Module
//
// A fast-paced Ludo variant where capturing an opponent sends their pawn
// home and instantly finishes your attacking pawn. Only the 4 entry points
// are safe squares, making the board much more aggressive.

pub mod board;
pub mod engine;
pub mod message;

pub use engine::create_ludo_rush;
