use sqlx::PgPool;

mod create;
mod delete;
mod read;
mod update;

/// Repository for managing game types
///
/// Handles PostgreSQL operations for games (e.g., "Lexi Wars", "Word Battle").
/// Each game defines rules, player limits, and platform availability.
#[derive(Clone)]
pub struct GameRepository {
    pub(crate) pool: PgPool,
}

impl GameRepository {
    /// Create a new GameRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
