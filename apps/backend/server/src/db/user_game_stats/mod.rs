use sqlx::PgPool;

mod read;
mod update;

/// Repository for per-game user statistics.
#[derive(Clone)]
pub struct UserGameStatsRepository {
    pub(crate) pool: PgPool,
}

impl UserGameStatsRepository {
    /// Create a new `UserGameStatsRepository` with the given pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
