# Games Module

Shipped game engines, the in-server registry, and integration helpers. **Portable traits and wire types** live in the `stacks_wars_core` crate (published on crates.io as `stacks_wars_core`). **Each first-party game** is a normal Rust crate in its own repository (for example `Stacks-Wars/checkers`); the server depends on those crates and re-exports them from `server/src/games/mod.rs` so `crate::games::lexi_wars` and similar paths keep working.

## Layout

```text
apps/backend/
  core/                 # stacks_wars_core — GameEngine, GameHost, DTOs
  server/               # stacks_wars_server — HTTP/WS/DB, KernelGameHost
  migrations/

# Sibling directories (monorepo root next to the `stacks-wars` checkout):
  checkers/             # stacks_wars_checkers
  lexi-wars/            # stacks_wars_lexi_wars
  ludo/                 # stacks_wars_ludo
  ludo-rush/            # stacks_wars_ludo_rush
```

**Dependency rule:** each game crate depends only on `stacks_wars_core` (from crates.io). The server depends on `stacks_wars_core` and every registered game crate by **version** from crates.io (`server/Cargo.toml`).

**Publishing:** from `apps/backend`, run `cargo publish -p stacks_wars_core`; then publish each `stacks_wars_<game>` crate from its own repo. Game crates must not depend on anything Stacks-Wars–specific besides `stacks_wars_core`.

## `GameHost` boundary

Engines do **not** take `AppState`. They receive `GameHostRef` (`Arc<dyn GameHost>`) from the room spawn path and call:

- `broadcast_game_message`, `broadcast_game_message_to_user`, `broadcast_game_message_to_room_except` for JSON payloads wrapped as `GameMessage` on the server.
- `broadcast_room_game` / `broadcast_user_room` for shared room messages (`GameRoomBroadcast`, `UserRoomMessage` in `core`, mapped to `RoomServerMessage` in `server/src/game_host.rs`).
- `save_player_result`, `finish_lobby`, `get_player_states_in_lobby` for persistence and Redis-backed player rows.

The concrete implementation is `KernelGameHost(AppState)` in `server/src/game_host.rs`. `kernel_app_state` exists for any code that must recover `AppState` from a kernel host reference.

## In-server `games/` tree

```
server/src/games/
├── mod.rs              # Re-exports core traits + wraps external game crates
├── registry.rs         # Game registry (ID → GameFactory)
├── common.rs           # Platform integration (save/finish; re-exports from core)
├── error.rs            # Game-specific errors
└── README.md           # This file
```

## `GameEngine` (see `stacks_wars_core`)

Refer to `stacks_wars_core::GameEngine` for the authoritative async trait (`initialize`, `handle_action`, `get_game_state`, `start_loop` with `GameHostRef`, etc.).

---

# How to Add a New Game

This is a complete walkthrough for adding a new game to Stacks Wars.

## Step 1: Plan Your Game

Before writing code, define:

1. **Player count**: Min/max players (e.g., 2-4)
2. **Turn structure**: Real-time, turn-based, or hybrid
3. **Win condition**: How does the game end?
4. **Actions**: What can players do?
5. **Events**: What does the server broadcast?

## Step 2: Create a standalone `stacks_wars_<game>` crate

Create a new repository (or crate) named on crates.io as `stacks_wars_<game>` (for example `stacks_wars_chess`). The `Cargo.toml` should depend on `stacks_wars_core` from crates.io (plus `async-trait`, `serde`, `tokio`, etc. as needed). Export `pub const MY_GAME_ID: Uuid` and `pub fn create_my_game(lobby_id: Uuid, host: GameHostRef) -> Box<dyn GameEngine>`.

Publish the crate, then wire it into this server: add `stacks_wars_chess = "0.1.0"` (or your version) to `server/Cargo.toml`, add `pub mod chess { pub use stacks_wars_chess::*; }` in `server/src/games/mod.rs`, and register the factory in `registry.rs`.

## Step 3: Define Messages

Create `games/my_game/src/message.rs`:

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Client → Server Actions
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum MyGameAction {
    /// Example: Player makes a move
    MakeMove { position: u8 },

    /// Example: Player uses an item
    UseItem { item_id: Uuid },

    /// Example: Player skips turn
    SkipTurn,
}

// ============================================================================
// Server → Client Events
// ============================================================================

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum MyGameEvent {
    /// Game state update (board, scores, etc.)
    StateUpdate {
        board: Vec<u8>,
        scores: Vec<(Uuid, i32)>,
    },

    /// It's a player's turn
    Turn {
        player_id: Uuid,
        time_limit: u64,
    },

    /// A player made a move
    MoveMade {
        player_id: Uuid,
        position: u8,
    },

    /// Countdown tick
    Countdown { seconds: u64 },

    /// Invalid action attempted
    Invalid { reason: String },
}
```

### Message Naming Conventions

- **Actions** (client → server): Verb phrases (`MakeMove`, `UseItem`, `SkipTurn`)
- **Events** (server → client): Past tense or state (`MoveMade`, `Turn`, `StateUpdate`)

## Step 4: Create the Engine

Create `src/games/my_game/engine.rs`:

```rust
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    db::player_state::save_player_result,
    games::{
        common::{GameResults, PlayerRanking, TurnRotation, WarsPointContext},
        error::GameError,
        GameEngine,
    },
    models::{lobby::finish_lobby, PlayerState},
    state::AppState,
    ws::broadcast,
    ws::room::messages::RoomServerMessage,
};

use super::message::{MyGameAction, MyGameEvent};

// ============================================================================
// Constants
// ============================================================================

const TURN_TIME_SECONDS: u64 = 30;
const COUNTDOWN_INTERVAL_SECONDS: u64 = 5;

// ============================================================================
// Game State
// ============================================================================

/// Internal mutable state
struct MyGameInner {
    turn_rotation: TurnRotation,
    board: Vec<u8>,
    scores: std::collections::HashMap<Uuid, i32>,
    player_states: std::collections::HashMap<Uuid, PlayerState>,
    results: Option<GameResults>,
    game_over: bool,
}

/// Thread-safe game engine
pub struct MyGameEngine {
    inner: Arc<RwLock<MyGameInner>>,
}

impl MyGameEngine {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MyGameInner {
                turn_rotation: TurnRotation::new(vec![]),
                board: vec![0; 64], // Example: 8x8 board
                scores: std::collections::HashMap::new(),
                player_states: std::collections::HashMap::new(),
                results: None,
                game_over: false,
            })),
        }
    }
}

// ============================================================================
// Factory Function
// ============================================================================

/// Factory function for the game registry
pub fn create_my_game() -> Box<dyn GameEngine> {
    Box::new(MyGameEngine::new())
}

// ============================================================================
// GameEngine Implementation
// ============================================================================

#[async_trait]
impl GameEngine for MyGameEngine {
    async fn initialize(&self, players: Vec<Uuid>) -> Result<(), GameError> {
        let mut inner = self.inner.write().await;

        // Initialize turn rotation
        inner.turn_rotation = TurnRotation::new(players.clone());

        // Initialize scores
        for player_id in &players {
            inner.scores.insert(*player_id, 0);
        }

        Ok(())
    }

    async fn handle_action(
        &self,
        player_id: Uuid,
        action: Value,
    ) -> Result<(), GameError> {
        // Parse action
        let action: MyGameAction = serde_json::from_value(action)
            .map_err(|e| GameError::InvalidAction(e.to_string()))?;

        let mut inner = self.inner.write().await;

        // Verify it's player's turn
        if inner.turn_rotation.current_player() != Some(player_id) {
            return Err(GameError::NotYourTurn);
        }

        match action {
            MyGameAction::MakeMove { position } => {
                // Validate move
                if position as usize >= inner.board.len() {
                    return Err(GameError::InvalidAction("Invalid position".into()));
                }

                // Apply move
                inner.board[position as usize] = 1;

                // Update score
                if let Some(score) = inner.scores.get_mut(&player_id) {
                    *score += 10;
                }

                // Advance turn
                inner.turn_rotation.next_turn();
            }
            MyGameAction::UseItem { .. } => {
                // Handle item usage
            }
            MyGameAction::SkipTurn => {
                inner.turn_rotation.next_turn();
            }
        }

        Ok(())
    }

    async fn get_game_state(&self) -> Value {
        let inner = self.inner.read().await;

        json!({
            "board": inner.board,
            "scores": inner.scores,
            "currentPlayer": inner.turn_rotation.current_player(),
        })
    }

    async fn start_loop(&self, state: AppState, lobby_id: Uuid) {
        let inner = self.inner.clone();

        tokio::spawn(async move {
            game_loop(inner, state, lobby_id).await;
        });
    }
}

// ============================================================================
// Game Loop
// ============================================================================

async fn game_loop(
    inner: Arc<RwLock<MyGameInner>>,
    state: AppState,
    lobby_id: Uuid,
) {
    loop {
        // Check if game is over
        {
            let guard = inner.read().await;
            if guard.game_over {
                break;
            }
        }

        // Get current player
        let current_player = {
            let guard = inner.read().await;
            guard.turn_rotation.current_player()
        };

        let Some(player_id) = current_player else {
            break;
        };

        // Broadcast turn start
        let turn_event = MyGameEvent::Turn {
            player_id,
            time_limit: TURN_TIME_SECONDS,
        };
        broadcast::broadcast_game_message(&state, lobby_id, &turn_event).await;

        // Countdown loop
        let mut remaining = TURN_TIME_SECONDS;
        while remaining > 0 {
            tokio::time::sleep(tokio::time::Duration::from_secs(
                COUNTDOWN_INTERVAL_SECONDS.min(remaining)
            )).await;

            remaining = remaining.saturating_sub(COUNTDOWN_INTERVAL_SECONDS);

            // Broadcast countdown
            let countdown = MyGameEvent::Countdown { seconds: remaining };
            broadcast::broadcast_game_message(&state, lobby_id, &countdown).await;

            // Check if turn was taken
            let guard = inner.read().await;
            if guard.turn_rotation.current_player() != Some(player_id) {
                break; // Player took their turn
            }
        }

        // If time ran out, force skip
        {
            let mut guard = inner.write().await;
            if guard.turn_rotation.current_player() == Some(player_id) {
                guard.turn_rotation.next_turn();
            }
        }

        // Check win condition
        if check_game_over(&inner).await {
            end_game(inner, state, lobby_id).await;
            break;
        }
    }
}

async fn check_game_over(inner: &Arc<RwLock<MyGameInner>>) -> bool {
    let guard = inner.read().await;

    // Example: Game over when someone reaches 100 points
    guard.scores.values().any(|&score| score >= 100)
}

async fn end_game(
    inner: Arc<RwLock<MyGameInner>>,
    state: AppState,
    lobby_id: Uuid,
) {
    let mut guard = inner.write().await;
    guard.game_over = true;

    // Sort players by score (descending)
    let mut rankings: Vec<_> = guard.scores.iter().collect();
    rankings.sort_by(|a, b| b.1.cmp(a.1));

    // Get lobby info for prize calculation
    let lobby = state.lobby_repo().find_by_id(lobby_id).await.ok().flatten();
    let entry_fee = lobby.as_ref().and_then(|l| l.entry_fee).unwrap_or(0.0);
    let participant_count = rankings.len();
    let total_pool = entry_fee * participant_count as f64;

    let mut player_rankings = Vec::new();
    let mut final_standings = Vec::new();

    for (rank, (user_id, score)) in rankings.iter().enumerate() {
        let rank = rank + 1;

        // Calculate prize (example: 70% to 1st, 30% to 2nd)
        let prize = match rank {
            1 => Some(total_pool * 0.7),
            2 if participant_count > 2 => Some(total_pool * 0.3),
            _ => None,
        };

        // Calculate wars points
        let wars_point = WarsPointContext {
            rank,
            participants: participant_count,
            entry_fee,
        }.calculate();

        // Save to database
        if let Err(e) = save_player_result(
            &state,
            lobby_id,
            **user_id,
            rank,
            prize,
            wars_point,
        ).await {
            tracing::error!("Failed to save player result: {}", e);
        }

        player_rankings.push(PlayerRanking {
            user_id: **user_id,
            rank,
            score: Some(**score),
            prize,
        });

        // Send individual GameOver message
        let game_over = RoomServerMessage::GameOver {
            rank,
            prize,
            wars_point,
        };
        broadcast::broadcast_user(&state, **user_id, &game_over).await;
    }

    // Broadcast final standings
    let final_standing = RoomServerMessage::FinalStanding {
        standings: final_standings,
    };
    broadcast::broadcast_room(&state, lobby_id, &final_standing).await;

    // Mark lobby as finished
    if let Err(e) = finish_lobby(&state, lobby_id).await {
        tracing::error!("Failed to finish lobby: {}", e);
    }

    // Store results
    guard.results = Some(GameResults {
        rankings: player_rankings,
        finished_at: chrono::Utc::now().timestamp(),
        metadata: None,
    });
}
```

## Step 5: Create Module File

Create `src/games/my_game/mod.rs`:

```rust
//! My Game Module
//!
//! A brief description of the game.

pub mod engine;
pub mod message;

pub use engine::create_my_game;
```

## Step 6: Register the Game

### Add Game ID

In `src/games/registry.rs`:

```rust
use crate::games::{
    GameFactory,
    lexi_wars::create_lexi_wars,
    ludo::create_ludo,
    my_game::create_my_game,  // Add import
};

// Game IDs - generate a new UUID for your game
pub const LEXI_WARS_GAME_ID: Uuid = uuid::uuid!("5eb61ff4-8f9f-48bd-ae61-dfd6d95052eb");
pub const LUDO_GAME_ID: Uuid = uuid::uuid!("a1b2c3d4-e5f6-7890-abcd-ef1234567890");
pub const MY_GAME_ID: Uuid = uuid::uuid!("YOUR-NEW-UUID-HERE");  // Generate new UUID
```

### Register Factory

In `src/games/registry.rs`:

```rust
pub fn create_game_registry() -> HashMap<Uuid, GameFactory> {
    let mut registry = HashMap::new();

    registry.insert(LEXI_WARS_GAME_ID, create_lexi_wars as GameFactory);
    registry.insert(LUDO_GAME_ID, create_ludo as GameFactory);
    registry.insert(MY_GAME_ID, create_my_game as GameFactory);  // Add your game

    registry
}
```

### Export Module

In `src/games/mod.rs`:

```rust
pub mod common;
pub mod error;
pub mod lexi_wars;
pub mod ludo;
pub mod my_game;  // Add module
pub mod registry;

pub use common::*;
pub use error::GameError;
pub use registry::{LEXI_WARS_GAME_ID, LUDO_GAME_ID, MY_GAME_ID, create_game_registry};
```

## Step 7: Add to Database

Insert your game into the `games` table:

- Current can be done from your Stacks wars profile.

## Step 8: Test Your Game

Create tests in `src/games/my_game/engine.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialization() {
        let engine = MyGameEngine::new();
        let players = vec![Uuid::new_v4(), Uuid::new_v4()];

        engine.initialize(players.clone()).await.unwrap();

        let state = engine.get_game_state().await;
        assert_eq!(state["scores"].as_object().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_make_move() {
        let engine = MyGameEngine::new();
        let players = vec![Uuid::new_v4(), Uuid::new_v4()];

        engine.initialize(players.clone()).await.unwrap();

        let action = json!({ "action": "makeMove", "position": 0 });
        engine.handle_action(players[0], action).await.unwrap();

        let state = engine.get_game_state().await;
        assert_eq!(state["board"][0], 1);
    }
}
```

Run tests:

```bash
cargo test --lib my_game
```

---

## Common Patterns

### Turn Rotation

Use `TurnRotation` from `common.rs`:

```rust
use crate::games::common::TurnRotation;

let mut rotation = TurnRotation::new(player_ids);
let current = rotation.current_player();  // Option<Uuid>
rotation.next_turn();                       // Advance to next player
rotation.eliminate_player(player_id);       // Remove from rotation
```

### Broadcasting

Always use centralized broadcast helpers:

```rust
use crate::ws::broadcast;

// Game event to all players in lobby
broadcast::broadcast_game_message(&state, lobby_id, &event).await;

// Room message (player joined, etc.)
broadcast::broadcast_room(&state, lobby_id, &message).await;

// Direct message to one user
broadcast::broadcast_user(&state, user_id, &message).await;
```

### Prize Calculation

Use `WarsPointContext` for consistent point calculation:

```rust
use crate::games::common::WarsPointContext;

let context = WarsPointContext {
    rank,
    participants: player_count,
    entry_fee,
};
let wars_point = context.calculate();
```

### Saving Results

Always save results and finish lobby:

```rust
use crate::db::player_state::save_player_result;
use crate::models::lobby::finish_lobby;

// Save each player's result
save_player_result(&state, lobby_id, user_id, rank, prize, wars_point).await?;

// Mark lobby as finished
finish_lobby(&state, lobby_id).await?;
```

---

## Checklist

- [ ] Created `src/games/my_game/` folder
- [ ] Defined actions and events in `message.rs`
- [ ] Implemented `GameEngine` trait in `engine.rs`
- [ ] Created factory function `create_my_game()`
- [ ] Exported from `mod.rs`
- [ ] Added game ID constant to `registry.rs`
- [ ] Registered in `create_game_registry()`
- [ ] Added module export to `src/games/mod.rs`
- [ ] Inserted game into database
- [ ] Written tests
- [ ] Verified compilation: `cargo check`
- [ ] Run tests: `cargo test --lib my_game`
