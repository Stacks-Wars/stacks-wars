// Game registry - central place for game contributors to register their games
use crate::games::{
    //lexi_wars::create_lexi_wars
    GameFactory,
    coin_flip::create_coin_flip,
};
use std::collections::HashMap;
use uuid::Uuid;

// Game IDs - randomly generated UUIDs
//pub const LEXI_WARS_GAME_ID: Uuid = uuid::uuid!("7c9e6679-7425-40de-944b-e6e9a5c5f0a4");
pub const COIN_FLIP_GAME_ID: Uuid = uuid::uuid!("05f920e9-6b71-471e-a98a-2e5fe9402c00");

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
    //registry.insert(LEXI_WARS_GAME_ID, create_lexi_wars as GameFactory);
    registry.insert(COIN_FLIP_GAME_ID, create_coin_flip as GameFactory);

    // Future games can be added here:
    // registry.insert(YOUR_GAME_ID, create_your_game as GameFactory);

    registry
}
