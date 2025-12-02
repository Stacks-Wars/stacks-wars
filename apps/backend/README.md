# Stacks Wars Backend 🎮

A real-time multiplayer word battle game server built with Rust, featuring WebSocket connections, Redis persistence, and competitive leaderboards.

## 🎯 Overview

Stacks Wars is a competitive word game where players battle in real-time lobbies, forming words under various rule constraints. Players earn wars points based on their performance and can compete for prizes in pooled matches.

**Frontend Client**: [Stacks Wars Frontend](https://github.com/iflames1/stacks-wars)

## 🚀 Features

### Core Gameplay

-   **Real-time multiplayer**: WebSocket-based gameplay with instant message broadcasting
-   **Word validation**: Dictionary-based word checking with rule enforcement
-   **Turn-based mechanics**: Timed turns with automatic progression
-   **Dynamic rules**: Various word formation rules (minimum length, required letters, etc.)

### Lobby System

-   **Lobby creation & management**: Public/private lobbies with customizable settings
-   **Pool-based betting**: Optional entry fees with prize distribution
-   **Auto-start timers**: Games begin automatically when enough players join
-   **Reconnection support**: Players can reconnect to ongoing games

### User Management

-   **JWT authentication**: Secure user sessions
-   **Wars points system**: Competitive scoring with positive/negative points
-   **Username & display names**: Customizable player identities
-   **Leaderboards**: Global rankings with win rates and PnL tracking

### Real-time Chat

-   **Lobby chat**: In-game communication between players
-   **Message persistence**: Chat history stored in Redis with TTL
-   **Offline message queuing**: Messages delivered when players reconnect

### Data Persistence

-   **Redis backend**: All game state, user data, and chat stored in Redis
-   **Atomic operations**: Race condition prevention with Redis transactions
-   **TTL management**: Automatic cleanup of expired data

## 🛠 Tech Stack

-   **Backend**: Rust with Axum web framework
-   **WebSockets**: Real-time bidirectional communication
-   **Database**: Redis for all data persistence
-   **Authentication**: JWT tokens
-   **Serialization**: Serde for JSON handling

## 📁 Project Structure

```
src/
├── auth/           # JWT authentication logic
├── db/             # Database operations
│   ├── user/       # User CRUD operations
│   ├── lobby/      # Lobby management
│   ├── leaderboard/# Ranking calculations
│   └── chat/       # Chat persistence
├── games/          # Game logic
│   └── lexi_wars/  # Word game implementation
├── http/           # REST API handlers
├── models/         # Data structures
├── state/          # Application state management
└── ws/             # WebSocket handlers
    ├── handlers/
    │   ├── lobby/  # Lobby WebSocket logic
    │   ├── chat/   # Chat WebSocket logic
    │   └── lexi_wars/ # Game WebSocket logic
    └── utils/      # WebSocket utilities
```

## 🎮 Game Flow

1. **User Authentication**: Players authenticate via JWT
2. **Lobby Creation/Joining**: Players create or join game lobbies
3. **Game Start**: Auto-start timer or manual start when ready
4. **Gameplay**: Turn-based word formation with rule validation
5. **Elimination**: Players eliminated for invalid words or timeouts
6. **Prize Distribution**: Winners receive pool prizes (if applicable)
7. **Wars Points**: All players earn/lose points based on performance

## 🏆 Scoring System

### Wars Points Calculation

```rust
base_points = (total_players - rank + 1) * 2
pool_bonus = (prize / total_players) + (entry_fee / 5)  // if pooled game
total_points = min(base_points + pool_bonus, 50)  // capped at 50
```

### Penalties

-   **Leaving lobby**: -10 wars points
-   **Timeout elimination**: Ranked based on elimination order

### Leaderboard Ranking

1. **Primary**: Wars points (descending)
2. **Tiebreaker**: Win rate percentage (descending)

## 🚦 Getting Started

### Prerequisites

-   Rust 1.70+
-   Redis server
-   Environment variables configured

### Environment Setup

```bash
REDIS_URL=redis://localhost:6379
JWT_SECRET=your_jwt_secret_key
TELEGRAM_BOT_TOKEN=your_telegram_bot_token
```

### Running the Server

```bash
# Clone the repository
git clone <repository_url>
cd stacks-wars-be

# Install dependencies and run
cargo run
```

The server will start on `http://localhost:3000`

## 🔮 WebSocket Message Types

### Lobby Messages

```typescript
// Client -> Server
{ type: "joinLobby" }
{ type: "leaveLobby" }
{ type: "updateGameState", newState: "InProgress" }

// Server -> Client
{ type: "playerUpdated", players: Player[] }
{ type: "gameStateUpdated", newState: "InProgress" }
{ type: "lobbyCountdown", time: number }
```

### Game Messages

```typescript
// Client -> Server
{ type: "wordEntry", word: string }
{ type: "ping", ts: number }

// Server -> Client
{ type: "turn", currentTurn: Player }
{ type: "rule", rule: string }
{ type: "wordEntry", word: string, sender: Player }
{ type: "gameOver" }
{ type: "finalStanding", standing: PlayerStanding[] }
{ type: "rank", rank: string }
{ type: "prize", amount: number }
{ type: "warsPoint", warsPoint: number }
```

### Chat Messages

```typescript
// Client -> Server
{ type: "chat", text: string }
{ type: "ping", ts: number }

// Server -> Client
{ type: "chat", message: ChatMessage }
{ type: "chatHistory", messages: ChatMessage[] }
{ type: "permitChat", allowed: boolean }
```

## 🗄️ Redis Schema

### Keys Structure

```
users:{user_id}                           # User data hash
lobbies:{lobby_id}:info                   # Lobby information
lobbies:{lobby_id}:player:{user_id}       # Player in lobby
lobbies:{lobby_id}:connected_player:{user_id} # Connected players
lobbies:{lobby_id}:chats                  # Chat messages list
games:{game_id}:lobbies                   # Game's lobby set
lobbies:waiting:state                     # Lobbies by state
```

## 🤝 Contributing

This is currently a personal project, but contributions and suggestions are welcome! Feel free to:

-   Report bugs via GitHub issues
-   Suggest new features
-   Submit pull requests
-   Improve documentation

## 📜 License

**All Rights Reserved**

Copyright (c) 2024 Stacks Wars Team. All rights reserved.

This software and associated documentation files (the "Software") are proprietary and confidential. No part of this Software may be reproduced, distributed, or transmitted in any form or by any means, including photocopying, recording, or other electronic or mechanical methods, without the prior written permission of the copyright holder, except in the case of brief quotations embodied in critical reviews and certain other noncommercial uses permitted by copyright law.

**Prohibited Activities:**

-   Copying, modifying, or distributing the Software
-   Creating derivative works based on the Software
-   Reverse engineering, decompiling, or disassembling the Software
-   Using the Software for commercial purposes without explicit permission
-   Removing or altering copyright notices

**Permitted Activities:**

-   Viewing the code for educational purposes
-   Contributing to the project via approved pull requests
-   Reporting bugs and suggesting improvements

For permission requests or licensing inquiries, please contact the development team through GitHub issues.

**Disclaimer:** The Software is provided "as is", without warranty of any kind, express or implied, including but not limited to the warranties of merchantability, fitness for a particular purpose and noninfringement. In no event shall the authors or copyright holders be liable for any claim, damages or other liability, whether in an action of contract, tort or otherwise, arising from, out of or in connection with the Software or the use or other dealings in the Software.
