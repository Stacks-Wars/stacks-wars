//  LobbyChatRepository: runtime Redis helpers for lobby chat messages

mod create;
mod delete;
mod read;
mod update;

use crate::state::RedisClient;

/// LobbyChatRepository (wraps the Redis client).
#[derive(Clone)]
pub struct LobbyChatRepository {
    pub(crate) redis: RedisClient,
}

impl LobbyChatRepository {
    /// Create a new `LobbyChatRepository`.
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }
}
