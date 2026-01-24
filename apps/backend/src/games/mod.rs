// Game engine infrastructure
use crate::errors::AppError;
use crate::state::AppState;
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use uuid::Uuid;

pub mod coin_flip;
pub mod common;
pub mod error;
pub mod lexi_wars;
pub mod registry;

pub use common::*;
pub use error::GameError;
pub use registry::{COIN_FLIP_GAME_ID, LEXI_WARS_GAME_ID, create_game_registry};

/// Base trait for all game actions (client -> server messages)
/// Each game defines its own action enum that implements this trait
pub trait GameAction: DeserializeOwned + Send + Sync + 'static {}

/// Base trait for all game events (server -> client messages)
/// Each game defines its own event enum that implements this trait
pub trait GameEvent: Serialize + Send + Sync + 'static {}

/// Core game engine trait that all games must implement
///
/// Actions and events are passed as JSON Value to avoid trait object issues
#[async_trait]
pub trait GameEngine: Send + Sync {
    /// Set the app state for broadcasting and database access
    /// Should be called before initialize()
    async fn set_state(&mut self, _state: AppState) {
        // Default: no-op - override if game needs app state
    }

    /// Handle a player action (as JSON) and return events to broadcast (as JSON)
    async fn handle_action(&mut self, user_id: Uuid, action: Value)
    -> Result<Vec<Value>, AppError>;

    /// Initialize game with player list, return initial events (as JSON)
    async fn initialize(&mut self, player_ids: Vec<Uuid>) -> Result<Vec<Value>, AppError>;

    /// Start the game loop (for games with background tasks like turn timers)
    /// Called after initialize() and after engine is stored in active_games
    /// Default implementation does nothing - override for games with loops
    fn start_loop(&mut self, _state: AppState) {
        // Default: no-op for games without background loops
    }

    /// Get game bootstrap state (for players joining mid-game or reconnecting)
    /// Returns JSON representation of current game state
    async fn get_bootstrap(&self) -> Result<Value, AppError>;

    /// Get game state for a specific user reconnecting mid-game
    /// This returns game-specific state that the client needs to restore the UI
    /// The user_id is optional - if provided, games can include user-specific info (e.g., current rule if it's their turn)
    /// Spectators (unauthenticated users) will receive the generic state without user-specific info
    async fn get_game_state(&self, _user_id: Option<Uuid>) -> Result<Value, AppError> {
        // Default: return the generic bootstrap
        self.get_bootstrap().await
    }

    /// Get final results if game is finished
    async fn get_results(&self) -> Result<Option<GameResults>, AppError>;

    /// Game tick for time-based events (called periodically), returns events (as JSON)
    async fn tick(&mut self) -> Result<Vec<Value>, AppError>;

    /// Check if game is finished
    fn is_finished(&self) -> bool;
}

/// Type of factory function that creates game engine instances
pub type GameFactory = fn(Uuid) -> Box<dyn GameEngine>;
