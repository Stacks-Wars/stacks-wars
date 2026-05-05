use sqlx::PgPool;

mod create;
mod delete;
mod read;
mod update;

#[derive(Clone)]
pub struct PlatformRatingRepository {
    pub(crate) pool: PgPool,
}

impl PlatformRatingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
