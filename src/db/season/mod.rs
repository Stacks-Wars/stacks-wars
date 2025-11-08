use sqlx::PgPool;

mod create;
mod read;
mod update;

/// Repository for managing game seasons
///
/// Handles all operations related to seasons including creation, retrieval,
/// and updates. Seasons define time periods for competitions and leaderboards.
///
/// # Architecture
/// - **create.rs** - Creating new seasons
/// - **read.rs** - Fetching season data
/// - **update.rs** - Modifying existing seasons
///
/// # Usage
/// ```rust,ignore
/// use crate::db::season::SeasonRepository;
///
/// let repo = SeasonRepository::new(postgres_pool);
/// let current_season = repo.get_current_season().await?;
/// ```
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
