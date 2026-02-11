// Ludo Game Engine
//
// Core game logic including:
// - LudoEngine struct and implementation
// - GameEngine trait implementation
// - Game loop (turn timeouts)
// - Prize/points calculation

use crate::{
    db::player_state::PlayerStateRepository,
    errors::AppError,
    games::{GameEngine, GameError, GameResults, common::*},
    models::PlayerState,
    state::AppState,
    ws::{broadcast, room::messages::RoomServerMessage},
};
use async_trait::async_trait;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Notify, RwLock};
use uuid::Uuid;

use super::board::{roll_dice, LudoBoard, PawnPosition, PAWNS_PER_PLAYER};
use super::message::{LudoAction, LudoEvent};

// ============================================================================
// Constants
// ============================================================================

pub const TURN_TIMEOUT_SECS: u64 = 30;
pub const ROLL_TIMEOUT_SECS: u64 = 15;

// ============================================================================
// Turn Phase
// ============================================================================

/// Represents the current phase of a player's turn
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnPhase {
    /// Waiting for player to roll dice
    WaitingForRoll,
    /// Waiting for player to choose a pawn to move
    WaitingForMove,
    /// Turn is complete
    Complete,
}

// ============================================================================
// Inner State (shared via Arc<RwLock>)
// ============================================================================

/// Inner state that is shared between the GameEngine methods and the game loop
struct LudoInner {
    lobby_id: Uuid,
    players: HashMap<Uuid, GamePlayerState>,
    player_states: HashMap<Uuid, PlayerState>,
    turn_rotation: TurnRotation,
    board: LudoBoard,
    total_players: usize,
    finished: bool,
    results: Option<GameResults>,

    // Current turn state
    turn_phase: TurnPhase,
    current_dice: Option<u8>,
    movable_pawns: Vec<usize>,
    consecutive_sixes: usize,

    // Prize/points calculation context
    entry_amount: Option<f64>,
    current_amount: Option<f64>,
    is_sponsored: bool,
    creator_id: Option<Uuid>,
    token_symbol: Option<String>,
    token_contract_id: Option<String>,

    // Game loop control
    turn_advance_notify: Arc<Notify>,

    state: AppState,
}

impl LudoInner {
    fn new(lobby_id: Uuid, state: AppState) -> Self {
        Self {
            lobby_id,
            players: HashMap::new(),
            player_states: HashMap::new(),
            turn_rotation: TurnRotation::new(Vec::new()),
            board: LudoBoard::new(&[]),
            total_players: 0,
            finished: false,
            results: None,
            turn_phase: TurnPhase::WaitingForRoll,
            current_dice: None,
            movable_pawns: Vec::new(),
            consecutive_sixes: 0,
            entry_amount: None,
            current_amount: None,
            is_sponsored: false,
            creator_id: None,
            token_symbol: None,
            token_contract_id: None,
            turn_advance_notify: Arc::new(Notify::new()),
            state,
        }
    }
}

// ============================================================================
// Game Engine (wraps inner state in Arc<RwLock>)
// ============================================================================

/// Ludo game engine - wraps shared state for thread-safe access
pub struct LudoEngine {
    inner: Arc<RwLock<LudoInner>>,
}

impl LudoEngine {
    pub fn new(lobby_id: Uuid, state: AppState) -> Self {
        Self {
            inner: Arc::new(RwLock::new(LudoInner::new(lobby_id, state))),
        }
    }

    /// Get the Arc<RwLock<Inner>> for spawning game loop
    fn get_inner(&self) -> Arc<RwLock<LudoInner>> {
        self.inner.clone()
    }

    /// Set lobby context for prize/points calculation
    pub async fn set_lobby_context(
        &self,
        entry_amount: Option<f64>,
        current_amount: Option<f64>,
        is_sponsored: bool,
        creator_id: Uuid,
        token_symbol: Option<String>,
        token_contract_id: Option<String>,
    ) {
        let mut inner = self.inner.write().await;
        inner.entry_amount = entry_amount;
        inner.current_amount = current_amount;
        inner.is_sponsored = is_sponsored;
        inner.creator_id = Some(creator_id);
        inner.token_symbol = token_symbol;
        inner.token_contract_id = token_contract_id;
    }
}

// ============================================================================
// Inner State Methods
// ============================================================================

impl LudoInner {
    /// Get a player's PlayerState by ID
    fn get_player_state(&self, user_id: Uuid) -> Option<PlayerState> {
        self.player_states.get(&user_id).cloned()
    }

    /// Get the current player's PlayerState
    fn get_current_player_state(&self) -> Option<PlayerState> {
        self.turn_rotation
            .current_player()
            .and_then(|id| self.player_states.get(&id).cloned())
    }

    /// Calculate prize for a given rank
    fn calculate_prize(&self, rank: usize, participants: usize) -> Option<f64> {
        let total_pool = self.current_amount?;

        if total_pool <= 0.0 {
            return None;
        }

        let prize = match rank {
            1 => {
                if participants == 2 {
                    (total_pool * 70.0) / 100.0
                } else {
                    (total_pool * 50.0) / 100.0
                }
            }
            2 => (total_pool * 30.0) / 100.0,
            3 => (total_pool * 20.0) / 100.0,
            _ => 0.0,
        };

        if prize > 0.0 {
            Some(prize)
        } else {
            None
        }
    }

    /// Build WarsPointContext for a player result
    fn build_wars_point_context(
        &self,
        user_id: Uuid,
        rank: usize,
        prize: Option<f64>,
    ) -> WarsPointContext {
        WarsPointContext {
            user_id,
            rank,
            prize,
            participants: self.total_players,
            entry_amount: self.entry_amount,
            current_amount: self.current_amount,
            is_sponsored: self.is_sponsored,
            creator_id: self.creator_id,
            active_players: self.turn_rotation.active_count(),
            token_symbol: self.token_symbol.clone(),
            token_contract_id: self.token_contract_id.clone(),
        }
    }

    /// Start the turn for the current player
    async fn start_turn(&mut self) {
        self.turn_phase = TurnPhase::WaitingForRoll;
        self.current_dice = None;
        self.movable_pawns.clear();

        let Some(current_player_id) = self.turn_rotation.current_player() else {
            return;
        };

        let Some(current_player_state) = self.get_player_state(current_player_id) else {
            return;
        };

        // Broadcast Turn event to room
        let turn_event = LudoEvent::Turn {
            player: current_player_state,
            timeout_secs: TURN_TIMEOUT_SECS,
        };
        broadcast::broadcast_game_message(
            &self.state,
            self.lobby_id,
            serde_json::to_value(&turn_event).unwrap_or_default(),
        )
        .await;
    }

    /// Handle dice roll action
    async fn handle_roll_dice(&mut self, user_id: Uuid) -> Result<bool, GameError> {
        // Verify it's this player's turn
        if self.turn_rotation.current_player() != Some(user_id) {
            return Err(GameError::NotYourTurn);
        }

        // Verify we're in the roll phase
        if self.turn_phase != TurnPhase::WaitingForRoll {
            let event = LudoEvent::Invalid {
                reason: "You've already rolled the dice".to_string(),
            };
            broadcast::broadcast_game_message_to_user(
                &self.state,
                user_id,
                serde_json::to_value(&event).unwrap_or_default(),
            )
            .await;
            return Ok(false);
        }

        // Roll the dice - guarantee a 6 if all pawns are at home so the game doesn't stall
        let player_board = self.board.get_player(user_id).unwrap();
        let dice = if player_board.all_pawns_at_home() {
            6
        } else {
            roll_dice()
        };
        self.current_dice = Some(dice);

        // Track consecutive sixes (3 sixes in a row = turn lost)
        if dice == 6 {
            self.consecutive_sixes += 1;
            if self.consecutive_sixes >= 3 {
                // Too many sixes, turn is lost
                let player_state = self.get_player_state(user_id).unwrap();
                let event = LudoEvent::NoValidMoves {
                    player: player_state,
                };
                broadcast::broadcast_game_message(
                    &self.state,
                    self.lobby_id,
                    serde_json::to_value(&event).unwrap_or_default(),
                )
                .await;

                self.consecutive_sixes = 0;
                self.turn_phase = TurnPhase::Complete;
                return Ok(true); // Turn advances
            }
        } else {
            self.consecutive_sixes = 0;
        }

        // Get movable pawns
        let player_board = self.board.get_player(user_id).unwrap();
        self.movable_pawns = player_board.get_movable_pawns(dice);

        let player_state = self.get_player_state(user_id).unwrap();

        // Broadcast dice roll
        let roll_event = LudoEvent::DiceRolled {
            player: player_state.clone(),
            dice,
            movable_pawns: self.movable_pawns.clone(),
        };
        broadcast::broadcast_game_message(
            &self.state,
            self.lobby_id,
            serde_json::to_value(&roll_event).unwrap_or_default(),
        )
        .await;

        // Check if player has any valid moves
        if self.movable_pawns.is_empty() {
            let event = LudoEvent::NoValidMoves {
                player: player_state,
            };
            broadcast::broadcast_game_message(
                &self.state,
                self.lobby_id,
                serde_json::to_value(&event).unwrap_or_default(),
            )
            .await;

            self.turn_phase = TurnPhase::Complete;
            return Ok(true); // Turn advances (no moves available)
        }

        // If only one pawn can move, auto-move it
        if self.movable_pawns.len() == 1 {
            return self.handle_move_pawn(user_id, self.movable_pawns[0]).await;
        }

        // Multiple pawns can move, wait for player choice
        self.turn_phase = TurnPhase::WaitingForMove;
        Ok(false)
    }

    /// Handle pawn move action
    async fn handle_move_pawn(&mut self, user_id: Uuid, pawn_id: usize) -> Result<bool, GameError> {
        // Verify it's this player's turn
        if self.turn_rotation.current_player() != Some(user_id) {
            return Err(GameError::NotYourTurn);
        }

        // Verify pawn_id is valid
        if pawn_id >= PAWNS_PER_PLAYER {
            let event = LudoEvent::Invalid {
                reason: "Invalid pawn ID".to_string(),
            };
            broadcast::broadcast_game_message_to_user(
                &self.state,
                user_id,
                serde_json::to_value(&event).unwrap_or_default(),
            )
            .await;
            return Ok(false);
        }

        // Verify the pawn can be moved
        if !self.movable_pawns.contains(&pawn_id) {
            let event = LudoEvent::Invalid {
                reason: "This pawn cannot be moved with the current dice roll".to_string(),
            };
            broadcast::broadcast_game_message_to_user(
                &self.state,
                user_id,
                serde_json::to_value(&event).unwrap_or_default(),
            )
            .await;
            return Ok(false);
        }

        let dice = self.current_dice.unwrap();
        let player_state = self.get_player_state(user_id).unwrap();

        // Get the pawn's current position before moving
        let from_position = self
            .board
            .get_player(user_id)
            .unwrap()
            .pawns[pawn_id]
            .position;

        // Move the pawn
        let to_position = self
            .board
            .get_player_mut(user_id)
            .unwrap()
            .move_pawn(pawn_id, dice);

        // Broadcast pawn moved
        let move_event = LudoEvent::PawnMoved {
            player: player_state.clone(),
            pawn_id,
            from: from_position,
            to: to_position,
        };
        broadcast::broadcast_game_message(
            &self.state,
            self.lobby_id,
            serde_json::to_value(&move_event).unwrap_or_default(),
        )
        .await;

        // Check for capture (only on main track)
        if let PawnPosition::OnTrack(pos) = to_position {
            if let Some((victim_id, victim_pawn_id)) = self.board.check_capture(user_id, pos) {
                // Send victim's pawn back home
                self.board
                    .get_player_mut(victim_id)
                    .unwrap()
                    .send_home(victim_pawn_id);

                let victim_state = self.get_player_state(victim_id).unwrap();

                let capture_event = LudoEvent::PawnCaptured {
                    attacker: player_state.clone(),
                    victim: victim_state,
                    pawn_id: victim_pawn_id,
                };
                broadcast::broadcast_game_message(
                    &self.state,
                    self.lobby_id,
                    serde_json::to_value(&capture_event).unwrap_or_default(),
                )
                .await;
            }
        }

        // Check if pawn finished
        if to_position == PawnPosition::Finished {
            let pawns_remaining = PAWNS_PER_PLAYER
                - self
                    .board
                    .get_player(user_id)
                    .unwrap()
                    .pawns_finished;

            let finish_event = LudoEvent::PawnFinished {
                player: player_state.clone(),
                pawn_id,
                pawns_remaining,
            };
            broadcast::broadcast_game_message(
                &self.state,
                self.lobby_id,
                serde_json::to_value(&finish_event).unwrap_or_default(),
            )
            .await;

            // Check for winner
            if self.board.get_player(user_id).unwrap().has_won() {
                self.turn_phase = TurnPhase::Complete;
                return Ok(true);
            }
        }

        // Broadcast board update
        let board_event = LudoEvent::BoardUpdate {
            board: self.board.clone(),
        };
        broadcast::broadcast_game_message(
            &self.state,
            self.lobby_id,
            serde_json::to_value(&board_event).unwrap_or_default(),
        )
        .await;

        // Check if player gets another turn (rolled a 6)
        if dice == 6 {
            let bonus_event = LudoEvent::BonusTurn {
                player: player_state,
            };
            broadcast::broadcast_game_message(
                &self.state,
                self.lobby_id,
                serde_json::to_value(&bonus_event).unwrap_or_default(),
            )
            .await;

            // Reset for another roll
            self.turn_phase = TurnPhase::WaitingForRoll;
            self.current_dice = None;
            self.movable_pawns.clear();
            return Ok(false); // Don't advance turn, player rolls again
        }

        self.turn_phase = TurnPhase::Complete;
        Ok(true) // Turn advances
    }

    /// End the game and calculate final standings
    async fn end_game(&mut self) {
        self.finished = true;

        // Get rankings from board state
        let rankings_data = self.board.get_rankings();
        let participants = self.total_players;
        let state = self.state.clone();
        let lobby_id = self.lobby_id;

        let mut final_standings: Vec<PlayerState> = Vec::new();
        let mut player_rankings: Vec<PlayerRanking> = Vec::new();

        for (rank_idx, (user_id, pawns_finished)) in rankings_data.iter().enumerate() {
            let rank = rank_idx + 1;
            let prize = self.calculate_prize(rank, participants);

            // Save player result
            let ctx = self.build_wars_point_context(*user_id, rank, prize);
            let wars_point = match save_player_result(&state, lobby_id, &ctx).await {
                Ok(result) => result.wars_point,
                Err(e) => {
                    tracing::error!("Failed to save player result: {}", e);
                    0.0
                }
            };

            // Update in-memory player_state
            if let Some(player_state) = self.player_states.get_mut(user_id) {
                player_state.rank = Some(rank);
                player_state.prize = prize;
                player_state.wars_point = Some(wars_point);
                final_standings.push(player_state.clone());
            }

            player_rankings.push(PlayerRanking {
                user_id: *user_id,
                rank,
                score: Some(*pawns_finished as i32),
                prize,
            });

            // Send GameOver to each player
            let game_over = RoomServerMessage::GameOver {
                rank,
                prize,
                wars_point,
            };
            broadcast::broadcast_user(&state, *user_id, &game_over).await;
        }

        // Broadcast FinalStanding to room
        let final_standing = RoomServerMessage::FinalStanding {
            standings: final_standings,
        };
        broadcast::broadcast_room(&state, lobby_id, &final_standing).await;

        // Mark lobby as finished
        if let Err(e) = finish_lobby(&state, lobby_id).await {
            tracing::error!("Failed to finish lobby {}: {}", lobby_id, e);
        }

        self.results = Some(GameResults {
            rankings: player_rankings,
            finished_at: chrono::Utc::now().timestamp(),
            metadata: None,
        });
    }
}

// ============================================================================
// GameEngine Trait Implementation
// ============================================================================

#[async_trait]
impl GameEngine for LudoEngine {
    async fn set_lobby_context(
        &mut self,
        entry_amount: Option<f64>,
        current_amount: Option<f64>,
        is_sponsored: bool,
        creator_id: Uuid,
        token_symbol: Option<String>,
        token_contract_id: Option<String>,
    ) {
        let mut inner = self.inner.write().await;
        inner.entry_amount = entry_amount;
        inner.current_amount = current_amount;
        inner.is_sponsored = is_sponsored;
        inner.creator_id = Some(creator_id);
        inner.token_symbol = token_symbol;
        inner.token_contract_id = token_contract_id;
    }

    async fn initialize(&mut self, player_ids: Vec<Uuid>) -> Result<Vec<Value>, AppError> {
        tracing::info!("Initializing Ludo with {} players", player_ids.len());

        if player_ids.len() < 2 {
            let error_event = RoomServerMessage::GameStartFailed {
                reason: "Ludo requires at least 2 players".to_string(),
            };
            return Ok(vec![serde_json::to_value(&error_event).unwrap_or_default()]);
        }

        if player_ids.len() > 4 {
            let error_event = RoomServerMessage::GameStartFailed {
                reason: "Ludo supports at most 4 players".to_string(),
            };
            return Ok(vec![serde_json::to_value(&error_event).unwrap_or_default()]);
        }

        let mut inner = self.inner.write().await;

        inner.total_players = player_ids.len();
        inner.players = player_ids
            .iter()
            .map(|&id| (id, GamePlayerState::new(id)))
            .collect();
        inner.turn_rotation = TurnRotation::new(player_ids.clone());
        inner.board = LudoBoard::new(&player_ids);

        // Load player states from Redis
        let player_repo = PlayerStateRepository::new(inner.state.redis.clone());
        if let Ok(states) = player_repo.get_all_in_lobby(inner.lobby_id).await {
            for state in states {
                inner.player_states.insert(state.user_id, state);
            }
        }

        // Send GameStarted event
        broadcast::broadcast_room(
            &inner.state,
            inner.lobby_id,
            &RoomServerMessage::GameStarted,
        )
        .await;

        // Send initial board state
        let board_event = LudoEvent::BoardUpdate {
            board: inner.board.clone(),
        };
        broadcast::broadcast_game_message(
            &inner.state,
            inner.lobby_id,
            serde_json::to_value(&board_event).unwrap_or_default(),
        )
        .await;

        Ok(Vec::new())
    }

    async fn handle_action(
        &mut self,
        user_id: Uuid,
        action: Value,
    ) -> Result<Vec<Value>, AppError> {
        let mut inner = self.inner.write().await;

        if inner.finished {
            return Err(AppError::BadRequest("Game is already finished".into()));
        }

        let action: LudoAction = serde_json::from_value(action)
            .map_err(|e| AppError::BadRequest(format!("Invalid action: {}", e)))?;

        tracing::debug!("Ludo action from {}: {:?}", user_id, action);

        let should_advance = match action {
            LudoAction::RollDice => inner.handle_roll_dice(user_id).await,
            LudoAction::MovePawn{pawn_id} => inner.handle_move_pawn(user_id, pawn_id).await,
        };

        match should_advance {
            Ok(true) => {
                // Check if game is over (someone won)
                if inner.board.get_winner().is_some() {
                    inner.end_game().await;
                } else {
                    // Signal turn advance
                    inner.turn_advance_notify.notify_one();
                }
            }
            Ok(false) => {
                // Turn continues (waiting for move or bonus turn)
            }
            Err(e) => {
                tracing::warn!("Ludo action error: {}", e);
            }
        }

        Ok(vec![])
    }

    async fn get_game_state(&self, _user_id: Option<Uuid>) -> Result<Value, AppError> {
        let inner = self.inner.read().await;

        let current_player = inner.get_current_player_state();
        let turn = current_player.map(|player| LudoEvent::Turn {
            player,
            timeout_secs: TURN_TIMEOUT_SECS,
        });

        let game_state = serde_json::json!({
            "board": serde_json::to_value(&inner.board).unwrap_or_default(),
            "turn": turn.map(|t| serde_json::to_value(&t).unwrap_or_default()),
            "turnPhase": format!("{:?}", inner.turn_phase),
            "currentDice": inner.current_dice,
            "movablePawns": inner.movable_pawns,
        });

        Ok(game_state)
    }

    fn is_finished(&self) -> bool {
        self.inner
            .try_read()
            .map(|inner| inner.finished)
            .unwrap_or(false)
    }

    fn start_loop(&mut self, state: AppState) {
        let inner = self.get_inner();
        tokio::spawn(run_game_loop(inner, state));
    }
}

// ============================================================================
// Game Loop
// ============================================================================

/// Run the game loop for Ludo
/// This handles turn timeouts and countdown broadcasts
async fn run_game_loop(inner: Arc<RwLock<LudoInner>>, state: AppState) {
    let (turn_advance_notify, lobby_id) = {
        let inner_guard = inner.read().await;
        (inner_guard.turn_advance_notify.clone(), inner_guard.lobby_id)
    };

    loop {
        // Get current turn state
        let (is_finished, has_winner) = {
            let inner_guard = inner.read().await;
            (
                inner_guard.finished,
                inner_guard.board.get_winner().is_some(),
            )
        };

        if is_finished || has_winner {
            tracing::info!("Ludo game loop ending for lobby {}", lobby_id);
            break;
        }

        // Start the turn
        {
            let mut inner_guard = inner.write().await;
            inner_guard.start_turn().await;
        }

        // Countdown loop
        let mut time_remaining = TURN_TIMEOUT_SECS;

        while time_remaining > 0 {
            // Broadcast countdown
            let countdown_event = LudoEvent::Countdown {
                time: time_remaining,
            };
            broadcast::broadcast_game_message(
                &state,
                lobby_id,
                serde_json::to_value(&countdown_event).unwrap_or_default(),
            )
            .await;

            // Wait for either: turn completion notification or 1 second timeout
            let wait_result = tokio::time::timeout(
                std::time::Duration::from_secs(1),
                turn_advance_notify.notified(),
            )
            .await;

            // Check if turn was completed
            let turn_complete = {
                let inner_guard = inner.read().await;
                inner_guard.turn_phase == TurnPhase::Complete
            };

            if wait_result.is_ok() || turn_complete {
                break;
            }

            time_remaining -= 1;
        }

        // Check if we timed out
        let (turn_phase, current_player_id) = {
            let inner_guard = inner.read().await;
            (inner_guard.turn_phase, inner_guard.turn_rotation.current_player())
        };

        if turn_phase != TurnPhase::Complete {
            // Player timed out - auto-action based on phase
            let mut inner_guard = inner.write().await;

            if let Some(player_id) = current_player_id {
                match inner_guard.turn_phase {
                    TurnPhase::WaitingForRoll => {
                        // Auto-roll dice
                        let _ = inner_guard.handle_roll_dice(player_id).await;
                    }
                    TurnPhase::WaitingForMove => {
                        // Auto-move first available pawn
                        if let Some(&pawn_id) = inner_guard.movable_pawns.first() {
                            let _ = inner_guard.handle_move_pawn(player_id, pawn_id).await;
                        }
                    }
                    TurnPhase::Complete => {}
                }
            }
        }

        // Check if game ended
        let has_winner = {
            let inner_guard = inner.read().await;
            inner_guard.board.get_winner().is_some()
        };

        if has_winner {
            let mut inner_guard = inner.write().await;
            inner_guard.end_game().await;
            break;
        }

        // Advance to next player (if turn is complete)
        {
            let mut inner_guard = inner.write().await;
            if inner_guard.turn_phase == TurnPhase::Complete {
                inner_guard.turn_rotation.next_turn();
                inner_guard.consecutive_sixes = 0;
            }
        }
    }
}

// ============================================================================
// Factory
// ============================================================================

/// Factory function to create new Ludo game instances
pub fn create_ludo(lobby_id: Uuid, state: AppState) -> Box<dyn GameEngine> {
    Box::new(LudoEngine::new(lobby_id, state))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    #[test]
    fn test_prize_calculation() {
        let total_pool = 100.0;

        // Helper to calculate prize
        let calc_prize = |rank: usize, participants: usize| -> Option<f64> {
            let prize = match rank {
                1 => {
                    if participants == 2 {
                        (total_pool * 70.0) / 100.0
                    } else {
                        (total_pool * 50.0) / 100.0
                    }
                }
                2 => (total_pool * 30.0) / 100.0,
                3 => (total_pool * 20.0) / 100.0,
                _ => 0.0,
            };
            if prize > 0.0 {
                Some(prize)
            } else {
                None
            }
        };

        // 4 players
        assert_eq!(calc_prize(1, 4), Some(50.0));
        assert_eq!(calc_prize(2, 4), Some(30.0));
        assert_eq!(calc_prize(3, 4), Some(20.0));
        assert_eq!(calc_prize(4, 4), None);

        // 2 players
        assert_eq!(calc_prize(1, 2), Some(70.0));
        assert_eq!(calc_prize(2, 2), Some(30.0));
    }
}
