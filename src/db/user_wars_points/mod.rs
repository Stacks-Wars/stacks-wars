use sqlx::PgPool;

mod create;
mod delete;
mod read;
mod update;

/// Repository for managing user wars points (seasonal data)
///
/// This repository handles all operations related to user points earned
/// in specific seasons. Wars points are separate from the global user profile
/// and track performance per season.
///
/// # Architecture
/// - **create.rs** - Creating new wars points entries
/// - **read.rs** - Fetching wars points and leaderboards
/// - **update.rs** - Modifying points and badges
/// - **delete.rs** - Removing wars points data
///
/// # Usage
/// ```rust,ignore
/// use crate::db::user_wars_points::UserWarsPointsRepository;
///
/// let repo = UserWarsPointsRepository::new(postgres_pool);
/// let points = repo.get_wars_points(user_id, season_id).await?;
/// ```
#[derive(Clone)]
pub struct UserWarsPointsRepository {
    pub(crate) pool: PgPool,
}

impl UserWarsPointsRepository {
    /// Create a new UserWarsPointsRepository
    ///
    /// # Arguments
    /// * `pool` - PostgreSQL connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
