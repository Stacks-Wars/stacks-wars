// Game registry — wires published / path-resolved game crates into the server.
use stacks_wars_core::GameFactory;
use stacks_wars_checkers::{self as checkers};
use stacks_wars_lexi_wars::{self as lexi_wars};
use stacks_wars_ludo::{self as ludo};
use stacks_wars_ludo_rush::{self as ludo_rush};
use std::collections::HashMap;
use uuid::Uuid;

pub use checkers::CHECKERS_GAME_ID;
pub use lexi_wars::LEXI_WARS_GAME_ID;
pub use ludo::LUDO_GAME_ID;
pub use ludo_rush::LUDO_RUSH_GAME_ID;

pub fn create_game_registry() -> HashMap<Uuid, GameFactory> {
    let mut registry = HashMap::new();

    registry.insert(CHECKERS_GAME_ID, checkers::create_checkers as GameFactory);
    registry.insert(LEXI_WARS_GAME_ID, lexi_wars::create_lexi_wars as GameFactory);
    registry.insert(LUDO_GAME_ID, ludo::create_ludo as GameFactory);
    registry.insert(LUDO_RUSH_GAME_ID, ludo_rush::create_ludo_rush as GameFactory);

    registry
}
