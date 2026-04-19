//! Stacks Wars SDK: traits and portable types for game crates.

mod dto;
mod engine;
mod error;
mod game_error;
mod host;
pub mod kit;
mod wire;

pub use dto::{ClaimState, JoinRequestState, PlayerStateWire, PlayerStatus};
pub use engine::{GameAction, GameEngine, GameEvent, GameFactory};
pub use error::CoreError;
pub use game_error::GameError;
pub use host::{GameHost, GameHostRef};
pub use kit::{
    GameBootstrap, GamePlayerState, GameResults, GameStatus, GameSummary, PlayerRanking,
    PlayerResult, TurnRotation, WarsPointContext, calculate_wars_point,
};
pub use wire::{GameRoomBroadcast, UserRoomMessage};
