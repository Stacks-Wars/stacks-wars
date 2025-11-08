//! # Lexi Wars Game State
//!
//! Game-specific state for Lexi Wars word game.
//!
//! ## Storage
//! - **Redis Key**: `lobbies:{lobby_id}:game_state`
//! - **Format**: JSON blob
//!
//! ## Data Stored
//! - Words used in the game
//! - Current rule and context
//! - Turn management
//! - Eliminated players
//!
//! ## Migration from Old Architecture
//!
//! ### Old Keys (scattered):
//! ```
//! lobbies:{id}:used_words         → Set
//! lobbies:{id}:current_rule       → String
//! lobbies:{id}:rule_context       → String
//! lobbies:{id}:rule_index         → Int
//! lobbies:{id}:current_turn       → UUID
//! lobbies:{id}:eliminated_players → Set
//! ```
//!
//! ### New Key (consolidated):
//! ```
//! lobbies:{id}:game_state → JSON:
//! {
//!   "usedWords": ["word1", "word2"],
//!   "currentRule": "starts_with_a",
//!   "ruleContext": "a",
//!   "ruleIndex": 0,
//!   "currentTurn": "uuid",
//!   "eliminatedPlayers": ["uuid1", "uuid2"]
//! }
//! ```

use crate::errors::AppError;
use crate::games::GameState;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lexi Wars specific game state
///
/// This contains ALL game-specific data for Lexi Wars.
/// Platform-generic data (player rank, prize) is in PlayerState.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LexiWarsGameState {
    /// Words used so far in the game (prevents repeats)
    pub used_words: Vec<String>,

    /// Current game rule (e.g., "starts_with_a", "contains_x", "ends_with_ing")
    pub current_rule: Option<String>,

    /// Rule context (e.g., the letter "a" for "starts_with_a" rule)
    pub rule_context: Option<String>,

    /// Current rule index (for cycling through rules)
    pub rule_index: usize,

    /// Current turn player ID
    pub current_turn: Option<Uuid>,

    /// List of eliminated player IDs
    pub eliminated_players: Vec<Uuid>,
}

impl GameState for LexiWarsGameState {
    /// Initialize new Lexi Wars game state
    fn initialize() -> Self {
        Self {
            used_words: Vec::new(),
            current_rule: None,
            rule_context: None,
            rule_index: 0,
            current_turn: None,
            eliminated_players: Vec::new(),
        }
    }

    /// Validate Lexi Wars game state
    fn validate(&self) -> Result<(), AppError> {
        // Check that rule and context are either both present or both absent
        match (&self.current_rule, &self.rule_context) {
            (Some(_), None) | (None, Some(_)) => {
                return Err(AppError::InvalidInput(
                    "Rule and context must both be present or both be absent".into(),
                ));
            }
            _ => {}
        }

        // Validate used words are not empty strings
        for word in &self.used_words {
            if word.trim().is_empty() {
                return Err(AppError::InvalidInput(
                    "Used words cannot be empty strings".into(),
                ));
            }
        }

        Ok(())
    }

    fn summary(&self) -> String {
        format!(
            "LexiWars - {} words used, {} players eliminated, rule: {:?}",
            self.used_words.len(),
            self.eliminated_players.len(),
            self.current_rule
        )
    }
}

impl LexiWarsGameState {
    /// Add a word to the used words list
    pub fn add_used_word(&mut self, word: String) {
        if !self.used_words.contains(&word) {
            self.used_words.push(word.to_lowercase());
        }
    }

    /// Check if a word has been used
    pub fn is_word_used(&self, word: &str) -> bool {
        self.used_words.contains(&word.to_lowercase())
    }

    /// Set the current rule and context
    pub fn set_rule(&mut self, rule: String, context: String) {
        self.current_rule = Some(rule);
        self.rule_context = Some(context);
    }

    /// Clear the current rule
    pub fn clear_rule(&mut self) {
        self.current_rule = None;
        self.rule_context = None;
    }

    /// Set the current turn to a specific player
    pub fn set_current_turn(&mut self, player_id: Uuid) {
        self.current_turn = Some(player_id);
    }

    /// Clear the current turn
    pub fn clear_current_turn(&mut self) {
        self.current_turn = None;
    }

    /// Eliminate a player
    pub fn eliminate_player(&mut self, player_id: Uuid) {
        if !self.eliminated_players.contains(&player_id) {
            self.eliminated_players.push(player_id);
        }
    }

    /// Check if a player is eliminated
    pub fn is_player_eliminated(&self, player_id: &Uuid) -> bool {
        self.eliminated_players.contains(player_id)
    }

    /// Get the number of active (non-eliminated) players from a total
    pub fn active_players_count(&self, total_players: usize) -> usize {
        total_players.saturating_sub(self.eliminated_players.len())
    }

    /// Reset the game state for a new round
    pub fn reset(&mut self) {
        self.used_words.clear();
        self.current_rule = None;
        self.rule_context = None;
        self.rule_index = 0;
        self.current_turn = None;
        self.eliminated_players.clear();
    }

    /// Get the count of words used
    pub fn words_count(&self) -> usize {
        self.used_words.len()
    }

    /// Advance to the next rule index
    pub fn next_rule_index(&mut self, max_rules: usize) {
        self.rule_index = (self.rule_index + 1) % max_rules;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() {
        let state = LexiWarsGameState::initialize();

        assert_eq!(state.used_words.len(), 0);
        assert_eq!(state.eliminated_players.len(), 0);
        assert!(state.current_rule.is_none());
        assert!(state.current_turn.is_none());
    }

    #[test]
    fn test_add_used_word() {
        let mut state = LexiWarsGameState::initialize();

        state.add_used_word("hello".to_string());
        assert_eq!(state.used_words.len(), 1);
        assert!(state.is_word_used("hello"));

        // Adding duplicate should not increase count
        state.add_used_word("hello".to_string());
        assert_eq!(state.used_words.len(), 1);
    }

    #[test]
    fn test_is_word_used() {
        let mut state = LexiWarsGameState::initialize();

        state.add_used_word("HELLO".to_string());
        assert!(state.is_word_used("hello")); // Case insensitive
        assert!(state.is_word_used("HELLO"));
        assert!(!state.is_word_used("world"));
    }

    #[test]
    fn test_set_rule() {
        let mut state = LexiWarsGameState::initialize();

        state.set_rule("starts_with_a".to_string(), "a".to_string());
        assert_eq!(state.current_rule, Some("starts_with_a".to_string()));
        assert_eq!(state.rule_context, Some("a".to_string()));
    }

    #[test]
    fn test_eliminate_player() {
        let mut state = LexiWarsGameState::initialize();
        let player_id = Uuid::new_v4();

        state.eliminate_player(player_id);
        assert!(state.is_player_eliminated(&player_id));
        assert_eq!(state.eliminated_players.len(), 1);

        // Eliminating again should not duplicate
        state.eliminate_player(player_id);
        assert_eq!(state.eliminated_players.len(), 1);
    }

    #[test]
    fn test_active_players_count() {
        let mut state = LexiWarsGameState::initialize();

        assert_eq!(state.active_players_count(5), 5);

        state.eliminate_player(Uuid::new_v4());
        assert_eq!(state.active_players_count(5), 4);

        state.eliminate_player(Uuid::new_v4());
        assert_eq!(state.active_players_count(5), 3);
    }

    #[test]
    fn test_validate() {
        let mut state = LexiWarsGameState::initialize();

        // Valid: both None
        assert!(state.validate().is_ok());

        // Valid: both Some
        state.set_rule("test".to_string(), "context".to_string());
        assert!(state.validate().is_ok());

        // Invalid: rule without context
        state.current_rule = Some("test".to_string());
        state.rule_context = None;
        assert!(state.validate().is_err());
    }

    #[test]
    fn test_json_serialization() {
        let state = LexiWarsGameState::initialize();

        let json = state.to_json().unwrap();
        let deserialized = LexiWarsGameState::from_json(&json).unwrap();

        assert_eq!(state.used_words, deserialized.used_words);
        assert_eq!(state.current_rule, deserialized.current_rule);
    }

    #[test]
    fn test_reset() {
        let mut state = LexiWarsGameState::initialize();

        state.add_used_word("hello".to_string());
        state.set_rule("test".to_string(), "context".to_string());
        state.eliminate_player(Uuid::new_v4());
        state.rule_index = 5;

        state.reset();

        assert_eq!(state.used_words.len(), 0);
        assert_eq!(state.eliminated_players.len(), 0);
        assert!(state.current_rule.is_none());
        assert_eq!(state.rule_index, 0);
    }
}
