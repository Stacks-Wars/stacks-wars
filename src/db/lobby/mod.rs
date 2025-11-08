use sqlx::PgPool;

/// Repository for Lobby database operations
///
/// Provides CRUD operations for multiplayer game lobbies.
/// Lobbies are game rooms where players gather before matches start.
///
/// # Related Tables
/// - `lobbies` (main table)
/// - Foreign keys: `creator_id` → users, `game_id` → games
///
/// # Usage Example
/// ```rust
/// let repo = LobbyRepository::new(pool.clone());
/// let lobby = repo.create_lobby(
///     "Epic Match",
///     Some("High stakes game"),
///     creator_id,
///     game_id,
///     Some(100.0),
///     Some("STX"),
///     false,
/// ).await?;
/// ```
pub struct LobbyRepository {
    pool: PgPool,
}

impl LobbyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get a reference to the database pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

// Operation modules
mod create;
mod delete;
mod read;
mod update;
