# Database Module

Repository pattern implementation for PostgreSQL and Redis operations.

## Structure

```
db/
├── mod.rs                    # Exports all repositories
├── hydration/                # Redis hydration logic
│   ├── mod.rs
│   ├── redis.rs              # Redis hydration functions
│   ├── types.rs              # Hydration types
│   └── redis/
│       ├── lobby_states.rs   # Lobby state hydration
│       └── player_states.rs  # Player state hydration
│
└── {entity}/                 # One folder per entity
    ├── mod.rs                # Repository struct + constructor
    ├── create.rs             # INSERT operations
    ├── read.rs               # SELECT operations
    ├── update.rs             # UPDATE operations
    └── delete.rs             # DELETE operations
```

## Repository Pattern

Each entity follows a consistent pattern:

### mod.rs - Repository Definition

```rust
use sqlx::PgPool;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

### create.rs - Insert Operations

```rust
impl UserRepository {
    pub async fn create(&self, user: &NewUser) -> Result<User, AppError> {
        sqlx::query_as!(...)
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::from)
    }
}
```

### read.rs - Select Operations

```rust
impl UserRepository {
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        sqlx::query_as!(...)
            .fetch_optional(&self.pool)
            .await
            .map_err(AppError::from)
    }
}
```

## Entities

| Repository                 | Table              | Purpose              |
| -------------------------- | ------------------ | -------------------- |
| `UserRepository`           | `users`            | User accounts        |
| `GameRepository`           | `games`            | Game definitions     |
| `LobbyRepository`          | `lobbies`          | Game lobbies         |
| `LobbyStateRepository`     | Redis              | Runtime lobby state  |
| `PlayerStateRepository`    | Redis              | Runtime player state |
| `LobbyChatRepository`      | Redis              | Chat messages        |
| `JoinRequestRepository`    | `join_requests`    | Lobby join requests  |
| `SeasonRepository`         | `seasons`          | Competitive seasons  |
| `PlatformRatingRepository` | `platform_ratings` | Player ratings       |
| `UserWarsPointsRepository` | `user_wars_points` | Wars points ledger   |

## Broadcasting Pattern

Repository methods that modify state accept `Option<AppState>` for broadcasting:

```rust
impl LobbyRepository {
    pub async fn update_status(
        &self,
        lobby_id: Uuid,
        status: LobbyStatus,
        state: Option<&AppState>,  // Pass None for migrations, Some for live
    ) -> Result<(), AppError> {
        // Update database
        sqlx::query!(...)
            .execute(&self.pool)
            .await?;

        // Broadcast if state provided
        if let Some(state) = state {
            broadcast_lobby_update(state, lobby_id).await;
        }

        Ok(())
    }
}
```

## Redis Keys

All Redis keys are built using `RedisKey` from `src/models/keys.rs`:

```rust
use crate::models::keys::RedisKey;

// Get key for lobby state
let key = RedisKey::lobby_state(lobby_id);  // "lobbies:{id}:state"

// Get key with wildcard for scanning
let pattern = RedisKey::lobby_player(lobby_id, KeyPart::Wildcard);
```

## Adding a New Entity

1. Create migration:

```sql
-- migrations/YYYYMMDDHHMMSS_add_entity.up.sql
CREATE TABLE entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

2. Create model in `src/models/entity.rs`:

```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Entity {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
```

3. Create repository folder `src/db/entity/`:
    - `mod.rs` - Repository struct
    - `create.rs` - Insert operations
    - `read.rs` - Select operations
    - `update.rs` - Update operations
    - `delete.rs` - Delete operations

4. Export from `src/db/mod.rs`:

```rust
pub mod entity;
pub use entity::EntityRepository;
```
