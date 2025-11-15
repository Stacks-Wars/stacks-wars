use serde::Deserialize;
use uuid::Uuid;

/// WebSocket connection query parameters
#[derive(Deserialize)]
pub struct WsQueryParams {
    pub user_id: Uuid,
}

//use crate::models::redis::{LobbyStatus, player_state::PlayerStatus};

///// Lobby listing query parameters
//#[derive(Deserialize)]
//pub struct LobbyQuery {
//    pub lobby_state: Option<String>,
//    pub player_state: Option<String>,
//    pub page: Option<u32>,
//    pub limit: Option<u32>,
//}

///// Player state filter query
//#[derive(Deserialize)]
//pub struct PlayerQuery {
//    pub player_state: Option<String>,
//}

///// Parse comma-separated lobby states from query string
//pub fn parse_lobby_states(state_param: Option<String>) -> Option<Vec<LobbyStatus>> {
//    state_param
//        .map(|s| {
//            s.split(',')
//                .filter_map(|state_str| {
//                    let trimmed = state_str.trim();
//                    match trimmed {
//                        "waiting" => Some(LobbyStatus::Waiting),
//                        "starting" => Some(LobbyStatus::Starting),
//                        "inProgress" => Some(LobbyStatus::InProgress),
//                        "finished" => Some(LobbyStatus::Finished),
//                        _ => {
//                            tracing::warn!("Invalid state filter: {}", trimmed);
//                            None
//                        }
//                    }
//                })
//                .collect()
//        })
//        .filter(|states: &Vec<LobbyStatus>| !states.is_empty())
//}

///// Parse player state from query string
//pub fn parse_player_state(param: Option<String>) -> Option<PlayerStatus> {
//    param.and_then(|s| match s.to_lowercase().as_str() {
//        "joined" => Some(PlayerStatus::Joined),
//        "notjoined" | "notJoined" => Some(PlayerStatus::NotJoined),
//        other => {
//            tracing::warn!("Invalid player_status filter: {}", other);
//            None
//        }
//    })
//}
