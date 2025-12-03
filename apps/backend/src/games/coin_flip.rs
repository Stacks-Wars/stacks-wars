// Coin Flip Game Implementation
//
// Game Rules:
// - Server randomly chooses heads or tails
// - Each player has 5 seconds to guess
// - Correct guess: Move to next player
// - Wrong guess: Player eliminated, move to next player
// - Last player standing wins

use crate::{
    errors::AppError,
    games::{GameAction, GameEngine, GameError, GameEvent, common::*},
};
use async_trait::async_trait;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

const TURN_TIMEOUT_SECS: u64 = 5;

/// Coin side enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CoinSide {
    Heads,
    Tails,
}

impl CoinSide {
    pub fn random() -> Self {
        if rand::rng().random_bool(0.5) {
            CoinSide::Heads
        } else {
            CoinSide::Tails
        }
    }
}

/// Client actions (client -> server)
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CoinFlipAction {
    MakeGuess { guess: CoinSide },
}

impl GameAction for CoinFlipAction {}

/// Server events (server -> client)
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CoinFlipEvent {
    /// Game started with bootstrap data
    GameStarted {
        players: Vec<Uuid>,
        current_player: Uuid,
        timeout_secs: u64,
    },

    /// New round started
    RoundStarted {
        round: usize,
        current_player: Uuid,
        timeout_secs: u64,
    },

    /// Player submitted their guess (not revealed yet)
    GuessReceived { player_id: Uuid },

    /// Player was eliminated due to timeout
    PlayerTimedOut { player_id: Uuid },

    /// Round completed - reveal results
    RoundComplete {
        round: usize,
        coin_result: CoinSide,
        results: Vec<RoundPlayerResult>,
        eliminated_players: Vec<Uuid>,
        remaining_players: Vec<Uuid>,
    },

    /// Game finished with final rankings
    GameFinished { results: GameResults },
}

impl GameEvent for CoinFlipEvent {}

/// Result for a single player in a round
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoundPlayerResult {
    pub player_id: Uuid,
    pub guess: Option<CoinSide>,
    pub correct: bool,
    pub eliminated: bool,
}

/// Coin Flip game state
#[derive(Debug, Clone)]
pub struct CoinFlipEngine {
    #[allow(dead_code)]
    lobby_id: Uuid,
    players: HashMap<Uuid, GamePlayerState>,
    turn_rotation: TurnRotation,
    current_round: usize,
    current_round_guesses: HashMap<Uuid, CoinSide>,
    turn_started_at: Option<i64>,
    finished: bool,
    results: Option<GameResults>,
}

impl CoinFlipEngine {
    pub fn new(lobby_id: Uuid) -> Self {
        Self {
            lobby_id,
            players: HashMap::new(),
            turn_rotation: TurnRotation::new(Vec::new()),
            current_round: 0,
            current_round_guesses: HashMap::new(),
            turn_started_at: None,
            finished: false,
            results: None,
        }
    }

    fn start_new_round(&mut self) -> Option<CoinFlipEvent> {
        if let Some(player_id) = self.turn_rotation.current_player() {
            self.current_round += 1;
            self.current_round_guesses.clear();
            self.turn_started_at = Some(chrono::Utc::now().timestamp());

            Some(CoinFlipEvent::RoundStarted {
                round: self.current_round,
                current_player: player_id,
                timeout_secs: TURN_TIMEOUT_SECS,
            })
        } else {
            None
        }
    }

    /// Check if all active players have submitted guesses for this round
    fn is_round_complete(&self) -> bool {
        let active_players = self.turn_rotation.active_players();
        active_players
            .iter()
            .all(|p| self.current_round_guesses.contains_key(p))
    }

    /// Process the round - evaluate all guesses and determine eliminations
    fn process_round(&mut self) -> Result<Vec<CoinFlipEvent>, GameError> {
        let mut events = Vec::new();

        // Generate the coin flip result
        let coin_result = CoinSide::random();

        let active_players = self.turn_rotation.active_players();
        let mut results = Vec::new();
        let mut players_to_eliminate = Vec::new();

        // Evaluate each player's guess
        for player_id in &active_players {
            let guess = self.current_round_guesses.get(player_id);
            let correct = guess.map(|g| *g == coin_result).unwrap_or(false);

            results.push(RoundPlayerResult {
                player_id: *player_id,
                guess: guess.copied(),
                correct,
                eliminated: !correct,
            });

            if !correct {
                players_to_eliminate.push(*player_id);
            }
        }

        // Special case: if only 2 players remain and both get it wrong or both get it right
        // Don't eliminate anyone, continue the game
        if active_players.len() == 2 {
            let correct_count = results.iter().filter(|r| r.correct).count();
            if correct_count == 0 || correct_count == 2 {
                // Both failed or both succeeded - no elimination
                players_to_eliminate.clear();
                for result in &mut results {
                    result.eliminated = false;
                }
            }
        }

        // Eliminate players
        for player_id in &players_to_eliminate {
            self.turn_rotation.eliminate_player(*player_id);
            if let Some(player_state) = self.players.get_mut(player_id) {
                player_state.eliminate();
            }
        }

        let remaining_players = self.turn_rotation.active_players();

        events.push(CoinFlipEvent::RoundComplete {
            round: self.current_round,
            coin_result,
            results,
            eliminated_players: players_to_eliminate.clone(),
            remaining_players: remaining_players.clone(),
        });

        // Check if game is over
        if self.turn_rotation.is_game_over() {
            self.finished = true;
            let player_states: Vec<GamePlayerState> = self.players.values().cloned().collect();
            self.results = Some(GameResults::from_game_states(player_states));

            events.push(CoinFlipEvent::GameFinished {
                results: self.results.clone().unwrap(),
            });
        } else {
            // Start next round
            self.turn_rotation.next_turn();
            if let Some(next_round_event) = self.start_new_round() {
                events.push(next_round_event);
            }
        }

        Ok(events)
    }

    fn handle_guess(
        &mut self,
        user_id: Uuid,
        guess: CoinSide,
    ) -> Result<Vec<CoinFlipEvent>, GameError> {
        let mut events = Vec::new();

        // Verify player is in the game
        if !self.players.contains_key(&user_id) {
            return Err(GameError::NotInGame);
        }

        // Verify player hasn't been eliminated
        if !self.turn_rotation.active_players().contains(&user_id) {
            return Err(GameError::AlreadyEliminated);
        }

        // Verify player hasn't already guessed this round
        if self.current_round_guesses.contains_key(&user_id) {
            return Err(GameError::InvalidAction(
                "Already guessed this round".to_string(),
            ));
        }

        // Store the guess
        self.current_round_guesses.insert(user_id, guess);

        // Send received confirmation
        events.push(CoinFlipEvent::GuessReceived { player_id: user_id });

        // Check if round is complete
        if self.is_round_complete() {
            events.extend(self.process_round()?);
        }

        Ok(events)
    }
}

#[async_trait]
impl GameEngine for CoinFlipEngine {
    async fn initialize(&mut self, player_ids: Vec<Uuid>) -> Result<Vec<Value>, AppError> {
        tracing::info!("Initializing CoinFlip with {} players", player_ids.len());

        self.players = player_ids
            .iter()
            .map(|&id| (id, GamePlayerState::new(id)))
            .collect();
        self.turn_rotation = TurnRotation::new(player_ids.clone());

        let mut events = Vec::new();

        // Start first round
        if let Some(current_player) = self.turn_rotation.current_player() {
            self.current_round = 1;
            self.turn_started_at = Some(chrono::Utc::now().timestamp());

            events.push(
                serde_json::to_value(CoinFlipEvent::GameStarted {
                    players: player_ids,
                    current_player,
                    timeout_secs: TURN_TIMEOUT_SECS,
                })
                .map_err(|e| AppError::Serialization(e.to_string()))?,
            );
        }

        Ok(events)
    }

    async fn handle_action(
        &mut self,
        user_id: Uuid,
        action: Value,
    ) -> Result<Vec<Value>, AppError> {
        let action: CoinFlipAction = serde_json::from_value(action)
            .map_err(|e| AppError::BadRequest(format!("Invalid action: {}", e)))?;

        tracing::debug!("CoinFlip action from {}: {:?}", user_id, action);

        let game_events = match action {
            CoinFlipAction::MakeGuess { guess } => self.handle_guess(user_id, guess)?,
        };

        // Convert to JSON
        game_events
            .into_iter()
            .map(|e| serde_json::to_value(e).map_err(|e| AppError::Serialization(e.to_string())))
            .collect()
    }

    async fn get_bootstrap(&self) -> Result<Value, AppError> {
        let bootstrap = GameBootstrap {
            game_id: Uuid::nil(), // Will be set by caller
            status: if self.finished {
                GameStatus::Finished
            } else {
                GameStatus::InProgress
            },
            current_state: serde_json::json!({
                "currentPlayer": self.turn_rotation.current_player(),
                "activePlayers": self.turn_rotation.active_players(),
                "currentRound": self.current_round,
                "timeoutSecs": TURN_TIMEOUT_SECS,
                "turnStartedAt": self.turn_started_at,
            }),
            players: self.players.keys().copied().collect(),
            started_at: chrono::Utc::now().timestamp(),
            finished_at: if self.finished {
                Some(chrono::Utc::now().timestamp())
            } else {
                None
            },
        };

        serde_json::to_value(bootstrap).map_err(|e| AppError::Serialization(e.to_string()))
    }

    async fn get_results(&self) -> Result<Option<GameResults>, AppError> {
        Ok(self.results.clone())
    }

    async fn tick(&mut self) -> Result<Vec<Value>, AppError> {
        // Check for turn timeout (eliminate current player if they haven't guessed)
        if let Some(started_at) = self.turn_started_at {
            let now = chrono::Utc::now().timestamp();
            if now - started_at > TURN_TIMEOUT_SECS as i64 {
                if let Some(current_player) = self.turn_rotation.current_player() {
                    // Check if this player hasn't guessed yet
                    if !self.current_round_guesses.contains_key(&current_player) {
                        tracing::info!(
                            "Player {} timed out in round {}",
                            current_player,
                            self.current_round
                        );

                        let mut events = vec![CoinFlipEvent::PlayerTimedOut {
                            player_id: current_player,
                        }];

                        // Mark player as having no guess (will be counted as wrong in process_round)
                        // Don't add to current_round_guesses - they get eliminated

                        // If this timeout completes the round, process it
                        let active_players = self.turn_rotation.active_players();
                        let guessed_players: Vec<_> = active_players
                            .iter()
                            .filter(|p| {
                                self.current_round_guesses.contains_key(p) || **p == current_player
                            })
                            .collect();

                        if guessed_players.len() == active_players.len() {
                            // Everyone has either guessed or timed out - process round
                            events.extend(self.process_round().map_err(|e| e)?);
                        }

                        return events
                            .into_iter()
                            .map(|e| {
                                serde_json::to_value(e)
                                    .map_err(|e| AppError::Serialization(e.to_string()))
                            })
                            .collect();
                    }
                }
            }
        }
        Ok(Vec::new())
    }

    fn is_finished(&self) -> bool {
        self.finished
    }
}

/// Factory function
pub fn create_coin_flip(lobby_id: Uuid) -> Box<dyn GameEngine> {
    Box::new(CoinFlipEngine::new(lobby_id))
}
