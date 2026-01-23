use crate::{
    errors::AppError,
    games::{GameAction, GameEngine, GameEvent, GameResults},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// LexiWars game actions (client -> server)
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LexiWarsAction {
    SubmitWord { word: String },
    Ready,
}

impl GameAction for LexiWarsAction {}

/// LexiWars game events (server -> client)
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LexiWarsEvent {
    GameInitialized { players: Vec<Uuid> },
    WordSubmitted { user_id: Uuid, word: String },
    RoundComplete { winner: Option<Uuid> },
    GameOver { winner: Uuid },
}

impl GameEvent for LexiWarsEvent {}

/// LexiWars game engine implementation
pub struct LexiWarsEngine {
    #[allow(dead_code)]
    lobby_id: Uuid,
    players: Vec<Uuid>,
    finished: bool,
}

impl LexiWarsEngine {
    pub fn new(lobby_id: Uuid) -> Self {
        Self {
            lobby_id,
            players: Vec::new(),
            finished: false,
        }
    }
}

#[async_trait]
impl GameEngine for LexiWarsEngine {
    async fn handle_action(
        &mut self,
        user_id: Uuid,
        action: Value,
    ) -> Result<Vec<Value>, AppError> {
        // Try to parse as LexiWarsAction
        let action: LexiWarsAction = serde_json::from_value(action)
            .map_err(|e| AppError::BadRequest(format!("Invalid LexiWars action: {}", e)))?;

        tracing::debug!("LexiWars action from {}: {:?}", user_id, action);

        // Placeholder implementation - just echo back an event
        match action {
            LexiWarsAction::SubmitWord { word } => {
                let event = LexiWarsEvent::WordSubmitted {
                    user_id,
                    word,
                };
                let json_event = serde_json::to_value(event).map_err(|e| {
                    AppError::Serialization(format!("Failed to serialize event: {}", e))
                })?;
                Ok(vec![json_event])
            }
            LexiWarsAction::Ready => {
                // Placeholder - just acknowledge
                Ok(vec![])
            }
        }
    }

    async fn initialize(&mut self, player_ids: Vec<Uuid>) -> Result<Vec<Value>, AppError> {
        tracing::info!("Initializing LexiWars with {} players", player_ids.len());
        self.players = player_ids.clone();

        let event = LexiWarsEvent::GameInitialized {
            players: player_ids,
        };

        let json_event = serde_json::to_value(event)
            .map_err(|e| AppError::Serialization(format!("Failed to serialize event: {}", e)))?;

        Ok(vec![json_event])
    }

    async fn tick(&mut self) -> Result<Vec<Value>, AppError> {
        // Placeholder - no time-based events yet
        Ok(vec![])
    }

    async fn get_bootstrap(&self) -> Result<Value, AppError> {
        // Placeholder bootstrap
        Ok(serde_json::json!({
            "players": self.players,
            "status": if self.finished { "finished" } else { "in_progress" }
        }))
    }

    async fn get_results(&self) -> Result<Option<GameResults>, AppError> {
        // Placeholder - no results yet
        Ok(None)
    }

    fn is_finished(&self) -> bool {
        self.finished
    }
}

/// Factory function to create new LexiWars game instances
pub fn create_lexi_wars(lobby_id: Uuid) -> Box<dyn GameEngine> {
    Box::new(LexiWarsEngine::new(lobby_id))
}
