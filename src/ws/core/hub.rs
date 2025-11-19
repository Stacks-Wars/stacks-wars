// Hub helpers for broadcasting messages to connected clients
use crate::models::redis::keys::{KeyPart, RedisKey};
use crate::state::AppState;
use axum::extract::ws::Message;
use futures::SinkExt;
use redis::AsyncCommands;
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
    let mut player_ids: Vec<Uuid> = Vec::new();
    if let Ok(mut conn) = state.redis.get().await {
        let pattern = RedisKey::lobby_player(lobby_id, KeyPart::Wildcard);
        if let Ok(keys) = conn.keys::<_, Vec<String>>(pattern).await {
            for key in keys {
                if let Some(part) = key.split(':').last() {
                    if let Ok(id) = Uuid::parse_str(part) {
                        player_ids.push(id);
                    }
                }
            }
        }
    }

    if !player_ids.is_empty() {
        broadcast_to_user_ids(state, &player_ids, msg).await;
    }
}

/// Broadcast to a specific list of user ids. Lookup connections in `state.connections`.
pub async fn broadcast_to_user_ids<M: Serialize>(state: &AppState, user_ids: &[Uuid], msg: &M) {
    if let Ok(json) = serde_json::to_string(msg) {
        let conns = state.connections.lock().await;
        for uid in user_ids.iter() {
            if let Some(conn) = conns.get(uid) {
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

/// Send a message to a single connected user (if present).
pub async fn send_to_user<M: Serialize>(state: &AppState, user_id: Uuid, msg: &M) {
    if let Ok(json) = serde_json::to_string(msg) {
        let conns = state.connections.lock().await;
        if let Some(conn) = conns.get(&user_id) {
            let mut s = conn.sender.lock().await;
            let _ = s.send(Message::Text(json.into())).await;
        }
    }
}
