// Hub helpers for broadcasting messages to connected clients
use crate::state::AppState;
use crate::ws::message::BroadcastMessage;
use axum::extract::ws::Message;
use futures::SinkExt;
use tokio::task::JoinHandle;
use uuid::Uuid;

/// Broadcast a message to every active connection in `AppState`.
pub async fn broadcast_to_all<M: BroadcastMessage>(state: &AppState, msg: &M) {
    if let Ok(json) = msg.to_json() {
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
pub async fn broadcast_to_lobby<M: BroadcastMessage>(state: &AppState, lobby_id: Uuid, msg: &M) {
    // Use the lobby_connections index to find all connection ids in the lobby,
    // then send the message to each connection directly.
    if let Ok(json) = msg.to_json() {
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
pub async fn broadcast_to_user_ids<M: BroadcastMessage>(
    state: &AppState,
    lobby_id: Uuid,
    user_ids: &[Uuid],
    msg: &M,
) {
    if let Ok(json) = msg.to_json() {
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
pub async fn broadcast_to_user<M: BroadcastMessage>(
    state: &AppState,
    lobby_id: Uuid,
    user_id: Uuid,
    msg: &M,
) {
    if let Ok(json) = msg.to_json() {
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

/// Broadcast to all participants (players) in a lobby by fetching
/// their user IDs from Redis and calling broadcast_to_user_ids.
pub async fn broadcast_to_lobby_participants<M: BroadcastMessage>(
    state: &AppState,
    lobby_id: Uuid,
    msg: &M,
) {
    use crate::db::player_state::PlayerStateRepository;

    let player_repo = PlayerStateRepository::new(state.redis.clone());

    // Get all player user IDs
    if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
        let user_ids: Vec<Uuid> = players.into_iter().map(|p| p.user_id).collect();

        // Broadcast to all player user IDs
        if !user_ids.is_empty() {
            broadcast_to_user_ids(state, lobby_id, &user_ids, msg).await;
        }
    }
}
