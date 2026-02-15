// Ludo Game Engine
//
// Core game logic including:
// - LudoEngine struct and implementation
// - GameEngine trait implementation
// - Game loop (turn timeouts)
// - Prize/points calculation
//
// Dual-dice variant:
// - Two dice per roll (die1, die2)
// - Player can use die1, die2, or die1+die2 (sum) per pawn move
// - After using a die its value becomes 0; using sum consumes both
// - Bonus turn only when BOTH dice are 6

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

pub const ROLL_TIMEOUT_SECS: u64 = 5;
pub const MOVE_TIMEOUT_SECS: u64 = 15;

// ============================================================================
// Turn Phase
// ============================================================================

/// Represents the current phase of a player's turn
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnPhase {
    /// Waiting for player to roll dice
    WaitingForRoll,
    /// Waiting for player to select a dice value and move a pawn
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
    /// The original dice values rolled this turn
    dice: Option<(u8, u8)>,
    /// Remaining value of die 1 (0 = used)
    dice1_remaining: u8,
    /// Remaining value of die 2 (0 = used)
    dice2_remaining: u8,
    /// Currently selected dice value for pawn selection
    selected_dice_value: Option<u8>,
    /// Movable pawns for the currently selected dice value
    movable_pawns: Vec<usize>,
    /// Incremented each time a dice value is consumed (so game loop can detect moves)
    move_generation: u64,

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
            dice: None,
            dice1_remaining: 0,
            dice2_remaining: 0,
            selected_dice_value: None,
            movable_pawns: Vec::new(),
            move_generation: 0,
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

    /// Get the available dice values the player can choose from.
    /// Returns a deduplicated, sorted list of available values.
    fn available_values(&self) -> Vec<u8> {
        let mut vals = Vec::new();
        if self.dice1_remaining > 0 {
            vals.push(self.dice1_remaining);
        }
        if self.dice2_remaining > 0 {
            vals.push(self.dice2_remaining);
        }
        // Sum is available only when BOTH dice are still unused
        if self.dice1_remaining > 0 && self.dice2_remaining > 0 {
            let sum = self.dice1_remaining + self.dice2_remaining;
            vals.push(sum);
        }
        vals.sort();
        vals.dedup();
        vals
    }

    /// Check whether a dice value is the sum of both dice (not an individual die).
    fn is_sum_value(&self, value: u8) -> bool {
        self.dice1_remaining > 0
            && self.dice2_remaining > 0
            && value == self.dice1_remaining + self.dice2_remaining
            && value != self.dice1_remaining
            && value != self.dice2_remaining
    }

    /// Get playable values — subset of available_values that actually have movable pawns.
    fn playable_values(&self, user_id: Uuid) -> Vec<u8> {
        let player_board = match self.board.get_player(user_id) {
            Some(p) => p,
            None => return vec![],
        };
        self.available_values()
            .into_iter()
            .filter(|&v| !player_board.get_movable_pawns(v, self.is_sum_value(v)).is_empty())
            .collect()
    }

    /// Consume a dice value after a move.
    /// If value == sum, both dice are consumed.
    /// If value == one die, that die is consumed.
    fn consume_dice_value(&mut self, value: u8) {
        let sum = self.dice1_remaining + self.dice2_remaining;
        if self.dice1_remaining > 0 && self.dice2_remaining > 0 && value == sum {
            // Sum uses both dice
            self.dice1_remaining = 0;
            self.dice2_remaining = 0;
        } else if value == self.dice1_remaining {
            self.dice1_remaining = 0;
        } else if value == self.dice2_remaining {
            self.dice2_remaining = 0;
        }
        self.selected_dice_value = None;
        self.movable_pawns.clear();
        self.move_generation += 1;
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

    /// Calculate prize for a given rank — winner takes all in Ludo
    fn calculate_prize(&self, rank: usize, _participants: usize) -> Option<f64> {
        let total_pool = self.current_amount?;

        if total_pool <= 0.0 {
            return None;
        }

        // Winner takes the entire pool
        if rank == 1 {
            Some(total_pool)
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
        self.dice = None;
        self.dice1_remaining = 0;
        self.dice2_remaining = 0;
        self.selected_dice_value = None;
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
            timeout_secs: ROLL_TIMEOUT_SECS,
        };
        broadcast::broadcast_game_message(
            &self.state,
            self.lobby_id,
            serde_json::to_value(&turn_event).unwrap_or_default(),
        )
        .await;
    }

    // ========================================================================
    // Action: Roll Dice
    // ========================================================================

    /// Handle dice roll action — rolls TWO dice
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

        let player_board = self.board.get_player(user_id).unwrap();
        let all_home = player_board.all_pawns_at_home();

        // Roll two dice
        // If all pawns are at home, fix one die to 6 so the player can enter
        let (die1, die2) = if all_home {
            (6u8, roll_dice())
        } else {
            (roll_dice(), roll_dice())
        };

        self.dice = Some((die1, die2));
        self.dice1_remaining = die1;
        self.dice2_remaining = die2;
        self.selected_dice_value = None;
        self.movable_pawns.clear();

        // Calculate which values are playable
        let playable = self.playable_values(user_id);

        let player_state = self.get_player_state(user_id).unwrap();

        // Broadcast dice roll
        let roll_event = LudoEvent::DiceRolled {
            player: player_state.clone(),
            dice1: die1,
            dice2: die2,
            playable_values: playable.clone(),
        };
        broadcast::broadcast_game_message(
            &self.state,
            self.lobby_id,
            serde_json::to_value(&roll_event).unwrap_or_default(),
        )
        .await;

        // Check if player has any valid moves at all
        if playable.is_empty() {
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
            return Ok(true);
        }

        // Wait for player to select a dice value and then a pawn
        self.turn_phase = TurnPhase::WaitingForMove;
        Ok(false)
    }

    // ========================================================================
    // Action: Select Dice Value
    // ========================================================================

    /// Handle selecting a dice value to play with
    async fn handle_select_dice_value(&mut self, user_id: Uuid, dice_value: u8) -> Result<bool, GameError> {
        // Verify it's this player's turn
        if self.turn_rotation.current_player() != Some(user_id) {
            return Err(GameError::NotYourTurn);
        }

        // Must be in WaitingForMove phase
        if self.turn_phase != TurnPhase::WaitingForMove {
            let event = LudoEvent::Invalid {
                reason: "Not in move phase".to_string(),
            };
            broadcast::broadcast_game_message_to_user(
                &self.state,
                user_id,
                serde_json::to_value(&event).unwrap_or_default(),
            )
            .await;
            return Ok(false);
        }

        // Verify the value is available
        let available = self.available_values();
        if !available.contains(&dice_value) {
            let event = LudoEvent::Invalid {
                reason: format!("Dice value {} is not available", dice_value),
            };
            broadcast::broadcast_game_message_to_user(
                &self.state,
                user_id,
                serde_json::to_value(&event).unwrap_or_default(),
            )
            .await;
            return Ok(false);
        }

        // Get movable pawns for this value
        let is_sum = self.is_sum_value(dice_value);
        let player_board = self.board.get_player(user_id).unwrap();
        let movable = player_board.get_movable_pawns(dice_value, is_sum);

        self.selected_dice_value = Some(dice_value);
        self.movable_pawns = movable.clone();

        // Send MovablePawns event to the requesting player only
        let event = LudoEvent::MovablePawns {
            dice_value,
            pawns: movable,
        };
        broadcast::broadcast_game_message_to_user(
            &self.state,
            user_id,
            serde_json::to_value(&event).unwrap_or_default(),
        )
        .await;

        Ok(false)
    }

    // ========================================================================
    // Action: Move Pawn
    // ========================================================================

    /// Handle pawn move action — uses the currently selected dice value
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

        // Must have a selected dice value
        let dice_value = match self.selected_dice_value {
            Some(v) => v,
            None => {
                let event = LudoEvent::Invalid {
                    reason: "Select a dice value first".to_string(),
                };
                broadcast::broadcast_game_message_to_user(
                    &self.state,
                    user_id,
                    serde_json::to_value(&event).unwrap_or_default(),
                )
                .await;
                return Ok(false);
            }
        };

        // Verify the pawn can be moved with this value
        if !self.movable_pawns.contains(&pawn_id) {
            let event = LudoEvent::Invalid {
                reason: "This pawn cannot be moved with the selected dice value".to_string(),
            };
            broadcast::broadcast_game_message_to_user(
                &self.state,
                user_id,
                serde_json::to_value(&event).unwrap_or_default(),
            )
            .await;
            return Ok(false);
        }

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
            .move_pawn(pawn_id, dice_value);

        // Broadcast pawn moved
        let move_event = LudoEvent::PawnMoved {
            player: player_state.clone(),
            pawn_id,
            from: from_position,
            to: to_position,
            dice_value,
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
                self.consume_dice_value(dice_value);
                self.turn_phase = TurnPhase::Complete;
                return Ok(true);
            }
        }

        // Consume the dice value
        self.consume_dice_value(dice_value);

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

        // Broadcast DiceValueUsed with remaining values
        let remaining = self.playable_values(user_id);
        let used_event = LudoEvent::DiceValueUsed {
            dice_value,
            remaining_values: remaining.clone(),
        };
        broadcast::broadcast_game_message(
            &self.state,
            self.lobby_id,
            serde_json::to_value(&used_event).unwrap_or_default(),
        )
        .await;

        // Check if both dice are used up
        let both_used = self.dice1_remaining == 0 && self.dice2_remaining == 0;

        if both_used {
            // Check for bonus turn — only if BOTH original dice were 6
            if let Some((d1, d2)) = self.dice {
                if d1 == 6 && d2 == 6 {
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
                    self.dice = None;
                    self.dice1_remaining = 0;
                    self.dice2_remaining = 0;
                    self.selected_dice_value = None;
                    self.movable_pawns.clear();
                    return Ok(false); // Don't advance turn
                }
            }

            // Both dice used, turn is over
            self.turn_phase = TurnPhase::Complete;
            return Ok(true);
        }

        // Still have remaining dice — check if any playable moves remain
        if remaining.is_empty() {
            // No more playable moves with remaining dice
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
            return Ok(true);
        }

        // Stay in WaitingForMove for the remaining dice value(s)
        // The timer continues from the game loop
        Ok(false)
    }

    // ========================================================================
    // Auto-move logic (for timeouts)
    // ========================================================================

    /// Perform an automatic move: prefer sum, then die1, then die2
    async fn auto_move(&mut self, user_id: Uuid) {
        let playable = self.playable_values(user_id);
        if playable.is_empty() {
            self.turn_phase = TurnPhase::Complete;
            return;
        }

        // Prefer sum (largest value), then first available
        let sum_available = if self.dice1_remaining > 0 && self.dice2_remaining > 0 {
            let s = self.dice1_remaining + self.dice2_remaining;
            if playable.contains(&s) { Some(s) } else { None }
        } else {
            None
        };

        let chosen = sum_available.unwrap_or(playable[0]);

        // Select dice value
        let is_sum = self.is_sum_value(chosen);
        let player_board = self.board.get_player(user_id).unwrap();
        let movable = player_board.get_movable_pawns(chosen, is_sum);
        if movable.is_empty() {
            self.turn_phase = TurnPhase::Complete;
            return;
        }

        self.selected_dice_value = Some(chosen);
        self.movable_pawns = movable.clone();

        // Move first available pawn
        let pawn_id = movable[0];
        let _ = self.handle_move_pawn(user_id, pawn_id).await;
    }

    // ========================================================================
    // Elimination / End game
    // ========================================================================

    /// Eliminate a player (called when they quit)
    async fn eliminate_player(&mut self, player_id: Uuid, reason: &str) {
        let remaining = self.turn_rotation.active_count();
        let rank = remaining;
        let prize = self.calculate_prize(rank, self.total_players);

        self.turn_rotation.eliminate_player(player_id);

        if let Some(player) = self.players.get_mut(&player_id) {
            player.eliminate();
        }

        // Save result
        let ctx = self.build_wars_point_context(player_id, rank, prize);
        let wars_point = match save_player_result(&self.state, self.lobby_id, &ctx).await {
            Ok(result) => result.wars_point,
            Err(e) => {
                tracing::error!("Failed to save player result: {}", e);
                calculate_wars_point(&ctx)
            }
        };

        // Update player_state
        if let Some(ps) = self.player_states.get_mut(&player_id) {
            ps.rank = Some(rank);
            ps.prize = prize;
            ps.wars_point = Some(wars_point);
        }

        // Broadcast PlayerQuit event
        if let Some(player) = self.player_states.get(&player_id).cloned() {
            let event = LudoEvent::PlayerQuit {
                player,
                reason: reason.to_string(),
            };
            broadcast::broadcast_game_message(
                &self.state,
                self.lobby_id,
                serde_json::to_value(&event).unwrap_or_default(),
            )
            .await;
        }

        // Send GameOver to the quitting player
        let game_over = RoomServerMessage::GameOver {
            rank,
            prize,
            wars_point,
        };
        broadcast::broadcast_user(&self.state, player_id, &game_over).await;
    }

    /// End the game when the last player standing wins (due to other players quitting)
    async fn end_game_last_standing(&mut self) {
        self.finished = true;

        let state = self.state.clone();
        let lobby_id = self.lobby_id;
        let participants = self.total_players;

        let active_players = self.turn_rotation.active_players();
        let mut final_standings: Vec<PlayerState> = Vec::new();
        let mut player_rankings: Vec<PlayerRanking> = Vec::new();

        // Process the winner (remaining active player)
        for &winner_id in &active_players {
            let rank = 1;
            let prize = self.calculate_prize(rank, participants);

            let ctx = self.build_wars_point_context(winner_id, rank, prize);
            let wars_point = match save_player_result(&state, lobby_id, &ctx).await {
                Ok(result) => result.wars_point,
                Err(e) => {
                    tracing::error!("Failed to save player result: {}", e);
                    calculate_wars_point(&ctx)
                }
            };

            if let Some(ps) = self.player_states.get_mut(&winner_id) {
                ps.rank = Some(rank);
                ps.prize = prize;
                ps.wars_point = Some(wars_point);
                final_standings.push(ps.clone());
            }

            player_rankings.push(PlayerRanking {
                user_id: winner_id,
                rank,
                score: None,
                prize,
            });

            // Send GameOver to winner
            let game_over = RoomServerMessage::GameOver {
                rank,
                prize,
                wars_point,
            };
            broadcast::broadcast_user(&state, winner_id, &game_over).await;
        }

        // Add already-eliminated players to final standings
        for ps in self.player_states.values() {
            if !active_players.contains(&ps.user_id) {
                final_standings.push(ps.clone());
            }
        }

        // Sort standings by rank
        final_standings.sort_by_key(|ps| ps.rank.unwrap_or(usize::MAX));

        // Broadcast FinalStanding
        let final_standing = RoomServerMessage::FinalStanding {
            standings: final_standings,
        };
        broadcast::broadcast_room(&state, lobby_id, &final_standing).await;

        // Finish lobby
        if let Err(e) = finish_lobby(&state, lobby_id).await {
            tracing::error!("Failed to finish lobby {}: {}", lobby_id, e);
        }

        self.results = Some(GameResults {
            rankings: player_rankings,
            finished_at: chrono::Utc::now().timestamp(),
            metadata: None,
        });
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
            LudoAction::SelectDiceValue { dice_value } => inner.handle_select_dice_value(user_id, dice_value).await,
            LudoAction::MovePawn { pawn_id } => inner.handle_move_pawn(user_id, pawn_id).await,
        };

        match should_advance {
            Ok(_) => {
                // Check if game is over (someone won)
                if inner.board.get_winner().is_some() {
                    inner.end_game().await;
                }
                // Always signal the game loop so it can react to phase changes
                inner.turn_advance_notify.notify_one();
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
            timeout_secs: ROLL_TIMEOUT_SECS,
        });

        let game_state = serde_json::json!({
            "board": serde_json::to_value(&inner.board).unwrap_or_default(),
            "turn": turn.map(|t| serde_json::to_value(&t).unwrap_or_default()),
            "turnPhase": format!("{:?}", inner.turn_phase),
            "dice1": inner.dice.map(|(d1, _)| d1),
            "dice2": inner.dice.map(|(_, d2)| d2),
            "dice1Remaining": inner.dice1_remaining,
            "dice2Remaining": inner.dice2_remaining,
            "selectedDiceValue": inner.selected_dice_value,
            "movablePawns": inner.movable_pawns,
            "playableValues": inner.turn_rotation.current_player()
                .map(|uid| inner.playable_values(uid))
                .unwrap_or_default(),
        });

        Ok(game_state)
    }

    async fn handle_player_quit(&mut self, user_id: Uuid) -> Result<Vec<Value>, AppError> {
        let mut inner = self.inner.write().await;

        if inner.finished {
            return Ok(vec![]);
        }

        // Check if player is active
        if !inner.turn_rotation.active_players().contains(&user_id) {
            return Ok(vec![]);
        }

        let is_current_player = inner.turn_rotation.current_player() == Some(user_id);

        // Eliminate the player
        inner.eliminate_player(user_id, "Player quit the game").await;

        // Check if game should end (1 or fewer active players)
        if inner.turn_rotation.active_count() <= 1 {
            inner.end_game_last_standing().await;
            inner.turn_advance_notify.notify_one();
        } else if is_current_player {
            // Current player quit, complete their turn so the game loop advances
            inner.turn_phase = TurnPhase::Complete;
            inner.turn_advance_notify.notify_one();
        }

        Ok(vec![])
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
        // Check if game should end
        let (is_finished, has_winner) = {
            let inner_guard = inner.read().await;
            (inner_guard.finished, inner_guard.board.get_winner().is_some())
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

        // Run the turn (handles bonus turns from double-6)
        run_turn(&inner, &state, &turn_advance_notify, lobby_id).await;

        // Check if game ended
        let has_winner = {
            let inner_guard = inner.read().await;
            inner_guard.board.get_winner().is_some()
        };
        if has_winner {
            let mut inner_guard = inner.write().await;
            if !inner_guard.finished {
                inner_guard.end_game().await;
            }
            break;
        }

        // Advance to next player (if turn is complete)
        {
            let mut inner_guard = inner.write().await;
            if inner_guard.turn_phase == TurnPhase::Complete {
                inner_guard.turn_rotation.next_turn();
            }
        }
    }
}

/// Run a single turn (including bonus turns from double-6)
async fn run_turn(
    inner: &Arc<RwLock<LudoInner>>,
    state: &AppState,
    notify: &Arc<Notify>,
    lobby_id: Uuid,
) {
    loop {
        // --- Phase 1: Wait for dice roll (ROLL_TIMEOUT_SECS) ---
        let roll_result = run_phase_countdown(
            inner, state, notify, lobby_id,
            ROLL_TIMEOUT_SECS,
            TurnPhase::WaitingForRoll,
        ).await;

        // If timed out while waiting for roll, auto-roll
        if matches!(roll_result, CountdownResult::TimedOut(TurnPhase::WaitingForRoll)) {
            let mut inner_guard = inner.write().await;
            if let Some(player_id) = inner_guard.turn_rotation.current_player() {
                let _ = inner_guard.handle_roll_dice(player_id).await;
            }
        }

        // Re-check phase after (auto-)roll
        let phase = {
            let inner_guard = inner.read().await;
            inner_guard.turn_phase
        };

        if phase == TurnPhase::Complete {
            break;
        }

        // Check if game ended (winner detected during auto-roll)
        let finished = {
            let inner_guard = inner.read().await;
            inner_guard.finished || inner_guard.board.get_winner().is_some()
        };
        if finished {
            break;
        }

        // --- Phase 2: Move loop — keep giving fresh countdowns until dice are used up ---
        while {
            let inner_guard = inner.read().await;
            inner_guard.turn_phase == TurnPhase::WaitingForMove
        } {
            let move_result = run_phase_countdown(
                inner, state, notify, lobby_id,
                MOVE_TIMEOUT_SECS,
                TurnPhase::WaitingForMove,
            ).await;

            match move_result {
                CountdownResult::TimedOut(TurnPhase::WaitingForMove) => {
                    // Timeout — auto-move with remaining dice
                    let mut inner_guard = inner.write().await;
                    if let Some(player_id) = inner_guard.turn_rotation.current_player() {
                        inner_guard.auto_move(player_id).await;
                    }
                    // auto_move will consume remaining dice and set phase to Complete,
                    // or WaitingForRoll for bonus turn — the while condition handles it
                }
                CountdownResult::MoveDetected => {
                    // Player used one die — the while loop will check if still WaitingForMove
                    // and give a fresh MOVE_TIMEOUT_SECS countdown
                    continue;
                }
                CountdownResult::PhaseChanged() => {
                    // Phase changed (Complete, WaitingForRoll for bonus, etc.)
                    break;
                }
                _ => break,
            }
        }

        // Check if game ended
        let finished = {
            let inner_guard = inner.read().await;
            inner_guard.finished || inner_guard.board.get_winner().is_some()
        };
        if finished {
            break;
        }

        // Check if bonus turn (double-6 → phase went back to WaitingForRoll)
        let phase = {
            let inner_guard = inner.read().await;
            inner_guard.turn_phase
        };

        if phase == TurnPhase::WaitingForRoll {
            // Bonus turn — loop back to roll phase
            continue;
        }

        // Turn complete
        break;
    }
}

/// Result from run_phase_countdown indicating why it returned.
enum CountdownResult {
    /// Phase changed (player acted, game ended, etc.)
    PhaseChanged(),
    /// A die was consumed but phase stayed the same — need fresh countdown
    MoveDetected,
    /// Timer expired without any action
    TimedOut(TurnPhase),
}

/// Run countdown for a specific phase.
/// Returns a CountdownResult indicating why the countdown ended.
async fn run_phase_countdown(
    inner: &Arc<RwLock<LudoInner>>,
    state: &AppState,
    notify: &Arc<Notify>,
    lobby_id: Uuid,
    timeout_secs: u64,
    expected_phase: TurnPhase,
) -> CountdownResult {
    let mut time_remaining = timeout_secs;

    // Capture the current move generation so we can detect when a die is consumed
    let start_generation = {
        let inner_guard = inner.read().await;
        inner_guard.move_generation
    };

    while time_remaining > 0 {
        // Broadcast countdown
        let countdown_event = LudoEvent::Countdown {
            time: time_remaining,
        };
        broadcast::broadcast_game_message(
            state,
            lobby_id,
            serde_json::to_value(&countdown_event).unwrap_or_default(),
        )
        .await;

        // Wait for notification or 1 second tick
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            notify.notified(),
        )
        .await;

        // Check if phase changed, game finished, or a move was made
        let (current_phase, finished, current_gen) = {
            let inner_guard = inner.read().await;
            (inner_guard.turn_phase, inner_guard.finished, inner_guard.move_generation)
        };

        if finished {
            return CountdownResult::PhaseChanged();
        }

        if current_phase != expected_phase {
            return CountdownResult::PhaseChanged();
        }

        // A die was consumed (move made) — exit so run_turn can restart the countdown
        if current_gen != start_generation {
            return CountdownResult::MoveDetected;
        }

        time_remaining -= 1;
    }

    // Broadcast 0 so clients see timer expire
    let countdown_event = LudoEvent::Countdown { time: 0 };
    broadcast::broadcast_game_message(
        state,
        lobby_id,
        serde_json::to_value(&countdown_event).unwrap_or_default(),
    )
    .await;

    // Return the current phase (unchanged = timed out)
    let inner_guard = inner.read().await;
    CountdownResult::TimedOut(inner_guard.turn_phase)
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
    fn test_prize_calculation_winner_takes_all() {
        let total_pool = 100.0;

        // Winner takes all — only rank 1 gets prize
        let calc_prize = |rank: usize| -> Option<f64> {
            if rank == 1 {
                Some(total_pool)
            } else {
                None
            }
        };

        // 4 players
        assert_eq!(calc_prize(1), Some(100.0));
        assert_eq!(calc_prize(2), None);
        assert_eq!(calc_prize(3), None);
        assert_eq!(calc_prize(4), None);

        // 2 players — same rule
        assert_eq!(calc_prize(1), Some(100.0));
        assert_eq!(calc_prize(2), None);
    }
}
