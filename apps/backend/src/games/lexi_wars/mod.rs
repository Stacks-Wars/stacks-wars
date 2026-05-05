// Lexi Wars Game Module
//
// A turn-based word game where players must submit valid words following rules.
// Rules cycle sequentially; after all rules used, min word length increases by 2.
//
// Module structure:
// - engine.rs: Core game logic (LexiWarsEngine, game loop, prize calculation)
// - message.rs: Game-specific message types (LexiWarsAction, LexiWarsEvent)
// - rule.rs: Rule definitions and validation logic
//
// Shared game events (GameStarted, GameStartFailed, FinalStanding, GameOver) are in
// ws/room/messages.rs as RoomServerMessage variants.
//
// Flow:
// 1. UpdateLobbyStatus::Starting triggers countdown in ws/room/engine.rs
// 2. After countdown, engine.rs calls game.initialize() → broadcasts GameStarted
// 3. initialize() returns events; start_loop() spawns the game loop task
// 4. Game loop: Turn → Rule (to current player) → Countdown (15s)
// 5. On SubmitWord action: validate → WordEntry (room) or Invalid/UsedWord (user)
// 6. Valid word signals turn advance via notify channel
// 7. Timeout → Eliminated + GameOver (to user) → next turn or FinalStanding if 1 player left

pub mod engine;
pub mod message;
pub mod rule;

// Re-export engine types
pub use engine::{
    create_lexi_wars, LexiWarsEngine, INITIAL_MIN_WORD_LENGTH, TURN_TIMEOUT_SECS,
    WORD_LENGTH_INCREMENT,
};

// Re-export message types
pub use message::{LexiWarsAction, LexiWarsEvent};

// Re-export rule types
pub use rule::{get_rule_at_index, lexi_wars_rules, rule_count, ClientRule, Rule, RuleContext};
