# stacks-wars

A multiplayer gaming platform built on Stacks blockchain.

## Features

### Frontend

- **Next.js 16** - Full-stack React framework
- **React Native + Expo** - Cross-platform mobile apps
- **TailwindCSS v4** - Utility-first CSS
- **shadcn/ui** - Reusable UI components
- **Turborepo** - Optimized monorepo build system

### Backend

- **Rust + Axum** - High-performance HTTP/WebSocket server
- **PostgreSQL** - Persistent storage (users, lobbies, games)
- **Redis** - Runtime state (game sessions, chat, player state)
- **SQLx** - Type-safe SQL queries

### Shared

- **TypeScript** - Type safety across frontend apps
- **Better-Auth** - Authentication
- **Drizzle ORM** - Database schema & queries

## Getting Started

### Prerequisites

- [Bun](https://bun.sh/) - JavaScript runtime & package manager
- [Rust](https://rustup.rs/) - For backend development
- PostgreSQL database
- Redis server

### Installation

```bash
bun install
```

### Development

```bash
# Start all apps (web)
bun run dev
cargo run

# Or start individually
bun run dev:web      # Next.js frontend
cargo run  # Rust backend
bun run dev:native   # React Native/Expo
```

- Web: [http://localhost:3001](http://localhost:3001)
- Backend API: [http://localhost:3000](http://localhost:3000)
- Mobile: Use Expo Go app

## Project Structure

```
stacks-wars/
├── apps/
│   ├── backend/     # Rust game server (Axum, WebSocket)
│   ├── web/         # Next.js web application
│   ├── native/      # React Native mobile app (Expo)
│   └── fumadocs/    # Documentation site
│
├── packages/
│   ├── auth/        # Authentication configuration
│   ├── config/      # Shared TypeScript config
│   ├── db/          # Drizzle schema & queries
│   └── shared/      # Shared utilities
│
└── docs/            # Additional documentation
```

## Backend Documentation

The Rust backend has comprehensive documentation in each module:

| Module        | Path                                                                   | Description                            |
| ------------- | ---------------------------------------------------------------------- | -------------------------------------- |
| **Main**      | [apps/backend/README.md](apps/backend/README.md)                       | Architecture overview, key concepts    |
| **Games**     | [apps/backend/src/games/README.md](apps/backend/src/games/README.md)   | How to add new games, GameEngine trait |
| **WebSocket** | [apps/backend/src/ws/README.md](apps/backend/src/ws/README.md)         | Real-time channels, message formats    |
| **HTTP**      | [apps/backend/src/http/README.md](apps/backend/src/http/README.md)     | REST API handlers, route organization  |
| **Database**  | [apps/backend/src/db/README.md](apps/backend/src/db/README.md)         | Repository pattern, PostgreSQL + Redis |
| **Models**    | [apps/backend/src/models/README.md](apps/backend/src/models/README.md) | Domain models, DTOs, Redis keys        |
| **Auth**      | [apps/backend/src/auth/README.md](apps/backend/src/auth/README.md)     | JWT authentication, extractors         |
| **CLI Tools** | [apps/backend/src/bin/README.md](apps/backend/src/bin/README.md)       | Hydration & migration scripts          |

## Available Scripts

### Development

- `bun run dev` - Start all applications
- `bun run dev:web` - Start Next.js frontend
- `bun run dev:native` - Start Expo development server
- `cargo run` - Start Rust backend

### Build

- `bun run build` - Build all applications
- `cargo run build` - Build Rust Backend
- `bun run check-types` - TypeScript type checking

## Test

- `cargo test` - Run Backend tests
