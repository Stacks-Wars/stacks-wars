//! Lexi Wars game crate.

pub mod engine;
pub mod message;
pub mod rule;

pub use engine::{
    create_lexi_wars, LexiWarsEngine, INITIAL_MIN_WORD_LENGTH, TURN_TIMEOUT_SECS,
    WORD_LENGTH_INCREMENT,
};
pub use message::{LexiWarsAction, LexiWarsEvent};
pub use rule::{get_rule_at_index, lexi_wars_rules, rule_count, ClientRule, Rule, RuleContext};

pub const LEXI_WARS_GAME_ID: uuid::Uuid = uuid::Uuid::from_bytes([
    94, 182, 31, 244, 143, 159, 72, 189, 174, 97, 223, 214, 217, 80, 82, 235,
]);
