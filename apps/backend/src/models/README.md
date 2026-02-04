# Models Module

Domain models, DTOs, and Redis key builders.

## Structure

```
models/
├── mod.rs              # Module exports
├── keys.rs             # Redis key builders
│
├── user.rs             # User model
├── game.rs             # Game definition model
├── lobby.rs            # Lobby model
├── lobby_state.rs      # Runtime lobby state (Redis)
├── player_state.rs     # Runtime player state (Redis)
├── chat_message.rs     # Chat message model
│
├── season.rs           # Competitive season
├── platform_rating.rs  # Player ratings
├── user_wars_point.rs  # Wars points ledger
│
├── username.rs         # Username validation
├── wallet_address.rs   # Wallet address validation
├── stacks.rs           # Blockchain types
└── bot.rs              # Telegram bot types
```

## Model Types

### Database Models

Mapped directly from PostgreSQL tables using `sqlx::FromRow`:

```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub wallet_address: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

### Redis Models

Serialized to/from JSON for Redis storage:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LobbyState {
    pub status: LobbyStatus,
    pub players: Vec<Uuid>,
    pub ready_players: HashSet<Uuid>,
}
```

### DTOs (Data Transfer Objects)

Used for API requests/responses:

```rust
#[derive(Debug, Serialize)]
pub struct LobbyInfo {
    pub id: Uuid,
    pub game: GameInfo,
    pub host: UserInfo,
    pub player_count: u8,
}
```

## Redis Keys

The `keys.rs` file provides type-safe Redis key construction:

```rust
pub struct RedisKey;

impl RedisKey {
    /// lobbies:{lobby_id}:state
    pub fn lobby_state(lobby_id: Uuid) -> String {
        format!("lobbies:{}:state", lobby_id)
    }

    /// lobbies:{lobby_id}:players:{user_id}
    pub fn lobby_player(lobby_id: Uuid, user_id: impl KeyPart) -> String {
        format!("lobbies:{}:players:{}", lobby_id, user_id.as_key_part())
    }

    /// lobbies:{lobby_id}:chat
    pub fn lobby_chat(lobby_id: Uuid) -> String {
        format!("lobbies:{}:chat", lobby_id)
    }
}
```

### KeyPart Trait

Allows flexible key construction with wildcards:

```rust
pub enum KeyPart {
    Uuid(Uuid),
    Wildcard,
}

// Usage
RedisKey::lobby_player(lobby_id, user_id)        // Specific player
RedisKey::lobby_player(lobby_id, KeyPart::Wildcard)  // Pattern for SCAN
```

## Validation Types

Custom types with built-in validation:

### Username

```rust
pub struct Username(String);

impl Username {
    pub fn new(value: &str) -> Result<Self, ValidationError> {
        // Validate length, characters, etc.
    }
}
```

### WalletAddress

```rust
pub struct WalletAddress(String);

impl WalletAddress {
    pub fn new(value: &str) -> Result<Self, ValidationError> {
        // Validate Stacks address format
    }
}
```

## Adding a New Model

1. Create file `src/models/my_model.rs`:

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct MyModel {
    pub id: Uuid,
    pub name: String,
}
```

2. Export from `src/models/mod.rs`:

```rust
pub mod my_model;
pub use my_model::MyModel;
```

3. If Redis storage needed, add keys to `keys.rs`:

```rust
impl RedisKey {
    pub fn my_model(id: Uuid) -> String {
        format!("my_models:{}", id)
    }
}
```
