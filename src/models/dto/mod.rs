//! Data Transfer Objects (DTOs)
//!
//! This module contains structs used for API requests, responses, and queries.
//! These are typically not stored directly but used for data transfer.

//pub mod lobby;
pub mod pagination;
pub mod query;

//pub use lobby::{JoinRequest, LobbyExtended, LobbyPoolInput, PlayerLobbyInfo};
pub use pagination::{PaginatedResponse, Pagination, PaginationMeta};
pub use query::{LobbyQuery, PlayerQuery, WsQueryParams, parse_lobby_states, parse_player_state};
