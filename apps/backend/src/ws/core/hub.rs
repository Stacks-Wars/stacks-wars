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
    if let Ok(json) = msg.to_json() {
        let indices = state.indices.lock().await;

        // Get all connection IDs for this lobby via index (O(1) lookup)
        if let Some(conn_ids) = indices.get_lobby_connections(&lobby_id) {
            let conns = state.connections.lock().await;

            for conn_id in conn_ids.iter() {
                if let Some(conn) = conns.get(conn_id) {
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

/// Broadcast to a specific list of user ids.
pub async fn broadcast_to_user_ids<M: BroadcastMessage>(
    state: &AppState,
    user_ids: &[Uuid],
    msg: &M,
) {
    if let Ok(json) = msg.to_json() {
        let indices = state.indices.lock().await;
        let conns = state.connections.lock().await;

        for user_id in user_ids.iter() {
            // Use index for O(1) lookup per user
            if let Some(conn_ids) = indices.get_user_connections(user_id) {
                for conn_id in conn_ids.iter() {
                    if let Some(conn) = conns.get(conn_id) {
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

/// Send a message to all connections for a specific user.
pub async fn broadcast_to_user<M: BroadcastMessage>(state: &AppState, user_id: Uuid, msg: &M) {
    if let Ok(json) = msg.to_json() {
        let indices = state.indices.lock().await;
        let conns = state.connections.lock().await;

        // Use index for O(1) lookup
        if let Some(conn_ids) = indices.get_user_connections(&user_id) {
            for conn_id in conn_ids.iter() {
                if let Some(conn) = conns.get(conn_id) {
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

/// Broadcast to all participants (players) in a lobby by fetching
/// their user IDs from Redis and using efficient index lookups.
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
            broadcast_to_user_ids(state, &user_ids, msg).await;
        }
    }
}
