// SpectatorState repository: runtime spectator state helpers for Redis

mod create;
mod read;

use crate::state::RedisClient;

/// Repository for spectator state operations (Redis-backed).
#[derive(Clone)]
pub struct SpectatorStateRepository {
    pub(crate) redis: RedisClient,
}

impl SpectatorStateRepository {
    /// Create a new `SpectatorStateRepository`.
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }
}
