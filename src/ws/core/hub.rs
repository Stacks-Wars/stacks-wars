// Hub helpers for broadcasting messages to connected clients
use crate::state::AppState;
use axum::extract::ws::Message;
use futures::SinkExt;
use serde::Serialize;
use tokio::task::JoinHandle;
use uuid::Uuid;

/// Broadcast a serializable message to every active connection in `AppState`.
pub async fn broadcast_all<M: Serialize>(state: &AppState, msg: &M) {
    if let Ok(json) = serde_json::to_string(msg) {
        let conns = state.connections.lock().await;
        let mut handles: Vec<JoinHandle<()>> = Vec::new();
        for (_uid, conn) in conns.iter() {
            let sender = conn.sender.clone();
            let json_clone = json.clone();
            handles.push(tokio::spawn(async move {
                let mut s = sender.lock().await;
                let _ = s.send(Message::Text(json_clone.into())).await;
            }));
        }
        for h in handles {
            let _ = h.await;
        }
    }
}

/// Broadcast to all participants (players + spectators) of a specific lobby.
pub async fn broadcast_lobby<M: Serialize>(state: &AppState, lobby_id: Uuid, msg: &M) {
    // Use the lobby_connections index to find all connection ids in the lobby,
    // then send the message to each connection directly.
    if let Ok(json) = serde_json::to_string(msg) {
        let lmap = state.lobby_connections.lock().await;
        if let Some(set) = lmap.get(&lobby_id) {
            let conns = state.connections.lock().await;
            for cid in set.iter() {
                if let Some(conn) = conns.get(cid) {
                    let sender = conn.sender.clone();
                    let json_clone = json.clone();
                    tokio::spawn(async move {
                        let mut s = sender.lock().await;
                        let _ = s.send(Message::Text(json_clone.into())).await;
                    });
                }
            }
        }
    }
}

/// Broadcast to a specific list of user ids. Lookup connections in `state.connections`.
pub async fn broadcast_to_user_ids<M: Serialize>(
    state: &AppState,
    lobby_id: Uuid,
    user_ids: &[Uuid],
    msg: &M,
) {
    if let Ok(json) = serde_json::to_string(msg) {
        let conns = state.connections.lock().await;
        for uid in user_ids.iter() {
            // find any connection in this lobby that belongs to uid
            for (_cid, conn) in conns.iter() {
                if conn.lobby_id == lobby_id {
                    if let Some(conn_user) = conn.user_id {
                        if &conn_user == uid {
                            let sender = conn.sender.clone();
                            let json_clone = json.clone();
                            tokio::spawn(async move {
                                let mut s = sender.lock().await;
                                let _ = s.send(Message::Text(json_clone.into())).await;
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Send a message to a single connected user (if present).
pub async fn send_to_user<M: Serialize>(state: &AppState, lobby_id: Uuid, user_id: Uuid, msg: &M) {
    if let Ok(json) = serde_json::to_string(msg) {
        let conns = state.connections.lock().await;
        for (_cid, conn) in conns.iter() {
            if conn.lobby_id == lobby_id {
                if let Some(conn_user) = conn.user_id {
                    if conn_user == user_id {
                        let mut s = conn.sender.lock().await;
                        let _ = s.send(Message::Text(json.into())).await;
                        return;
                    }
                }
            }
        }
    }
}
