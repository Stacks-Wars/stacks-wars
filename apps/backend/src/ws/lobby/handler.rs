// Lobby WebSocket handler - manages lobby list browsing with status filtering
use axum::{
    extract::{
        Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures::stream::StreamExt;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db::{lobby::LobbyRepository, lobby_state::LobbyStateRepository},
    models::{LobbyExtended, LobbyStatus},
    state::{AppState, ConnectionContext, ConnectionInfo},
    ws::{
        core::manager,
        lobby::{LobbyClientMessage, LobbyServerMessage},
    },
};

#[derive(Debug, Deserialize)]
pub struct LobbyQueryParams {
    #[serde(default)]
    pub status: Option<String>, // Comma-separated: "waiting,starting"
}

/// WebSocket handler for lobby list connections
pub async fn lobby_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<LobbyQueryParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, params, state))
}

async fn handle_socket(socket: WebSocket, params: LobbyQueryParams, state: AppState) {
    let (sender, mut receiver) = socket.split();
    let connection_id = Uuid::new_v4();

    // Parse status filter from query params
    let status_strings = parse_status_filter(&params.status);

    // Register connection with status-based context
    let conn = Arc::new(ConnectionInfo {
        connection_id,
        user_id: None, // Lobby browsing doesn't require authentication
        context: ConnectionContext::Lobby(status_strings.clone()),
        sender: Arc::new(tokio::sync::Mutex::new(sender)),
    });

    manager::register_connection(&state, connection_id, Arc::clone(&conn)).await;

    // Send initial lobby list
    let lobby_repo = LobbyRepository::new(state.postgres.clone());
    let lobby_state_repo = LobbyStateRepository::new(state.redis.clone());
    let status_filter = parse_status_enum(&status_strings);
    let status_filter_opt = if status_filter.is_empty() {
        None
    } else {
        Some(status_filter)
    };
    send_lobby_list(
        &conn,
        &lobby_repo,
        &lobby_state_repo,
        &status_filter_opt,
        0,
        12,
    )
    .await;

    // Message loop
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(lobby_msg) = serde_json::from_str::<LobbyClientMessage>(&text) {
                    handle_message(
                        lobby_msg,
                        &conn,
                        &state,
                        &lobby_repo,
                        &lobby_state_repo,
                        connection_id,
                    )
                    .await;
                }
            }
            Ok(Message::Close(_)) => break,
            _ => {}
        }
    }

    // Cleanup
    manager::unregister_connection(&state, &connection_id).await;
}

async fn handle_message(
    msg: LobbyClientMessage,
    conn: &Arc<ConnectionInfo>,
    state: &AppState,
    lobby_repo: &LobbyRepository,
    lobby_state_repo: &LobbyStateRepository,
    connection_id: Uuid,
) {
    match msg {
        LobbyClientMessage::Subscribe { status, limit } => {
            // User wants to change their status filter
            if let Some(new_statuses) = status {
                // Parse new status filter
                let status_strings: Vec<String> =
                    new_statuses.iter().map(|s| status_to_string(s)).collect();

                // Unregister old connection
                manager::unregister_connection(state, &connection_id).await;

                // Create new connection with updated context
                let new_conn = Arc::new(ConnectionInfo {
                    connection_id,
                    user_id: conn.user_id,
                    context: ConnectionContext::Lobby(Some(status_strings.clone())),
                    sender: conn.sender.clone(),
                });

                // Register with new context
                manager::register_connection(state, connection_id, new_conn).await;

                // Send updated lobby list
                let status_filter = Some(new_statuses);
                send_lobby_list(conn, lobby_repo, lobby_state_repo, &status_filter, 0, limit).await;
            } else {
                // No filter - send all lobbies
                send_lobby_list(conn, lobby_repo, lobby_state_repo, &None, 0, limit).await;
            }
        }
        LobbyClientMessage::LoadMore { offset } => {
            // Get current filter from connection context
            let status_filter_vec = match &conn.context {
                ConnectionContext::Lobby(opt_strings) => parse_status_enum(opt_strings),
                _ => vec![],
            };
            let status_filter_opt = if status_filter_vec.is_empty() {
                None
            } else {
                Some(status_filter_vec)
            };
            send_lobby_list(
                conn,
                lobby_repo,
                lobby_state_repo,
                &status_filter_opt,
                offset,
                12,
            )
            .await;
        }
    }
}

async fn send_lobby_list(
    conn: &Arc<ConnectionInfo>,
    lobby_repo: &LobbyRepository,
    lobby_state_repo: &LobbyStateRepository,
    status_filter: &Option<Vec<LobbyStatus>>,
    offset: usize,
    limit: usize,
) {
    match fetch_lobbies(lobby_repo, lobby_state_repo, status_filter, offset, limit).await {
        Ok((lobbies, total)) => {
            let _ = manager::send_to_connection(
                conn,
                &LobbyServerMessage::LobbyList { lobbies, total },
            )
            .await;
        }
        Err(e) => {
            let _ = manager::send_to_connection(
                conn,
                &LobbyServerMessage::Error {
                    code: "FETCH_FAILED".to_string(),
                    message: e,
                },
            )
            .await;
        }
    }
}

async fn fetch_lobbies(
    lobby_repo: &LobbyRepository,
    lobby_state_repo: &LobbyStateRepository,
    status_filter: &Option<Vec<LobbyStatus>>,
    offset: usize,
    limit: usize,
) -> Result<(Vec<LobbyExtended>, usize), String> {
    use crate::models::LobbyState;

    // Fetch lobbies with joined user and game data using optimized query
    let lobbies_with_joins = if let Some(statuses) = status_filter {
        lobby_repo
            .find_by_statuses_with_joins(statuses, offset, limit)
            .await
            .map_err(|e| format!("Failed to fetch lobbies: {}", e))?
    } else {
        lobby_repo
            .find_all_with_joins(offset, limit)
            .await
            .map_err(|e| format!("Failed to fetch lobbies: {}", e))?
    };

    tracing::debug!("Fetched {} lobbies with joins", lobbies_with_joins.len());
    let total = lobbies_with_joins.len();

    // Fetch lobby states from Redis for all lobbies using pipeline (single round-trip)
    let lobby_ids: Vec<uuid::Uuid> = lobbies_with_joins.iter().map(|l| l.id).collect();

    let states_batch = lobby_state_repo
        .get_states_batch(&lobby_ids)
        .await
        .map_err(|e| format!("Failed to fetch lobby states: {}", e))?;

    // Construct LobbyExtended objects
    let mut extended_lobbies = Vec::new();
    for (lobby_with_joins, (lobby_id, state_opt)) in
        lobbies_with_joins.into_iter().zip(states_batch.into_iter())
    {
        // Use the lobby state from Redis, or create a default state if not found
        let state = state_opt.unwrap_or_else(|| LobbyState::new(lobby_id));

        let (creator_wallet, creator_username, creator_display_name) =
            lobby_with_joins.creator_info();
        let (game_image_url, game_min_players, game_max_players) = lobby_with_joins.game_info();

        let extended = LobbyExtended::from_parts(
            lobby_with_joins.to_lobby(),
            state,
            creator_wallet,
            creator_username,
            creator_display_name,
            game_image_url,
            game_min_players,
            game_max_players,
        );
        extended_lobbies.push(extended);
    }

    tracing::debug!("Constructed {} extended lobbies", extended_lobbies.len());
    Ok((extended_lobbies, total))
}

fn parse_status_filter(param: &Option<String>) -> Option<Vec<String>> {
    param.as_ref().map(|s| {
        s.split(',')
            .map(|part| part.trim().to_lowercase())
            .filter(|part| {
                matches!(
                    part.as_str(),
                    "waiting" | "starting" | "in_progress" | "inprogress" | "finished"
                )
            })
            .collect()
    })
}

fn parse_status_enum(strings: &Option<Vec<String>>) -> Vec<LobbyStatus> {
    strings
        .as_ref()
        .map(|strings| {
            strings
                .iter()
                .filter_map(|s| match s.as_str() {
                    "waiting" => Some(LobbyStatus::Waiting),
                    "starting" => Some(LobbyStatus::Starting),
                    "in_progress" | "inprogress" => Some(LobbyStatus::InProgress),
                    "finished" => Some(LobbyStatus::Finished),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn status_to_string(status: &LobbyStatus) -> String {
    match status {
        LobbyStatus::Waiting => "waiting".to_string(),
        LobbyStatus::Starting => "starting".to_string(),
        LobbyStatus::InProgress => "in_progress".to_string(),
        LobbyStatus::Finished => "finished".to_string(),
    }
}
