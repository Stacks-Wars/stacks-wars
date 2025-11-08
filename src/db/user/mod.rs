//! User Repository Module
//!
//! This module implements the Repository Pattern for user data access.
//! All PostgreSQL operations for users are encapsulated here.
//!
//! # Architecture
//! ```text
//! Handlers → Services → Repository → PostgreSQL
//! ```
//!
//! # Organization
//! - `create.rs` - User creation operations
//! - `read.rs` - User retrieval operations
//! - `update.rs` - User modification operations
//! - `delete.rs` - User deletion operations
//! - `search.rs` - User search and filtering
//!
//! **Note**: Wars points operations moved to `db::user_wars_points::UserWarsPointsRepository`

mod create;
mod delete;
mod read;
mod search;
mod update;

use sqlx::PgPool;

/// User Repository - Encapsulates all PostgreSQL user operations
///
/// This struct provides a clean interface for user data access.
/// All methods are async and return `Result<T, AppError>`.
///
/// # Examples
/// ```rust,ignore
/// let repo = UserRepository::new(postgres_pool);
/// let user = repo.find_by_id(user_id).await?;
/// ```
pub struct UserRepository {
    pub(crate) pool: PgPool,
}

impl UserRepository {
    /// Create a new UserRepository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
