use sqlx::PgPool;

mod create;
mod read;
mod update;

/// Season repository: create/read/update operations for competitive seasons.
///
/// Modules: `create`, `read`, `update`.
#[derive(Clone)]
pub struct SeasonRepository {
    pub(crate) pool: PgPool,
}

impl SeasonRepository {
    /// Create a new SeasonRepository
    ///
    /// # Arguments
    /// * `pool` - PostgreSQL connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
