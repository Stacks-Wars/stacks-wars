// Game registry — aggregates built-in game crates and in-tree engines.
use crate::games::{lexi_wars, ludo, ludo_rush};
use stacks_wars_core::GameFactory;
use stacks_wars_game_checkers::{self as checkers};
use std::collections::HashMap;
use uuid::Uuid;

pub use checkers::CHECKERS_GAME_ID;
pub use stacks_wars_game_lexi_wars::LEXI_WARS_GAME_ID;
pub use stacks_wars_game_ludo::LUDO_GAME_ID;
pub use stacks_wars_game_ludo_rush::LUDO_RUSH_GAME_ID;

pub fn create_game_registry() -> HashMap<Uuid, GameFactory> {
    let mut registry = HashMap::new();

    registry.insert(CHECKERS_GAME_ID, checkers::create_checkers as GameFactory);
    registry.insert(LEXI_WARS_GAME_ID, lexi_wars::create_lexi_wars as GameFactory);
    registry.insert(LUDO_GAME_ID, ludo::create_ludo as GameFactory);
    registry.insert(LUDO_RUSH_GAME_ID, ludo_rush::create_ludo_rush as GameFactory);

    registry
}
