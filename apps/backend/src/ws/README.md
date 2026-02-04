# WebSocket Module

Real-time communication handlers for lobby updates and in-game messaging.

## Structure

```
ws/
├── mod.rs              # Module exports
├── routes.rs           # WebSocket route definitions
├── broadcast.rs        # Centralized broadcast helpers
│
├── core/               # Core WebSocket infrastructure
│   ├── mod.rs
│   ├── manager.rs      # Connection management
│   └── message.rs      # Base message types
│
├── lobby/              # Lobby list channel (/ws/lobby)
│   ├── mod.rs
│   ├── handler.rs      # Connection handler
│   ├── messages.rs     # Lobby list messages
│   └── error.rs        # Lobby-specific errors
│
└── room/               # Game room channel (/ws/room/{id})
    ├── mod.rs
    ├── handler.rs      # Connection handler
    ├── engine.rs       # Game engine integration
    ├── messages.rs     # Room messages
    └── error.rs        # Room-specific errors
```

## Channels

### Lobby Channel (`/ws/lobby`)

Broadcasts lobby list updates to all connected clients.

**Events:**

- `LobbyCreated` - New lobby created
- `LobbyUpdated` - Lobby state changed
- `LobbyDeleted` - Lobby removed

**Usage:**

```javascript
const ws = new WebSocket("wss://api.example.com/ws/lobby");
ws.onmessage = (event) => {
	const msg = JSON.parse(event.data);
	switch (msg.type) {
		case "LobbyCreated":
			addLobby(msg.lobby);
			break;
		case "LobbyUpdated":
			updateLobby(msg.lobby);
			break;
	}
};
```

### Room Channel (`/ws/room/{lobby_id}`)

Handles in-game communication for a specific lobby.

**Client Messages:**

- `Ready` - Player ready to start
- `Unready` - Player not ready
- `GameAction` - Game-specific action
- `Chat` - Chat message

**Server Messages:**

- `PlayerJoined` - Player entered room
- `PlayerLeft` - Player left room
- `PlayerUpdated` - Player state changed
- `GameStarted` - Game has begun
- `GameMessage` - Game-specific event
- `GameOver` - Game ended, show results
- `ChatMessage` - Chat from player

## Broadcasting

All broadcasts go through `src/ws/broadcast.rs`:

```rust
use crate::ws::broadcast;

// Broadcast to all lobby list subscribers
broadcast::broadcast_lobby_update(&state, lobby_id).await;

// Broadcast to all players in a room
broadcast::broadcast_room(&state, lobby_id, &message).await;

// Broadcast to a specific user
broadcast::broadcast_user(&state, user_id, &message).await;

// Broadcast game-specific message
broadcast::broadcast_game_message(&state, lobby_id, &game_event).await;
```

## Message Format

All WebSocket messages are JSON with a `type` field:

```rust
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum RoomServerMessage {
    PlayerJoined { player: PlayerState },
    GameMessage { data: serde_json::Value },
    Error { code: String, message: String },
}
```

Serializes to:

```json
{
	"type": "PlayerJoined",
	"player": { "id": "...", "username": "..." }
}
```

## Connection Management

Connections are tracked in `AppState`:

```rust
pub struct AppState {
    // Lobby list subscribers
    lobby_connections: DashMap<Uuid, Sender<Message>>,

    // Room subscribers (lobby_id -> user_id -> sender)
    room_connections: DashMap<Uuid, DashMap<Uuid, Sender<Message>>>,
}
```

## Error Handling

WebSocket errors convert to error messages:

```rust
#[derive(Debug, thiserror::Error)]
pub enum RoomError {
    #[error("Not in lobby")]
    NotInLobby,

    #[error("Game not started")]
    GameNotStarted,
}

impl From<RoomError> for RoomServerMessage {
    fn from(err: RoomError) -> Self {
        RoomServerMessage::Error {
            code: err.code().to_string(),
            message: err.to_string(),
        }
    }
}
```

## Game Engine Integration

The room handler routes game actions to the appropriate engine:

```rust
// In room/handler.rs
match client_message {
    RoomClientMessage::GameAction { data } => {
        if let Some(engine) = state.get_game_engine(lobby_id) {
            engine.handle_action(user_id, data).await;
        }
    }
}
```

See [src/games/README.md](../games/README.md) for game engine details.
