// JoinRequestRepository: runtime Redis helpers for join request lifecycle

mod create;
mod delete;
mod read;
mod update;

use crate::state::RedisClient;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum JoinRequestState {
    Pending,
    Accepted,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JoinRequest {
    pub user_id: Uuid,
    pub state: JoinRequestState,
    pub wallet_address: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub trust_rating: f64,
    pub is_creator: bool,
    pub created_at: i64,
}

/// JoinRequestRepository (wraps the Redis client).
#[derive(Clone)]
pub struct JoinRequestRepository {
    pub(crate) redis: RedisClient,
}

impl JoinRequestRepository {
    /// Create a new `JoinRequestRepository`.
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }
}
