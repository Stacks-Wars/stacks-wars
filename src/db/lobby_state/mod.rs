//! # Lobby State Repository
//!
//! Repository for managing LobbyState in Redis.
//!
//! LobbyState represents the runtime state of a lobby (status, participants, timing).
//! This is separate from the PostgreSQL Lobby model which stores persistent configuration.
//!
//! ## Redis Key Pattern
//! Keys are constructed via `crate::models::redis::keys::RedisKey` helpers.
//! For example: `RedisKey::lobby_state(lobby_id)` -> `lobbies:{lobby_id}:state`

mod create;
mod delete;
mod read;
mod update;

use crate::state::RedisClient;

/// Repository for LobbyState operations
#[derive(Clone)]
pub struct LobbyStateRepository {
    pub(crate) redis: RedisClient,
}

impl LobbyStateRepository {
    /// Create a new LobbyStateRepository
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }
}
