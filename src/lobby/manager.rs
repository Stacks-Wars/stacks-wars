// Lobby connection manager helpers
use crate::lobby::handler::LobbyServerMessage;
use crate::state::{AppState, ConnectionInfo};
use crate::ws::core::{connection, hub};
use std::sync::Arc;
use uuid::Uuid;

/// Register a connection for `user_id`.
pub async fn register_connection(state: &AppState, user_id: Uuid, conn: Arc<ConnectionInfo>) {
    let mut map = state.connections.lock().await;
    map.insert(user_id, conn);
}

/// Unregister connection for `user_id`.
pub async fn unregister_connection(state: &AppState, user_id: &Uuid) {
    let mut map = state.connections.lock().await;
    map.remove(user_id);
}

/// Broadcast a `LobbyServerMessage` to all participants (players + spectators)
/// of the given lobby. This uses the generic hub helper which avoids holding
/// the global connection lock while sending.
pub async fn broadcast(state: &AppState, lobby_id: Uuid, msg: &LobbyServerMessage) {
    hub::broadcast_lobby(state, lobby_id, msg).await
}

/// Send a typed server message to a specific connection.
pub async fn send_to_connection(conn: &Arc<ConnectionInfo>, msg: &LobbyServerMessage) {
    let _ = connection::send_json(conn, msg).await;
}

/// Send raw text to a connection.
pub async fn send_text_to_conn(conn: &Arc<ConnectionInfo>, text: String) {
    connection::send_text(conn, text).await;
}
