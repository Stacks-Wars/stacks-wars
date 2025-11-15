//! Protocol crate-level module for canonical websocket message shapes
//!
//! Keep protocol types centralized so all websocket handlers and domain
//! managers reuse the same canonical serde shapes.
pub mod lobby;
