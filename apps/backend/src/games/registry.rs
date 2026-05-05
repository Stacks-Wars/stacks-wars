// Game registry - central place for game contributors to register their games
use crate::games::{
    GameFactory, checkers::create_checkers, lexi_wars::create_lexi_wars, ludo::create_ludo,
    ludo_rush::create_ludo_rush,
};
use std::collections::HashMap;
use uuid::Uuid;

// Game IDs - randomly generated UUIDs
pub const LEXI_WARS_GAME_ID: Uuid = uuid::uuid!("5eb61ff4-8f9f-48bd-ae61-dfd6d95052eb");
pub const LUDO_GAME_ID: Uuid = uuid::uuid!("d04edafc-c9f7-42cc-bfae-68fd93422e43");
pub const LUDO_RUSH_GAME_ID: Uuid = uuid::uuid!("76bf88c3-7ceb-4131-8d2d-6a679d70f82b");
pub const CHECKERS_GAME_ID: Uuid = uuid::uuid!("4d94aca3-0ba2-4965-8fa2-4628e5e3ee27");

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
    registry.insert(LUDO_GAME_ID, create_ludo as GameFactory);
    registry.insert(LUDO_RUSH_GAME_ID, create_ludo_rush as GameFactory);
    registry.insert(CHECKERS_GAME_ID, create_checkers as GameFactory);

    // Future games can be added here:
    // registry.insert(YOUR_GAME_ID, create_your_game as GameFactory);

    registry
}
