//! # Player State Repository
//!
//! Repository for managing PlayerState in Redis.
//!
//! PlayerState represents the runtime state of a player in a lobby.
//! This is GENERIC and contains NO game-specific data.
//!
//! ## Redis Key Pattern
//! Keys are constructed via `crate::models::redis::keys::RedisKey` helpers.
//! For example: `RedisKey::lobby_player(lobby_id, user_id)` -> `lobbies:{lobby_id}:players:{user_id}`

mod create;
mod delete;
mod read;
mod update;

use crate::state::RedisClient;

/// Repository for PlayerState operations
#[derive(Clone)]
pub struct PlayerStateRepository {
    pub(crate) redis: RedisClient,
}

impl PlayerStateRepository {
    /// Create a new PlayerStateRepository
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }
}
