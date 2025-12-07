use crate::state::{AppState, ConnectionInfo};
use axum::extract::ws::Message;
use futures::SinkExt;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

/// Send a serializable message to a connection
pub async fn send_to_connection<M: Serialize>(
    conn: &Arc<ConnectionInfo>,
    msg: &M,
) -> Result<(), serde_json::Error> {
    let json = serde_json::to_string(msg)?;
    let mut s = conn.sender.lock().await;
    let _ = s.send(Message::Text(json.into())).await;
    Ok(())
}

/// Register a connection under its `connection_id` and add it to all relevant indices.
pub async fn register_connection(state: &AppState, connection_id: Uuid, conn: Arc<ConnectionInfo>) {
    // Insert into global connections map
    let mut conns = state.connections.lock().await;
    conns.insert(connection_id, conn.clone());
    drop(conns);

    // Insert into all indices
    let mut indices = state.indices.lock().await;
    indices.insert(&conn);
}

/// Unregister connection by `connection_id` and remove it from all indices.
pub async fn unregister_connection(state: &AppState, connection_id: &Uuid) {
    // Remove from global connections map
    let mut conns = state.connections.lock().await;
    if let Some(conn) = conns.remove(connection_id) {
        drop(conns);

        // Remove from all indices
        let mut indices = state.indices.lock().await;
        indices.remove(&conn);
    }
}

/// Unregister all connections for a specific lobby (e.g., when lobby closes).
pub async fn unregister_lobby_connections(state: &AppState, lobby_id: Uuid) -> usize {
    let mut indices = state.indices.lock().await;

    // Get all connection IDs for this lobby before removing
    let conn_ids: Vec<Uuid> = indices
        .get_lobby_connections(&lobby_id)
        .map(|set| set.iter().copied().collect())
        .unwrap_or_default();

    let count = conn_ids.len();

    // Remove from global connections
    let mut conns = state.connections.lock().await;
    for conn_id in &conn_ids {
        if let Some(conn) = conns.remove(conn_id) {
            indices.remove(&conn);
        }
    }

    count
}

/// Unregister all connections for a specific user (e.g., when user is banned).
pub async fn unregister_user_connections(state: &AppState, user_id: Uuid) -> usize {
    let mut indices = state.indices.lock().await;

    // Get all connection IDs for this user
    let conn_ids: Vec<Uuid> = indices
        .get_user_connections(&user_id)
        .map(|set| set.iter().copied().collect())
        .unwrap_or_default();

    let count = conn_ids.len();

    // Remove from global connections
    let mut conns = state.connections.lock().await;
    for conn_id in &conn_ids {
        if let Some(conn) = conns.remove(conn_id) {
            indices.remove(&conn);
        }
    }

    count
}
