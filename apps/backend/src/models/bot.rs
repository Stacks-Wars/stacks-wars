use serde::{Deserialize, Serialize};
use super::lobby::Lobby;

#[derive(Serialize, Deserialize)]
pub struct BotNewLobbyPayload {
    pub lobby: Lobby,
    pub creator_name: Option<String>,
    pub wallet_address: String,
}