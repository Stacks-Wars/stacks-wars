use serde::Deserialize;
use uuid::Uuid;

use crate::models::enums::{LobbyState, PlayerState};

/// WebSocket connection query parameters
#[derive(Deserialize)]
pub struct WsQueryParams {
    pub user_id: Uuid,
}

/// Lobby listing query parameters
#[derive(Deserialize)]
pub struct LobbyQuery {
    pub lobby_state: Option<String>,
    pub player_state: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// Player state filter query
#[derive(Deserialize)]
pub struct PlayerQuery {
    pub player_state: Option<String>,
}

/// Parse comma-separated lobby states from query string
pub fn parse_lobby_states(state_param: Option<String>) -> Option<Vec<LobbyState>> {
    state_param
        .map(|s| {
            s.split(',')
                .filter_map(|state_str| {
                    let trimmed = state_str.trim();
                    match trimmed {
                        "waiting" => Some(LobbyState::Waiting),
                        "starting" => Some(LobbyState::Starting),
                        "inProgress" => Some(LobbyState::InProgress),
                        "finished" => Some(LobbyState::Finished),
                        _ => {
                            tracing::warn!("Invalid state filter: {}", trimmed);
                            None
                        }
                    }
                })
                .collect()
        })
        .filter(|states: &Vec<LobbyState>| !states.is_empty())
}

/// Parse player state from query string
pub fn parse_player_state(param: Option<String>) -> Option<PlayerState> {
    param.and_then(|s| match s.to_lowercase().as_str() {
        "joined" => Some(PlayerState::Joined),
        "notjoined" | "notJoined" => Some(PlayerState::NotJoined),
        other => {
            tracing::warn!("Invalid player_state filter: {}", other);
            None
        }
    })
}
