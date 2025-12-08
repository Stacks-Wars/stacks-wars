// WebSocket integration tests entry point
// Sub-modules contain specific test suites

#[path = "common/mod.rs"]
mod common;

#[path = "ws/lobby.rs"]
mod lobby;

#[path = "ws/room.rs"]
mod room;
