// Consolidated WebSocket broadcasting functions
use crate::db::{
    game::GameRepository, lobby::LobbyRepository, lobby_state::LobbyStateRepository,
    user::UserRepository,
};
use crate::models::{LobbyExtended, LobbyInfo};
use crate::state::AppState;
use crate::ws::core::message::BroadcastMessage;
use crate::ws::lobby::LobbyServerMessage;
use crate::ws::room::messages::GameMessage;
use axum::extract::ws::Message;
use futures::SinkExt;
use uuid::Uuid;

/// Broadcast lobby update to lobby list subscribers
pub async fn broadcast_lobby_update(state: AppState, lobby_id: Uuid) {
    tokio::spawn(async move {
        let lobby_repo = LobbyRepository::new(state.postgres.clone());
        let lobby_state_repo = LobbyStateRepository::new(state.redis.clone());

        if let Ok(lobby) = lobby_repo.find_by_id(lobby_id).await {
            let game_repo = GameRepository::new(state.postgres.clone());
            let user_repo = UserRepository::new(state.postgres.clone());

            let (game, creator, lobby_state) = tokio::join!(
                game_repo.find_by_id(lobby.game_id),
                user_repo.find_by_id(lobby.creator_id),
                lobby_state_repo.get_state(lobby_id),
            );

            if let (Ok(game), Ok(creator), Ok(lobby_state)) = (game, creator, lobby_state) {
                let lobby_extended = LobbyExtended::from_parts(lobby, lobby_state);
                let lobby_info = LobbyInfo {
                    lobby: lobby_extended,
                    game,
                    creator,
                };

                let _ = broadcast_lobby_list(
                    &state,
                    &LobbyServerMessage::LobbyUpdated { lobby: lobby_info },
                )
                .await;
            }
        }
    });
}

/// Broadcast lobby creation to lobby list subscribers
pub async fn broadcast_lobby_creation(
    state: AppState,
    lobby_id: Uuid,
    game_id: Uuid,
    creator_id: Uuid,
) {
    tokio::spawn(async move {
        let lobby_repo = LobbyRepository::new(state.postgres.clone());
        let game_repo = GameRepository::new(state.postgres.clone());
        let user_repo = UserRepository::new(state.postgres.clone());
        let lobby_state_repo = LobbyStateRepository::new(state.redis.clone());

        let (lobby, game, creator, lobby_state) = tokio::join!(
            lobby_repo.find_by_id(lobby_id),
            game_repo.find_by_id(game_id),
            user_repo.find_by_id(creator_id),
            lobby_state_repo.get_state(lobby_id),
        );

        if let (Ok(lobby), Ok(lobby_state), Ok(game), Ok(creator)) =
            (lobby, lobby_state, game, creator)
        {
            let lobby_extended = LobbyExtended::from_parts(lobby, lobby_state);
            let lobby_info = LobbyInfo {
                lobby: lobby_extended,
                game,
                creator,
            };

            let _ = broadcast_lobby_list(&state, &LobbyServerMessage::LobbyCreated { lobby_info })
                .await;
        }
    });
}

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

/// Broadcast to all lobby list connections (including those with status filters)
pub async fn broadcast_lobby_list<M: BroadcastMessage>(state: &AppState, msg: &M) {
    if let Ok(json) = msg.to_json() {
        let indices = state.indices.lock().await;
        let conns = state.connections.lock().await;

        // Collect all unique connection IDs from lobby-related context keys
        let mut sent_to: std::collections::HashSet<Uuid> = std::collections::HashSet::new();

        // Include connections without status filter ("lobby" key)
        if let Some(conn_ids) = indices.get_context_connections("lobby") {
            for conn_id in conn_ids.iter() {
                sent_to.insert(*conn_id);
            }
        }

        // Include connections with status filters ("lobby:*" keys)
        for (context_key, conn_ids) in indices.by_context.iter() {
            if context_key.starts_with("lobby:") {
                for conn_id in conn_ids.iter() {
                    sent_to.insert(*conn_id);
                }
            }
        }

        // Send to all unique connections
        for conn_id in sent_to {
            if let Some(conn) = conns.get(&conn_id) {
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

/// Broadcast a game-specific message to a specific user.
///
/// This wraps the message in the GameMessage wrapper format:
/// ```json
/// { "game": { "type": "...", ...fields } }
/// ```
///
/// The payload should be a serialized game event with a "type" field
pub async fn broadcast_game_message_to_user(
    state: &AppState,
    user_id: Uuid,
    payload: serde_json::Value,
) {
    let game_msg = GameMessage::new(payload);

    if let Ok(json) = serde_json::to_string(&game_msg) {
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

/// Broadcast a game-specific message to all connections in a lobby room.
///
/// This wraps the message in the GameMessage wrapper format:
/// ```json
/// { "game": { "type": "...", ...fields } }
/// ```
///
/// The payload should be a serialized game event with a "type" field
pub async fn broadcast_game_message(
    state: &AppState,
    lobby_id: Uuid,
    payload: serde_json::Value,
) {
    let game_msg = GameMessage::new(payload);

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

/// Broadcast a game-specific message to all connections in a lobby room except a specific user.
///
/// This wraps the message in the GameMessage wrapper format:
/// ```json
/// { "game": { "type": "...", ...fields } }
/// ```
///
/// The payload should be a serialized game event with a "type" field
pub async fn broadcast_game_message_to_room_except(
    state: &AppState,
    lobby_id: Uuid,
    except_user_id: Uuid,
    payload: serde_json::Value,
) {
    let game_msg = GameMessage::new(payload);

    if let Ok(json) = serde_json::to_string(&game_msg) {
        let indices = state.indices.lock().await;

        if let Some(conn_ids) = indices.get_lobby_connections(&lobby_id) {
            let conns = state.connections.lock().await;

            // Get connection IDs for the user to exclude
            let excluded_conn_ids: std::collections::HashSet<Uuid> = indices
                .get_user_connections(&except_user_id)
                .map(|ids| ids.iter().copied().collect())
                .unwrap_or_default();

            for conn_id in conn_ids.iter() {
                // Skip connections belonging to the excluded user
                if excluded_conn_ids.contains(conn_id) {
                    continue;
                }

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
