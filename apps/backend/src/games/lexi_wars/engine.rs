// Lexi Wars Game Engine
//
// Core game logic including:
// - LexiWarsEngine struct and implementation
// - GameEngine trait implementation
// - Game loop (start_game_loop)
// - Prize/points calculation
// - Word validation

use crate::{
    db::{
        player_state::PlayerStateRepository, season::SeasonRepository,
        user_wars_points::UserWarsPointsRepository,
    },
    errors::AppError,
    games::{GameEngine, GameError, GameResults, common::*},
    models::PlayerState,
    state::AppState,
    ws::{broadcast, room::messages::RoomServerMessage},
};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::{Notify, RwLock};
use uuid::Uuid;

use super::message::{LexiWarsAction, LexiWarsEvent};
use super::rule::{Rule, RuleContext, get_rule_at_index, rule_count};

// ============================================================================
// Constants
// ============================================================================

pub const TURN_TIMEOUT_SECS: u64 = 15;
pub const INITIAL_MIN_WORD_LENGTH: usize = 4;
pub const WORD_LENGTH_INCREMENT: usize = 2;

// Load dictionary at compile time
static DICTIONARY: Lazy<HashSet<String>> = Lazy::new(|| {
    let dict_json = include_str!("../../assets/dictionary.json");
    serde_json::from_str(dict_json).unwrap_or_default()
});

// ============================================================================
// Inner State (shared via Arc<RwLock>)
// ============================================================================

/// Inner state that is shared between the GameEngine methods and the game loop
struct LexiWarsInner {
    lobby_id: Uuid,
    players: HashMap<Uuid, GamePlayerState>,
    player_states: HashMap<Uuid, PlayerState>,
    turn_rotation: TurnRotation,
    used_words: HashSet<String>,
    current_round: usize,
    current_rule_index: usize,
    current_min_word_length: usize,
    current_rule: Option<Rule>,
    current_rule_context: Option<RuleContext>,
    total_players: usize,
    finished: bool,
    results: Option<GameResults>,

    // Prize/points calculation context
    entry_amount: Option<f64>,
    current_amount: Option<f64>,
    is_sponsored: bool,
    creator_id: Option<Uuid>,

    // Game loop control - Notify is used to signal valid word submission
    turn_advance_notify: Arc<Notify>,
    app_state: Option<AppState>,
}

impl LexiWarsInner {
    fn new(lobby_id: Uuid) -> Self {
        Self {
            lobby_id,
            players: HashMap::new(),
            player_states: HashMap::new(),
            turn_rotation: TurnRotation::new(Vec::new()),
            used_words: HashSet::new(),
            current_round: 0,
            current_rule_index: 0,
            current_min_word_length: INITIAL_MIN_WORD_LENGTH,
            current_rule: None,
            current_rule_context: None,
            total_players: 0,
            finished: false,
            results: None,
            entry_amount: None,
            current_amount: None,
            is_sponsored: false,
            creator_id: None,
            turn_advance_notify: Arc::new(Notify::new()),
            app_state: None,
        }
    }
}

// ============================================================================
// Game Engine (wraps inner state in Arc<RwLock>)
// ============================================================================

/// Lexi Wars game engine - wraps shared state for thread-safe access
pub struct LexiWarsEngine {
    inner: Arc<RwLock<LexiWarsInner>>,
}

impl LexiWarsEngine {
    pub fn new(lobby_id: Uuid) -> Self {
        Self {
            inner: Arc::new(RwLock::new(LexiWarsInner::new(lobby_id))),
        }
    }

    /// Get the Arc<RwLock<Inner>> for spawning game loop
    fn get_inner(&self) -> Arc<RwLock<LexiWarsInner>> {
        self.inner.clone()
    }

    /// Set app state for broadcasting
    pub async fn set_app_state(&self, state: AppState) {
        let mut inner = self.inner.write().await;
        inner.app_state = Some(state);
    }

    /// Set lobby context for prize/points calculation
    pub async fn set_lobby_context(
        &self,
        entry_amount: Option<f64>,
        current_amount: Option<f64>,
        is_sponsored: bool,
        creator_id: Uuid,
    ) {
        let mut inner = self.inner.write().await;
        inner.entry_amount = entry_amount;
        inner.current_amount = current_amount;
        inner.is_sponsored = is_sponsored;
        inner.creator_id = Some(creator_id);
    }
}

// ============================================================================
// Inner State Methods
// ============================================================================

impl LexiWarsInner {
    /// Validate a word against dictionary
    fn is_valid_dictionary_word(&self, word: &str) -> bool {
        DICTIONARY.contains(&word.to_lowercase())
    }

    /// Check if word has been used
    fn is_word_used(&self, word: &str) -> bool {
        self.used_words.contains(&word.to_lowercase())
    }

    /// Get the current player's PlayerState
    fn get_current_player_state(&self) -> Option<PlayerState> {
        self.turn_rotation
            .current_player()
            .and_then(|id| self.player_states.get(&id).cloned())
    }

    /// Get a player's PlayerState by ID
    fn get_player_state(&self, user_id: Uuid) -> Option<PlayerState> {
        self.player_states.get(&user_id).cloned()
    }

    /// Advance to the next rule (cycling through rules, increasing difficulty after full cycle)
    fn advance_rule(&mut self) {
        self.current_rule_index += 1;

        // Check if we've completed a full cycle of rules
        if self.current_rule_index >= rule_count() {
            self.current_rule_index = 0;
            self.current_min_word_length += WORD_LENGTH_INCREMENT;
            self.current_round += 1;
            tracing::info!(
                "LexiWars: Rule cycle complete, increasing min word length to {}",
                self.current_min_word_length
            );
        }

        // Create new context with regenerated letter
        let ctx = RuleContext::new(
            self.current_round,
            self.current_rule_index,
            self.current_min_word_length,
        );
        let rule = get_rule_at_index(&ctx);

        self.current_rule_context = Some(ctx);
        self.current_rule = Some(rule);
    }

    /// Initialize the first rule
    fn init_first_rule(&mut self) {
        self.current_round = 1;
        self.current_rule_index = 0;
        self.current_min_word_length = INITIAL_MIN_WORD_LENGTH;

        let ctx = RuleContext::new(
            self.current_round,
            self.current_rule_index,
            self.current_min_word_length,
        );
        let rule = get_rule_at_index(&ctx);

        self.current_rule_context = Some(ctx);
        self.current_rule = Some(rule);
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

        if prize > 0.0 { Some(prize) } else { None }
    }

    /// Calculate wars points for a player
    fn calculate_wars_point(&self, user_id: Uuid, rank: usize, participants: usize) -> f64 {
        let base_point = ((participants - rank + 1) * 2) as f64;
        let mut total_point = base_point;

        // Pool bonus for non-sponsored games
        if !self.is_sponsored {
            if let (Some(entry_amount), Some(current_amount)) =
                (self.entry_amount, self.current_amount)
            {
                if entry_amount > 0.0 {
                    let pool_bonus = (current_amount / participants as f64) + (entry_amount / 5.0);
                    total_point += pool_bonus;
                }
            }
        }

        // Sponsor bonus if this is a sponsored lobby and the player is the creator
        if self.is_sponsored {
            if let Some(creator_id) = self.creator_id {
                if user_id == creator_id {
                    let sponsor_bonus = 2.5 * self.turn_rotation.active_count() as f64;
                    total_point += sponsor_bonus;
                }
            }
        }

        // Cap at 50 points maximum
        total_point.min(50.0)
    }

    /// Eliminate a player (called on timeout)
    /// This also calculates and sends GameOver to the eliminated player
    async fn eliminate_player(&mut self, player_id: Uuid, reason: &str) {
        // Calculate rank, prize and wars_point before elimination
        // Rank is based on remaining active players + 1 (last eliminated = highest rank number)
        let remaining = self.turn_rotation.active_count();
        let rank = remaining + 1; // e.g., if 2 players remain, eliminated player gets rank 3
        let prize = self.calculate_prize(rank, self.total_players);
        let wars_point = self.calculate_wars_point(player_id, rank, self.total_players);

        self.turn_rotation.eliminate_player(player_id);

        if let Some(player_state) = self.players.get_mut(&player_id) {
            player_state.eliminate();
        }

        // Update player_state with rank, prize, wars_point
        if let Some(ps) = self.player_states.get_mut(&player_id) {
            ps.rank = Some(rank);
            ps.prize = prize;
        }

        // Save to Redis and PostgreSQL
        if let Some(state) = &self.app_state {
            let player_repo = PlayerStateRepository::new(state.redis.clone());
            let _ = player_repo
                .set_result(self.lobby_id, player_id, rank, prize)
                .await;

            // Save wars points
            let season_repo = SeasonRepository::new(state.postgres.clone());
            if let Ok(season_id) = season_repo.get_current_season().await {
                let wars_points_repo = UserWarsPointsRepository::new(state.postgres.clone());
                let _ = wars_points_repo
                    .upsert_wars_points(player_id, season_id, wars_point)
                    .await;
            }
        }

        // Broadcast Eliminated event to room (game-specific)
        if let (Some(state), Some(player)) =
            (&self.app_state, self.player_states.get(&player_id).cloned())
        {
            let event = LexiWarsEvent::Eliminated {
                player,
                reason: reason.to_string(),
            };
            broadcast::broadcast_game_message(
                state,
                self.lobby_id,
                serde_json::to_value(&event).unwrap_or_default(),
            )
            .await;

            // Send GameOver to the eliminated player (shared event via RoomServerMessage)
            let game_over = RoomServerMessage::GameOver {
                rank,
                prize,
                wars_point,
            };
            broadcast::broadcast_user(state, player_id, &game_over).await;

            // Broadcast PlayersCount update
            let count_event = LexiWarsEvent::PlayersCount {
                remaining: self.turn_rotation.active_count(),
                total: self.total_players,
            };
            broadcast::broadcast_game_message(
                state,
                self.lobby_id,
                serde_json::to_value(&count_event).unwrap_or_default(),
            )
            .await;
        }
    }

    /// End the game and calculate final standings
    /// Sends GameOver to remaining players (winner(s)) and FinalStanding to room
    async fn end_game(&mut self) {
        self.finished = true;

        // Build rankings from player states
        let player_game_states: Vec<GamePlayerState> = self.players.values().cloned().collect();
        let results = GameResults::from_game_states(player_game_states);

        // Get remaining active players (they need GameOver too)
        let active_player_ids: Vec<Uuid> = self.turn_rotation.active_players().clone();

        // Update player states with rank, prize, wars_point
        let participants = self.total_players;
        let mut final_standings: Vec<PlayerState> = Vec::new();

        for ranking in &results.rankings {
            let prize = self.calculate_prize(ranking.rank, participants);
            let wars_point = self.calculate_wars_point(ranking.user_id, ranking.rank, participants);

            if let Some(player_state) = self.player_states.get_mut(&ranking.user_id) {
                player_state.rank = Some(ranking.rank);
                player_state.prize = prize;
                final_standings.push(player_state.clone());
            }

            // Save to Redis player_state and PostgreSQL user_wars_points
            if let Some(state) = &self.app_state {
                let player_repo = PlayerStateRepository::new(state.redis.clone());
                let _ = player_repo
                    .set_result(self.lobby_id, ranking.user_id, ranking.rank, prize)
                    .await;

                // Save wars points to user_wars_points table for current season
                let season_repo = SeasonRepository::new(state.postgres.clone());
                if let Ok(season_id) = season_repo.get_current_season().await {
                    let wars_points_repo = UserWarsPointsRepository::new(state.postgres.clone());
                    let _ = wars_points_repo
                        .upsert_wars_points(ranking.user_id, season_id, wars_point)
                        .await;
                }

                // Send GameOver to active players (those not yet eliminated)
                // They receive GameOver at end_game, not during elimination
                if active_player_ids.contains(&ranking.user_id) {
                    let game_over = RoomServerMessage::GameOver {
                        rank: ranking.rank,
                        prize,
                        wars_point,
                    };
                    broadcast::broadcast_user(state, ranking.user_id, &game_over).await;
                }
            }
        }

        // Broadcast FinalStanding to room (shared event via RoomServerMessage)
        if let Some(state) = &self.app_state {
            let final_standing = RoomServerMessage::FinalStanding {
                standings: final_standings,
            };
            broadcast::broadcast_room(state, self.lobby_id, &final_standing).await;
        }

        self.results = Some(results);
    }

    /// Start the turn for the current player
    async fn start_turn(&mut self) {
        let Some(current_player_id) = self.turn_rotation.current_player() else {
            return;
        };

        let Some(current_player_state) = self.get_player_state(current_player_id) else {
            return;
        };

        let Some(state) = &self.app_state else {
            return;
        };

        // Broadcast Turn event to room
        let turn_event = LexiWarsEvent::Turn {
            player: current_player_state.clone(),
            timeout_secs: TURN_TIMEOUT_SECS,
        };
        broadcast::broadcast_game_message(
            state,
            self.lobby_id,
            serde_json::to_value(&turn_event).unwrap_or_default(),
        )
        .await;

        // Broadcast Rule event to room
        // Current player gets Some(rule), others get None (to clear previous rule)
        let rule_for_current = self.current_rule.as_ref().map(|r| r.to_client_rule());

        // Send Rule with Some(rule) to current player
        let rule_event = LexiWarsEvent::Rule {
            rule: rule_for_current.clone(),
        };
        broadcast::broadcast_game_message_to_user(
            state,
            current_player_id,
            serde_json::to_value(&rule_event).unwrap_or_default(),
        )
        .await;

        // Send Rule with None to all other players in room (to clear their UI)
        let rule_event_clear = LexiWarsEvent::Rule { rule: None };
        broadcast::broadcast_game_message_to_room_except(
            state,
            self.lobby_id,
            current_player_id,
            serde_json::to_value(&rule_event_clear).unwrap_or_default(),
        )
        .await;
    }

    /// Handle word submission
    fn handle_submit_word(
        &mut self,
        user_id: Uuid,
        word: String,
    ) -> Result<Vec<LexiWarsEvent>, GameError> {
        let mut events = Vec::new();
        let word_lower = word.to_lowercase().trim().to_string();

        // Verify it's this player's turn
        if self.turn_rotation.current_player() != Some(user_id) {
            return Err(GameError::NotYourTurn);
        }

        // Verify player is in the game and not eliminated
        if !self.players.contains_key(&user_id) {
            return Err(GameError::NotInGame);
        }

        if self
            .turn_rotation
            .active_players()
            .iter()
            .all(|p| *p != user_id)
        {
            return Err(GameError::AlreadyEliminated);
        }

        // Check if word has been used - send only to submitting user
        if self.is_word_used(&word_lower) {
            events.push(LexiWarsEvent::UsedWord { word: word_lower });
            return Ok(events);
        }

        // Validate against dictionary - send only to submitting user
        if !self.is_valid_dictionary_word(&word_lower) {
            events.push(LexiWarsEvent::Invalid {
                reason: format!("'{}' is not in the dictionary", word),
            });
            return Ok(events);
        }

        // Validate against current rule - send only to submitting user
        if let (Some(rule), Some(ctx)) = (&self.current_rule, &self.current_rule_context) {
            if let Err(reason) = (rule.validate)(&word_lower, ctx) {
                events.push(LexiWarsEvent::Invalid { reason });
                return Ok(events);
            }
        }

        // Word is valid! Mark as used
        self.used_words.insert(word_lower.clone());

        // Get player state for WordEntry event
        let player_state = self.get_player_state(user_id);

        if let Some(player) = player_state {
            events.push(LexiWarsEvent::WordEntry {
                word: word_lower,
                player,
            });
        }

        Ok(events)
    }
}

// ============================================================================
// GameEngine Trait Implementation
// ============================================================================

#[async_trait]
impl GameEngine for LexiWarsEngine {
    async fn set_state(&mut self, state: AppState) {
        let mut inner = self.inner.write().await;
        inner.app_state = Some(state);
    }

    async fn initialize(&mut self, player_ids: Vec<Uuid>) -> Result<Vec<Value>, AppError> {
        tracing::info!("Initializing LexiWars with {} players", player_ids.len());

        if player_ids.len() < 2 {
            // Return StartFailed as a shared event (RoomServerMessage)
            let event = RoomServerMessage::GameStartFailed {
                reason: "Need at least 2 players to start".to_string(),
            };
            return Ok(vec![
                serde_json::to_value(event).map_err(|e| AppError::Serialization(e.to_string()))?,
            ]);
        }

        let mut inner = self.inner.write().await;

        inner.total_players = player_ids.len();
        inner.players = player_ids
            .iter()
            .map(|&id| (id, GamePlayerState::new(id)))
            .collect();
        inner.turn_rotation = TurnRotation::new(player_ids.clone());

        // Load player states from Redis
        if let Some(state) = &inner.app_state {
            let player_repo = PlayerStateRepository::new(state.redis.clone());
            if let Ok(states) = player_repo.get_all_in_lobby(inner.lobby_id).await {
                for ps in states {
                    inner.player_states.insert(ps.user_id, ps);
                }
            }
        }

        // Initialize first rule
        inner.init_first_rule();

        // Send GameStarted event (room-level, no game-specific fields)
        let events = vec![
            serde_json::to_value(RoomServerMessage::GameStarted)
                .map_err(|e| AppError::Serialization(e.to_string()))?,
        ];

        Ok(events)
    }

    async fn handle_action(
        &mut self,
        user_id: Uuid,
        action: Value,
    ) -> Result<Vec<Value>, AppError> {
        let mut inner = self.inner.write().await;

        if inner.finished {
            return Err(AppError::BadRequest(
                "Game has already finished".to_string(),
            ));
        }

        let action: LexiWarsAction = serde_json::from_value(action)
            .map_err(|e| AppError::BadRequest(format!("Invalid LexiWars action: {}", e)))?;

        tracing::debug!("LexiWars action from {}: {:?}", user_id, action);

        let game_events = match action {
            LexiWarsAction::SubmitWord { word } => {
                let events = inner.handle_submit_word(user_id, word)?;

                // Check if we got a valid WordEntry (not UsedWord or Invalid)
                let has_valid_word = events
                    .iter()
                    .any(|e| matches!(e, LexiWarsEvent::WordEntry { .. }));

                if has_valid_word {
                    // Signal the game loop to advance turn
                    inner.turn_advance_notify.notify_one();
                }

                events
            }
        };

        // Convert to JSON
        game_events
            .into_iter()
            .map(|e| serde_json::to_value(e).map_err(|e| AppError::Serialization(e.to_string())))
            .collect()
    }

    async fn get_bootstrap(&self) -> Result<Value, AppError> {
        let inner = self.inner.read().await;

        let current_player = inner.get_current_player_state();
        let active_players: Vec<PlayerState> = inner
            .turn_rotation
            .active_players()
            .iter()
            .filter_map(|id| inner.player_states.get(id).cloned())
            .collect();

        let bootstrap = serde_json::json!({
            "gameId": inner.lobby_id,
            "status": if inner.finished { "finished" } else { "inProgress" },
            "currentPlayer": current_player,
            "activePlayers": active_players,
            "currentRound": inner.current_round,
            "currentRuleIndex": inner.current_rule_index,
            "minWordLength": inner.current_min_word_length,
            "timeoutSecs": TURN_TIMEOUT_SECS,
            "usedWordsCount": inner.used_words.len(),
            "totalPlayers": inner.total_players,
            "remainingPlayers": inner.turn_rotation.active_count(),
        });

        Ok(bootstrap)
    }

    async fn get_game_state(&self, user_id: Option<Uuid>) -> Result<Value, AppError> {
        let inner = self.inner.read().await;

        // PlayersCount
        let players_count = LexiWarsEvent::PlayersCount {
            remaining: inner.turn_rotation.active_count(),
            total: inner.total_players,
        };

        // Turn - current player info
        let current_player = inner.get_current_player_state();
        let turn = current_player.as_ref().map(|player| LexiWarsEvent::Turn {
            player: player.clone(),
            timeout_secs: TURN_TIMEOUT_SECS,
        });

        // Rule - Some(rule) for current player, None for others
        let is_current_player = match user_id {
            Some(uid) => inner.turn_rotation.current_player() == Some(uid),
            None => false,
        };

        let rule = LexiWarsEvent::Rule {
            rule: if is_current_player {
                inner.current_rule.as_ref().map(|r| r.to_client_rule())
            } else {
                None
            },
        };

        // Countdown - we don't track exact remaining time in state,
        // but the game loop will broadcast the next countdown tick
        // For now, we'll use the full timeout; the next tick will correct it
        let countdown = LexiWarsEvent::Countdown {
            time: TURN_TIMEOUT_SECS,
        };

        let game_state = serde_json::json!({
            "playersCount": serde_json::to_value(&players_count).unwrap_or_default(),
            "turn": turn.map(|t| serde_json::to_value(&t).unwrap_or_default()),
            "rule": serde_json::to_value(&rule).unwrap_or_default(),
            "countdown": serde_json::to_value(&countdown).unwrap_or_default(),
        });

        Ok(game_state)
    }

    async fn get_results(&self) -> Result<Option<GameResults>, AppError> {
        let inner = self.inner.read().await;
        Ok(inner.results.clone())
    }

    async fn tick(&mut self) -> Result<Vec<Value>, AppError> {
        // Tick is handled by the game loop spawned in start_loop
        Ok(Vec::new())
    }

    fn is_finished(&self) -> bool {
        // This is sync, so we use try_read to avoid blocking
        // Default to false if lock can't be acquired
        self.inner
            .try_read()
            .map(|inner| inner.finished)
            .unwrap_or(false)
    }

    fn start_loop(&mut self, state: AppState) {
        // Clone the inner Arc to pass to the spawned task
        let inner = self.get_inner();
        tokio::spawn(run_game_loop(inner, state));
    }
}

// ============================================================================
// Game Loop
// ============================================================================

/// Run the game loop for Lexi Wars
/// This handles turn timeouts and countdown broadcasts
///
/// Flow:
/// 1. Check if game finished or only 1 player left → end_game() + FinalStanding
/// 2. Broadcast Turn event to room
/// 3. Send Rule event to current player only
/// 4. Start countdown loop (TURN_TIMEOUT_SECS)
/// 5. Each second: broadcast Countdown event
/// 6. Wait for either:
///    - turn_advance_notify (valid word submitted) → advance turn
///    - timeout → Eliminated event + advance turn or end_game
/// 7. Loop back to step 1
async fn run_game_loop(inner: Arc<RwLock<LexiWarsInner>>, state: AppState) {
    // Get the notify handle and lobby_id
    let (turn_advance_notify, lobby_id, total_players) = {
        let mut inner_guard = inner.write().await;
        inner_guard.app_state = Some(state.clone());
        (
            inner_guard.turn_advance_notify.clone(),
            inner_guard.lobby_id,
            inner_guard.total_players,
        )
    };

    // Broadcast initial PlayersCount at game start
    let players_count = LexiWarsEvent::PlayersCount {
        remaining: total_players,
        total: total_players,
    };
    broadcast::broadcast_game_message(
        &state,
        lobby_id,
        serde_json::to_value(&players_count).unwrap_or_default(),
    )
    .await;

    loop {
        // Get current turn state
        let (is_finished, active_count, current_player_id) = {
            let inner_guard = inner.read().await;
            (
                inner_guard.finished,
                inner_guard.turn_rotation.active_count(),
                inner_guard.turn_rotation.current_player(),
            )
        };

        if is_finished {
            tracing::info!("LexiWars game loop ending - game finished");
            break;
        }

        // Check if game should end (1 or fewer players)
        if active_count <= 1 {
            let mut inner_guard = inner.write().await;
            inner_guard.end_game().await;
            break;
        }

        // Start the turn - broadcasts Turn to room and Rule to current player
        {
            let mut inner_guard = inner.write().await;
            inner_guard.start_turn().await;
        }

        // Countdown loop
        let mut time_remaining = TURN_TIMEOUT_SECS;
        let mut word_submitted = false;

        while time_remaining > 0 {
            // Broadcast Countdown event to room
            let countdown_event = LexiWarsEvent::Countdown {
                time: time_remaining,
            };
            broadcast::broadcast_game_message(
                &state,
                lobby_id,
                serde_json::to_value(&countdown_event).unwrap_or_default(),
            )
            .await;

            // Wait 1 second or for turn_advance_notify (valid word submitted)
            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                    time_remaining -= 1;
                }
                _ = turn_advance_notify.notified() => {
                    word_submitted = true;
                    break;
                }
            }
        }

        if word_submitted {
            // Player submitted a valid word (WordEntry was broadcast)
            // Advance to next turn and next rule
            let mut inner_guard = inner.write().await;
            inner_guard.turn_rotation.next_turn();
            inner_guard.advance_rule();
        } else {
            // Timeout - eliminate current player with Eliminated event
            if let Some(player_id) = current_player_id {
                let mut inner_guard = inner.write().await;
                inner_guard
                    .eliminate_player(player_id, "Time ran out!")
                    .await;

                // Move to next player if game continues
                if inner_guard.turn_rotation.active_count() > 1 {
                    inner_guard.turn_rotation.next_turn();
                    inner_guard.advance_rule();
                }
            }
        }
    }
}

// ============================================================================
// Factory
// ============================================================================

/// Factory function to create new LexiWars game instances
pub fn create_lexi_wars(lobby_id: Uuid) -> Box<dyn GameEngine> {
    Box::new(LexiWarsEngine::new(lobby_id))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prize_calculation() {
        let engine = LexiWarsEngine::new(Uuid::new_v4());
        {
            let mut inner = engine.inner.write().await;
            inner.current_amount = Some(100.0);
            inner.total_players = 3;

            // 3 players
            assert_eq!(inner.calculate_prize(1, 3), Some(50.0)); // 50%
            assert_eq!(inner.calculate_prize(2, 3), Some(30.0)); // 30%
            assert_eq!(inner.calculate_prize(3, 3), Some(20.0)); // 20%

            // 2 players
            assert_eq!(inner.calculate_prize(1, 2), Some(70.0)); // 70%
            assert_eq!(inner.calculate_prize(2, 2), Some(30.0)); // 30%
        }
    }

    #[tokio::test]
    async fn test_wars_point_calculation() {
        let user_id = Uuid::new_v4();
        let engine = LexiWarsEngine::new(Uuid::new_v4());
        {
            let mut inner = engine.inner.write().await;
            inner.entry_amount = Some(10.0);
            inner.current_amount = Some(30.0);
            inner.is_sponsored = false;

            // Base points: (participants - rank + 1) * 2
            // For 3 participants, rank 1: (3 - 1 + 1) * 2 = 6
            let points = inner.calculate_wars_point(user_id, 1, 3);
            assert!(points >= 6.0);
            assert!(points <= 50.0); // Cap
        }
    }
}
