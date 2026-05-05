// Game engine infrastructure and platform helpers.

pub mod common;
pub mod error;

/// Shipped Lexi Wars implementation (standalone crate).
pub mod lexi_wars {
    pub use stacks_wars_lexi_wars::*;
}

/// Shipped Ludo implementation (standalone crate).
pub mod ludo {
    pub use stacks_wars_ludo::*;
}

/// Shipped Ludo Rush implementation (standalone crate).
pub mod ludo_rush {
    pub use stacks_wars_ludo_rush::*;
}

pub mod registry;

pub use common::*;
pub use error::GameError;
pub use registry::{
    create_game_registry, CHECKERS_GAME_ID, LEXI_WARS_GAME_ID, LUDO_GAME_ID, LUDO_RUSH_GAME_ID,
};
pub use stacks_wars_core::{
    GameAction, GameEngine, GameEvent, GameFactory, GameHost, GameHostRef,
};
