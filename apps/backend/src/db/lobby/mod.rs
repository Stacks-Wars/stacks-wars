use sqlx::PgPool;

// Re-export types from read module
pub use read::LobbyWithJoins;

/// Lobby repository for CRUD operations (backed by `lobbies` table).
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

mod create;
mod delete;
mod read;
mod update;
