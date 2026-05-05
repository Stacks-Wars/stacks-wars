# HTTP Module

REST API handlers and route definitions using Axum.

## Structure

```
http/
├── mod.rs              # Module exports
├── handlers/           # Request handlers (business logic)
│   ├── mod.rs
│   ├── user.rs         # User CRUD
│   ├── game.rs         # Game info
│   ├── lobby.rs        # Lobby management
│   ├── season.rs       # Season info
│   ├── platform_rating.rs
│   ├── player_stats.rs
│   ├── stacks.rs       # Blockchain interactions
│   └── contract.rs     # Smart contract calls
│
├── routes/             # Route definitions (URL mappings)
│   ├── mod.rs
│   ├── api.rs          # Main API router
│   ├── auth.rs         # Auth routes (login, register)
│   ├── public.rs       # Public routes (no auth required)
│   ├── strict.rs       # Protected routes (auth required)
│   └── admin.rs        # Admin-only routes
│
└── bot/                # Telegram bot handlers
    ├── mod.rs
    ├── handlers.rs
    └── broadcasts.rs
```

## Route Organization

### Public Routes (`/api/public/...`)

No authentication required.

```rust
// routes/public.rs
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/games", get(handlers::game::list_games))
        .route("/games/{id}", get(handlers::game::get_game))
}
```

### Protected Routes (`/api/...`)

Require valid JWT token.

```rust
// routes/strict.rs
pub fn strict_routes() -> Router<AppState> {
    Router::new()
        .route("/lobbies", post(handlers::lobby::create_lobby))
        .route("/user/profile", get(handlers::user::get_profile))
}
```

### Admin Routes (`/api/admin/...`)

Require admin privileges.

```rust
// routes/admin.rs
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/seasons", post(handlers::season::create_season))
}
```

## Handler Pattern

Handlers receive extractors and return responses:

```rust
use axum::{extract::State, Json};
use crate::{state::AppState, auth::AuthUser, errors::AppError};

pub async fn create_lobby(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(payload): Json<CreateLobbyRequest>,
) -> Result<Json<Lobby>, AppError> {
    let lobby = state.lobby_repo()
        .create(&payload, user.id)
        .await?;

    Ok(Json(lobby))
}
```

## Adding a New Endpoint

### 1. Create Handler

```rust
// src/http/handlers/my_feature.rs
use axum::{extract::State, Json};
use crate::{state::AppState, errors::AppError};

pub async fn my_handler(
    State(state): State<AppState>,
) -> Result<Json<MyResponse>, AppError> {
    // Implementation
    Ok(Json(response))
}
```

### 2. Export Handler

```rust
// src/http/handlers/mod.rs
pub mod my_feature;
```

### 3. Add Route

```rust
// src/http/routes/api.rs (or appropriate route file)
.route("/my-feature", get(handlers::my_feature::my_handler))
```

## Request/Response Types

Define request and response types near the handler:

```rust
#[derive(Debug, Deserialize)]
pub struct CreateLobbyRequest {
    pub game_id: Uuid,
    pub max_players: u8,
    pub entry_fee: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct CreateLobbyResponse {
    pub lobby: Lobby,
    pub join_code: String,
}
```

## Error Handling

Handlers return `Result<T, AppError>`. Errors are automatically converted to HTTP responses:

```rust
pub async fn get_lobby(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Lobby>, AppError> {
    let lobby = state.lobby_repo()
        .find_by_id(id)
        .await?
        .ok_or(AppError::NotFound("Lobby not found"))?;

    Ok(Json(lobby))
}
```
