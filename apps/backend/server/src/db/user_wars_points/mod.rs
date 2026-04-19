use sqlx::PgPool;

mod create;
mod delete;
mod read;
mod update;

/// Repository for seasonal user wars points.
#[derive(Clone)]
pub struct UserWarsPointsRepository {
    pub(crate) pool: PgPool,
}

impl UserWarsPointsRepository {
    /// Create a new `UserWarsPointsRepository` with the given pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
