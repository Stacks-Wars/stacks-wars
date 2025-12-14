// LobbyState repository (Redis): runtime state helpers and key patterns

mod create;
mod delete;
mod read;
mod update;

use crate::state::RedisClient;

/// Repository for lobby state operations.
#[derive(Clone)]
pub struct LobbyStateRepository {
    pub(crate) redis: RedisClient,
}

impl LobbyStateRepository {
    /// Create a new `LobbyStateRepository`.
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }
}
