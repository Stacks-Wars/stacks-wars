use crate::models::User;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderBoard {
    pub user: User,
    pub win_rate: f64,
    pub rank: u64,
    pub total_match: u64,
    pub total_wins: u64,
    pub pnl: f64,
}
