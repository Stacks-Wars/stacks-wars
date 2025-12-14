// Consolidated WebSocket broadcasting functions
use crate::state::AppState;
use crate::ws::core::message::BroadcastMessage;
use axum::extract::ws::Message;
use futures::SinkExt;
use uuid::Uuid;

/// Send a message to a single connection
pub async fn send<M: BroadcastMessage>(state: &AppState, connection_id: Uuid, msg: &M) {
    if let Ok(json) = msg.to_json() {
        let conns = state.connections.lock().await;
        if let Some(conn) = conns.get(&connection_id) {
            let sender = conn.sender.clone();
            tokio::spawn(async move {
                let mut s = sender.lock().await;
                let _ = s.send(Message::Text(json.into())).await;
            });
        }
    }
}

/// Broadcast to all connections
pub async fn broadcast_all<M: BroadcastMessage>(state: &AppState, msg: &M) {
    if let Ok(json) = msg.to_json() {
        let conns = state.connections.lock().await;

        for conn in conns.values() {
            let sender = conn.sender.clone();
            let json_clone = json.clone();
            tokio::spawn(async move {
                let mut s = sender.lock().await;
                let _ = s.send(Message::Text(json_clone.into())).await;
            });
        }
    }
}

/// Broadcast to all connections in a specific lobby room
pub async fn broadcast_room<M: BroadcastMessage>(state: &AppState, lobby_id: Uuid, msg: &M) {
    if let Ok(json) = msg.to_json() {
        let indices = state.indices.lock().await;

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

/// Broadcast to all connections for a specific user (multi-tab support)
pub async fn broadcast_user<M: BroadcastMessage>(state: &AppState, user_id: Uuid, msg: &M) {
    if let Ok(json) = msg.to_json() {
        let indices = state.indices.lock().await;
        let conns = state.connections.lock().await;

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

/// Broadcast to multiple users (batch operation)
pub async fn broadcast_users<M: BroadcastMessage>(state: &AppState, user_ids: &[Uuid], msg: &M) {
    if let Ok(json) = msg.to_json() {
        let indices = state.indices.lock().await;
        let conns = state.connections.lock().await;

        for user_id in user_ids {
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

/// Broadcast to all room participants (fetches player list from Redis)
pub async fn broadcast_room_participants<M: BroadcastMessage>(
    state: &AppState,
    lobby_id: Uuid,
    msg: &M,
) {
    use crate::db::player_state::PlayerStateRepository;

    let player_repo = PlayerStateRepository::new(state.redis.clone());

    if let Ok(players) = player_repo.get_all_in_lobby(lobby_id).await {
        let user_ids: Vec<Uuid> = players.into_iter().map(|p| p.user_id).collect();
        if !user_ids.is_empty() {
            broadcast_users(state, &user_ids, msg).await;
        }
    }
}

/// Broadcast to all lobby list connections
pub async fn broadcast_lobby_list<M: BroadcastMessage>(state: &AppState, msg: &M) {
    if let Ok(json) = msg.to_json() {
        let indices = state.indices.lock().await;

        if let Some(conn_ids) = indices.get_context_connections("lobby") {
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

/// Broadcast to lobby list connections filtered by status
pub async fn broadcast_lobby_by_status<M: BroadcastMessage>(
    state: &AppState,
    status: &str,
    msg: &M,
) {
    if let Ok(json) = msg.to_json() {
        let indices = state.indices.lock().await;

        // Get connections for this specific status context
        let context_key = format!("lobby:{}", status);
        if let Some(conn_ids) = indices.get_context_connections(&context_key) {
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

/// Broadcast a game-specific message to all connections in a lobby room.
/// 
/// This wraps the message with game identifier for frontend router.
pub async fn broadcast_game_message(
    state: &AppState,
    lobby_id: Uuid,
    game_path: &str,
    msg_type: &str,
    payload: serde_json::Value,
) {
    use crate::ws::room::messages::GameMessage;
    
    let game_msg = GameMessage::new(
        game_path.to_string(),
        msg_type.to_string(),
        payload,
    );
    
    if let Ok(json) = serde_json::to_string(&game_msg) {
        let indices = state.indices.lock().await;

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
