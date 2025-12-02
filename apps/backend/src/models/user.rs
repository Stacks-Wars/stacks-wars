use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::game::Player;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub wallet_address: String,
    pub wars_point: f64,

    pub username: Option<String>,
    pub display_name: Option<String>,
}

impl From<Player> for User {
    fn from(player: Player) -> Self {
        player.into_user()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,    // user ID
    pub wallet: String, // wallet address
    pub exp: usize,     // expiration time
}
