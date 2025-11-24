// Lobby connection manager helpers
use crate::lobby::handler::LobbyServerMessage;
use crate::state::{AppState, ConnectionInfo};
use crate::ws::core::connection;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

/// Register a connection under its `connection_id` and add it to the lobby index.
pub async fn register_connection(state: &AppState, connection_id: Uuid, conn: Arc<ConnectionInfo>) {
    // Insert into global connections map
    let mut conns = state.connections.lock().await;
    conns.insert(connection_id, conn.clone());
    drop(conns);

    // Insert into lobby index
    let mut lmap = state.lobby_connections.lock().await;
    let set = lmap.entry(conn.lobby_id).or_insert_with(HashSet::new);
    set.insert(connection_id);
}

/// Unregister connection by `connection_id` and remove it from the lobby index.
pub async fn unregister_connection(state: &AppState, connection_id: &Uuid) {
    // Remove from global connections map and capture lobby_id
    let mut conns = state.connections.lock().await;
    if let Some(conn) = conns.remove(connection_id) {
        let lobby = conn.lobby_id;
        drop(conns);

        // Remove from lobby index
        let mut lmap = state.lobby_connections.lock().await;
        if let Some(set) = lmap.get_mut(&lobby) {
            set.remove(connection_id);
            if set.is_empty() {
                lmap.remove(&lobby);
            }
        }
    }
}

/// Send a typed server message to a specific connection.
pub async fn send_to_connection(conn: &Arc<ConnectionInfo>, msg: &LobbyServerMessage) {
    let _ = connection::send_json(conn, msg).await;
}
