// Game engine infrastructure
use crate::errors::AppError;
use crate::state::AppState;
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use uuid::Uuid;

pub mod common;
pub mod error;
pub mod lexi_wars;
pub mod ludo;
pub mod ludo_rush;
pub mod registry;

pub use common::*;
pub use error::GameError;
pub use registry::{LEXI_WARS_GAME_ID, LUDO_GAME_ID, LUDO_RUSH_GAME_ID, create_game_registry};

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

    /// Set lobby context for prize/points calculation (entry amount, token info, etc.)
    /// Should be called after creation and before initialize()
    async fn set_lobby_context(
        &mut self,
        _game_id: Uuid,
        _entry_amount: Option<f64>,
        _current_amount: Option<f64>,
        _is_sponsored: bool,
        _creator_id: Uuid,
        _token_symbol: Option<String>,
        _token_contract_id: Option<String>,
    ) {
        // Default: no-op - override if game needs lobby context
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

    /// Get game state for a specific user reconnecting mid-game
    /// This returns game-specific state that the client needs to restore the UI
    /// The user_id is optional - if provided, games can include user-specific info (e.g., current rule if it's their turn)
    /// Spectators (unauthenticated users) will receive the generic state without user-specific info
    async fn get_game_state(&self, user_id: Option<Uuid>) -> Result<Value, AppError>;

    /// Handle a player quitting the game mid-game
    /// Each game implements its own quit logic (elimination, forfeit, etc.)
    /// Returns events to broadcast (as JSON)
    async fn handle_player_quit(&mut self, _user_id: Uuid) -> Result<Vec<Value>, AppError> {
        // Default: no-op
        Ok(vec![])
    }

    /// Check if game is finished
    fn is_finished(&self) -> bool;
}

/// Type of factory function that creates game engine instances
pub type GameFactory = fn(Uuid, AppState) -> Box<dyn GameEngine>;
