//! Database models representing PostgreSQL tables
//!
//! This module contains all models that map directly to database tables.
//! All structs here derive `FromRow` for SQLx compatibility.

pub mod game;
pub mod lobby;
pub mod platform_rating;
pub mod season;
pub mod user;

pub use game::GameV2;
pub use lobby::Lobby;
pub use platform_rating::PlatformRating;
pub use season::Season;
pub use user::{UserV2, UserWarsPoints};
