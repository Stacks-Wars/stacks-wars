// Game registry - central place for game contributors to register their games
use crate::games::{GameFactory, lexi_wars::create_lexi_wars};
use std::collections::HashMap;
use uuid::Uuid;

// Game IDs - randomly generated UUIDs
pub const LEXI_WARS_GAME_ID: Uuid = uuid::uuid!("97f19daa-b6b4-455b-a21e-f225884767d5");

/// Initialize and return the game registry with all registered games
///
/// Game contributors should add their games here by:
/// 1. Defining a constant UUID for their game
/// 2. Inserting their factory function into the registry
///
/// This keeps game registration centralized and makes it easy to add new games
/// without touching AppState or other core infrastructure.
pub fn create_game_registry() -> HashMap<Uuid, GameFactory> {
    let mut registry = HashMap::new();

    // Register games
    registry.insert(LEXI_WARS_GAME_ID, create_lexi_wars as GameFactory);

    // Future games can be added here:
    // registry.insert(YOUR_GAME_ID, create_your_game as GameFactory);

    registry
}
