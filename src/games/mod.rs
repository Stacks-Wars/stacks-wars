// Game engine infrastructure
use crate::errors::AppError;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use uuid::Uuid;

/// Base trait for all game actions (client -> server messages)
/// Each game defines its own action enum that implements this trait
pub trait GameAction: DeserializeOwned + Send + Sync + 'static {}

/// Base trait for all game events (server -> client messages)
/// Each game defines its own event enum that implements this trait
pub trait GameEvent: Serialize + Send + Sync + 'static {}

/// Core game engine trait that all games must implement
///
/// Actions and events are passed as JSON Value to avoid trait object issues
pub trait GameEngine: Send + Sync {
    /// Handle a player action (as JSON) and return events to broadcast (as JSON)
    fn handle_action(&mut self, user_id: Uuid, action: Value) -> Result<Vec<Value>, AppError>;

    /// Initialize game with player list, return initial events (as JSON)
    fn initialize(&mut self, player_ids: Vec<Uuid>) -> Result<Vec<Value>, AppError>;

    /// Game tick for time-based events (called periodically), returns events (as JSON)
    fn tick(&mut self) -> Result<Vec<Value>, AppError>;

    /// Check if game is finished
    fn is_finished(&self) -> bool;
}

/// Type of factory function that creates game engine instances
pub type GameFactory = fn() -> Box<dyn GameEngine>;
