//! Models module - organized by storage layer and purpose
//!
//! ## Structure
//! - `db/` - PostgreSQL models (FromRow-derived structs)
//! - `redis/` - Redis cache models (GameType, LobbyInfo, Player)
//! - `dto/` - Data Transfer Objects (requests, responses, queries)

pub mod db;
pub mod dto;
pub mod redis;
