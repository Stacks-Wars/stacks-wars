//! # Spectator State Repository
//!
//! Repository for managing SpectatorState in Redis.
//!
//! SpectatorState represents the runtime state of a spectator in a lobby.
//!
//! ## Redis Key Pattern
//! Keys are constructed via `crate::models::redis::keys::RedisKey` helpers.
//! For example: `RedisKey::lobby_spectator(lobby_id, user_id)` -> `lobbies:{lobby_id}:spectators:{user_id}`

mod create;
mod read;

use crate::state::RedisClient;

/// Repository for SpectatorState operations
#[derive(Clone)]
pub struct SpectatorStateRepository {
    pub(crate) redis: RedisClient,
}

impl SpectatorStateRepository {
    /// Create a new SpectatorStateRepository
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }
}
