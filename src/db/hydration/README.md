# PostgreSQL Hydration Guide

## Overview

Since PostgreSQL was added to the architecture after Redis, we need to populate (hydrate) PostgreSQL with existing data from Redis. This is a one-time migration.

## What Gets Hydrated

### 1. Users Table

**Source:** `users:{user_id}` (Redis hash)
**Destination:** `users` table (PostgreSQL)

**Fields migrated:**

-   `id` → `id`
-   `username` → `username`
-   `stx_address` → `wallet_address`
-   `telegram_id` → `telegram_id`
-   Default `trust_rating` = 10.0

### 2. Lobbies Table

**Source:** `lobbies:{lobby_id}:info` (Redis hash)
**Destination:** `lobbies` table (PostgreSQL)

**Fields migrated:**

-   `id` → `id`
-   `name` → `name`
-   `description` → `description`
-   `creator_id` → `creator_id`
-   `game_id` → `game_id`
-   `entry_amount` → `entry_amount`
-   `max_participants` → `max_participants`
-   `is_private` → `is_private`
-   `is_sponsored` → `is_sponsored`
-   Default `status` = "Waiting"

## How to Run

### Prerequisites

1. PostgreSQL database created and migrations run
2. Redis instance with existing data
3. `.env` file configured with both DATABASE_URL and REDIS_URL

### Execution

```bash
# Run the hydration script
cargo run --bin hydrate
```

### What Happens

1. **Connects to Redis and PostgreSQL**
2. **Scans Redis for all user keys** (`users:*`)
3. **Inserts users into PostgreSQL** (uses ON CONFLICT DO NOTHING for idempotency)
4. **Scans Redis for all lobby keys** (`lobbies:*:info`)
5. **Inserts lobbies into PostgreSQL** (uses ON CONFLICT DO NOTHING for idempotency)
6. **Reports statistics**

### Output Example

```
🚀 Initializing application state...
✅ Connected to PostgreSQL and Redis

╔═══════════════════════════════════════════════╗
║  Starting PostgreSQL Hydration from Redis   ║
╚═══════════════════════════════════════════════╝

📊 Phase 1: Hydrating users table...
✅ Hydrated user: alice123 (550e8400-e29b-41d4-a716-446655440000)
✅ Hydrated user: bob456 (550e8400-e29b-41d4-a716-446655440001)
  User 550e8400-e29b-41d4-a716-446655440002 already exists, skipping
   3 users migrated

📊 Phase 2: Hydrating lobbies table...
✅ Hydrated lobby: Epic Battle (7c9e6679-7425-40de-944b-e07fc1f90ae7)
✅ Hydrated lobby: Quick Match (7c9e6679-7425-40de-944b-e07fc1f90ae8)
   2 lobbies migrated

╔═══════════════════════════════════════════════╗
║  🎉 Hydration Complete!                      ║
║  ✅ 3 users migrated
║  ✅ 2 lobbies migrated
╚═══════════════════════════════════════════════╝

✨ Hydration script completed successfully!
```

## Idempotency

The script is **safe to run multiple times**:

-   Uses `ON CONFLICT (id) DO NOTHING`
-   Skips already-migrated records
-   No data loss or duplication

## Troubleshooting

### "Failed to get Redis connection"

-   Check `REDIS_URL` in `.env`
-   Ensure Redis is running

### "Failed to insert user/lobby"

-   Check foreign key constraints (lobbies need users and games to exist first)
-   Ensure PostgreSQL migrations have run
-   Check that user already exists if lobbying dependency error

### Missing Fields

-   Script handles missing optional fields gracefully
-   Warns about skipped records with missing required fields

## Next Steps

After hydration completes:

1. **Verify data**: Check PostgreSQL tables have correct counts
2. **Continue to Phase 2**: Create new state models (LobbyState, PlayerState)
3. **Phase 3**: Create state repositories
4. **Phase 4**: Restructure Redis keys (separate migration script)
5. **Phase 5**: Update application code

See `STATE_MANAGEMENT_PROPOSAL.md` for the complete migration strategy.

## Implementation Details

### Why Raw SQL?

The hydration uses raw SQL `INSERT` statements instead of repository methods because:

1. **Repository signatures don't match** - They were designed for new data creation, not migration
2. **Need full control** - Want to set specific IDs, timestamps from Redis
3. **Bypass validation** - Trust existing Redis data is valid
4. **Performance** - Direct inserts are faster

### Code Location

-   **Hydration logic**: `src/db/hydration/mod.rs`
-   **Binary**: `src/bin/hydrate.rs`
-   **Module export**: `src/db/mod.rs`

## Phase 2: Redis → Redis State Restructuring

After PostgreSQL hydration completes, we'll create separate hydration functions for:

### LobbyState Hydration (`lobbies:{id}:info` → `lobbies:{id}:state`)

```rust
pub async fn hydrate_lobby_states_in_redis(redis: &RedisClient) -> Result<usize, AppError>
```

-   Read from `lobbies:{id}:info` (old LobbyInfo)
-   Extract state-specific fields
-   Write to `lobbies:{id}:state` (new LobbyState)
-   Keep original keys until migration complete

### PlayerState Hydration (`lobbies:{id}:player:{user_id}` → `lobbies:{id}:players:{user_id}`)

```rust
pub async fn hydrate_player_states_in_redis(redis: &RedisClient) -> Result<usize, AppError>
```

-   Read from `lobbies:{id}:player:{user_id}` (old Player)
-   Extract platform-generic fields only
-   Write to `lobbies:{id}:players:{user_id}` (new PlayerState)
-   Game-specific data (used_words) goes to GameState

### GameState Hydration (game-specific keys → `lobbies:{id}:game_state`)

```rust
pub async fn hydrate_game_states_in_redis(redis: &RedisClient) -> Result<usize, AppError>
```

-   Read from various game-specific keys:
    -   `lobbies:{id}:used_words`
    -   `lobbies:{id}:current_rule`
    -   `lobbies:{id}:rule_context`
    -   `lobbies:{id}:current_turn`
    -   etc.
-   Combine into LexiWarsGameState
-   Write to `lobbies:{id}:game_state` as JSON

This will be implemented in **Phase 4** of the migration roadmap.

## Future TODO

Once state management refactoring is complete:

-   [ ] Hydrate `games` table (if not manually created)
-   [ ] Hydrate `seasons` table
-   [ ] Hydrate `user_wars_points` table
-   [ ] Create Phase 4 migration script for Redis → Redis restructuring
-   [ ] Archive old Redis keys after migration complete
