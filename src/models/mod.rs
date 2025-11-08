//! Models module - organized by storage layer and purpose
//!
//! ## Structure
//! - `db/` - PostgreSQL models (FromRow-derived structs)
//! - `redis/` - Redis cache models (GameType, LobbyInfo, Player)
//! - `dto/` - Data Transfer Objects (requests, responses, queries)
//! - `enums/` - Shared enums (PlayerState, LobbyState, etc.)
//!
//! ## Legacy modules (to be gradually migrated)
//! Old flat structure files are kept temporarily for backward compatibility.

// New layered structure
pub mod db;
pub mod dto;
pub mod enums;
pub mod redis;

// Legacy flat modules (temporary - for backward compatibility)
pub mod chat;
pub mod game;
//pub mod leaderboard;
pub mod lexi_wars;
pub mod lobby;
pub mod platform_rating;
pub mod redis_key;
pub mod season;
pub mod user;
