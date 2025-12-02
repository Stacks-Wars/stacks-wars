use axum::{
    extract::{ConnectInfo, Path, Query, State, WebSocketUpgrade, ws::WebSocket},
    http::StatusCode,
    response::IntoResponse,
};
use futures::StreamExt;
use std::net::SocketAddr;

use crate::{
    db::{
        chat::get::get_chat_history,
        lobby::get::{get_lobby_info, get_lobby_players},
        user::get::get_user_by_id,
    },
    models::{
        chat::ChatServerMessage,
        game::{LobbyState, Player, PlayerState, WsQueryParams},
    },
    state::{AppState, ChatConnectionInfoMap, RedisClient},
    ws::handlers::chat::{message_handler, utils::*},
};
use axum::extract::ws::{CloseFrame, Message};
use uuid::Uuid;

pub async fn chat_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQueryParams>,
    Path(lobby_id): Path<Uuid>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::debug!("New chat WebSocket connection from {}", addr);

    let player_id = query.user_id;
    let redis = state.redis.clone();
    let chat_connections = state.chat_connections.clone();

    // Check if lobby exists and get lobby state
    let lobby_info = get_lobby_info(lobby_id, redis.clone())
        .await
        .map_err(|e| e.to_response())?;

    // If game is finished, close connection immediately
    if lobby_info.state == LobbyState::Finished {
        tracing::debug!(
            "Player {} trying to connect to chat while game is finished",
            player_id
        );

        return Ok(ws.on_upgrade(move |mut socket| async move {
            let close_frame = CloseFrame {
                code: axum::extract::ws::close_code::NORMAL,
                reason: "finished".into(),
            };

            let _ = socket.send(Message::Close(Some(close_frame))).await;
        }));
    }

    // Get user info to create player object
    let user = get_user_by_id(player_id, redis.clone())
        .await
        .map_err(|e| e.to_response())?;

    let player = Player {
        id: user.id,
        state: PlayerState::NotJoined,
        rank: None,
        used_words: None,
        tx_id: None,
        claim: None,
        prize: None,
        last_ping: None,
        user: Some(user.clone()),
    };

    Ok(ws.on_upgrade(move |socket| {
        handle_chat_socket(socket, lobby_id, player, chat_connections, redis)
    }))
}

async fn handle_chat_socket(
    socket: WebSocket,
    lobby_id: Uuid,
    player: Player,
    chat_connections: ChatConnectionInfoMap,
    redis: RedisClient,
) {
    let (sender, receiver) = socket.split();

    store_chat_connection_and_send_queued_messages(
        lobby_id,
        player.id,
        sender,
        &chat_connections,
        &redis,
    )
    .await;

    // Check if player is in the lobby and send permission status
    let is_lobby_member =
        match get_lobby_players(lobby_id, Some(PlayerState::Joined), redis.clone()).await {
            Ok(players) => players.iter().any(|p| p.id == player.id),
            Err(e) => {
                tracing::error!("Failed to check lobby membership: {}", e);
                false
            }
        };

    let permit_msg = ChatServerMessage::PermitChat {
        allowed: is_lobby_member,
    };
    send_chat_message_to_player(player.id, &permit_msg, &chat_connections).await;

    // If player is a lobby member, send chat history from Redis
    if is_lobby_member {
        match get_chat_history(lobby_id, &redis).await {
            Ok(chat_history) => {
                if !chat_history.is_empty() {
                    let history_msg = ChatServerMessage::ChatHistory {
                        messages: chat_history,
                    };
                    send_chat_message_to_player(player.id, &history_msg, &chat_connections).await;
                }
            }
            Err(e) => {
                tracing::error!("Failed to load chat history from Redis: {}", e);
            }
        }
    }

    message_handler::handle_incoming_chat_messages(
        receiver,
        lobby_id,
        &player,
        &chat_connections,
        redis.clone(),
    )
    .await;

    remove_chat_connection(player.id, &chat_connections).await;
}
