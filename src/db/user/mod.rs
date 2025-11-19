// User repository module: CRUD operations for users (Postgres)

mod create;
mod delete;
mod read;
mod search;
mod update;

use sqlx::PgPool;

/// User repository for PostgreSQL-backed operations.
pub struct UserRepository {
    pub(crate) pool: PgPool,
}

impl UserRepository {
    /// Create a new `UserRepository`.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
