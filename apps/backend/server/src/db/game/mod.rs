use sqlx::PgPool;

mod create;
mod delete;
mod read;
mod update;

/// Repository for managing game types and related DB operations.
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
