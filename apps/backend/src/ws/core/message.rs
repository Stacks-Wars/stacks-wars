/// Unified message trait for WebSocket broadcasting
///
/// This trait allows the hub to broadcast both LobbyServerMessage and game events
/// without needing separate broadcast functions for each type.
use serde::Serialize;
use serde_json::Value;

/// Trait for messages that can be broadcast over WebSocket
pub trait BroadcastMessage {
    fn to_json(&self) -> Result<String, serde_json::Error>;
}

/// Implement for any Serialize type (covers LobbyServerMessage and game events)
impl<T: Serialize> BroadcastMessage for T {
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Wrapper for raw JSON values (for game events that are already serialized)
#[derive(Debug, Clone)]
pub struct JsonMessage(pub Value);

impl BroadcastMessage for JsonMessage {
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.0)
    }
}

impl From<Value> for JsonMessage {
    fn from(value: Value) -> Self {
        JsonMessage(value)
    }
}
