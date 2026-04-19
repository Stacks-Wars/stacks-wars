# Stacks Wars Backend

A Rust/Axum multiplayer game platform with PostgreSQL for persistence and Redis for runtime state. WebSocket handles real-time communication; HTTP handles REST endpoints.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Clients                                  │
│                  (Web, Mobile, Desktop)                          │
└─────────────────────────┬───────────────────────────────────────┘
                          │
            ┌─────────────┴─────────────┐
            │                           │
            ▼                           ▼
    ┌───────────────┐           ┌───────────────┐
    │   HTTP REST   │           │   WebSocket   │
    │   (Axum)      │           │   (Axum)      │
    └───────┬───────┘           └───────┬───────┘
            │                           │
            └─────────────┬─────────────┘
                          │
                          ▼
              ┌───────────────────────┐
              │      AppState         │
              │  ┌─────────────────┐  │
              │  │ PostgreSQL Pool │  │
              │  │ Redis Pool      │  │
              │  │ WS Connections  │  │
              │  │ Game Registry   │  │
              │  └─────────────────┘  │
              └───────────────────────┘
                          │
            ┌─────────────┴─────────────┐
            │                           │
            ▼                           ▼
    ┌───────────────┐           ┌───────────────┐
    │  PostgreSQL   │           │     Redis     │
    │  (Persist)    │           │   (Runtime)   │
    └───────────────┘           └───────────────┘
```

## Project Structure

```
src/
├── main.rs          # Application entry point
├── lib.rs           # Library exports
├── state.rs         # AppState - shared application state
├── errors.rs        # Global error types (AppError)
├── middleware.rs    # HTTP middleware (rate limiting, auth)
│
├── auth/            # Authentication (JWT, extractors)
├── db/              # Database repositories (PostgreSQL + Redis)
├── games/           # Game engines (implement GameEngine trait)
├── http/            # HTTP handlers and routes
├── models/          # Domain models and Redis key builders
├── ws/              # WebSocket handlers (lobby list, game rooms)
│
├── bin/             # CLI tools
│   ├── hydrate.rs       # Hydrate Redis from PostgreSQL
│   └── migrate_redis.rs # Redis data migrations
│
└── assets/          # Static assets (dictionary.json for word games)
```

## Key Concepts

### Dual Storage Model

| Storage    | Purpose                           | Examples                        |
| ---------- | --------------------------------- | ------------------------------- |
| PostgreSQL | Persistence, source of truth      | Users, lobbies, games, seasons  |
| Redis      | Runtime state, fast mutable cache | Lobby state, player state, chat |

### Lobby Lifecycle

```
Create → Join/Leave → Ready Up → Game Start → Game End → Cleanup
```

1. **Create**: Host creates lobby, stored in PostgreSQL + Redis
2. **Join/Leave**: Players join via join requests or direct join
3. **Ready Up**: Players mark ready, host can start when all ready
4. **Game Start**: Game engine takes over, manages game state
5. **Game End**: Results saved, prizes distributed
6. **Cleanup**: Redis state cleared, PostgreSQL updated

### WebSocket Channels

| Channel    | Path                  | Purpose                      |
| ---------- | --------------------- | ---------------------------- |
| Lobby List | `/ws/lobby`           | Real-time lobby list updates |
| Game Room  | `/ws/room/{lobby_id}` | In-game communication        |

## Commands

```bash
# Development (from monorepo root; apps/backend is a nested Cargo workspace)
cargo check --manifest-path apps/backend/Cargo.toml --workspace
cargo test --manifest-path apps/backend/Cargo.toml --workspace
cargo run --manifest-path apps/backend/Cargo.toml -p stacks_wars_server --bin stacks_wars_server

# CLI Tools
cargo run --bin hydrate        # Hydrate Redis from PostgreSQL
cargo run --bin migrate_redis  # Run Redis migrations

# Environment Variables
DATABASE_URL=postgres://...    # PostgreSQL connection
REDIS_URL=redis://...          # Redis connection
JWT_SECRET=...                 # JWT signing secret
```

## Adding a New Feature

### Adding a New Game

See [server/src/games/README.md](server/src/games/README.md) for a complete walkthrough.

### Adding a New HTTP Endpoint

1. Add handler in `src/http/handlers/`
2. Add route in appropriate `src/http/routes/` file
3. Export from `mod.rs` files

### Adding a New Database Entity

1. Create migration in `migrations/`
2. Create repository folder in `src/db/` with:
    - `mod.rs` - Repository struct
    - `create.rs`, `read.rs`, `update.rs`, `delete.rs` - CRUD operations
3. Add model in `src/models/`
4. Add Redis keys in `src/models/keys.rs` if needed

## Testing

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test ws

# Run tests with output
cargo test -- --nocapture
```

Tests use testcontainers for PostgreSQL and Redis isolation.
