// PlayerState repository: runtime Redis helpers for player state

mod create;
mod delete;
mod read;
mod update;
mod sync;

use crate::state::RedisClient;

/// PlayerState repository (wraps the Redis client).
#[derive(Clone)]
pub struct PlayerStateRepository {
    pub(crate) redis: RedisClient,
}

impl PlayerStateRepository {
    /// Create a new `PlayerStateRepository`.
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }
}
