use chrono::Utc;
use redis::AsyncCommands;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        User,
        lobby::JoinState,
        redis::{KeyPart, RedisKey},
    },
    state::RedisClient,
};

const MAX_JOIN_REQUESTS: isize = 20;

#[derive(Debug, Clone)]
pub struct JoinRequestEntry {
    pub user: User,
    pub state: JoinState,
    pub created_at: i64, // timestamp for sorting
}

impl JoinRequestEntry {
    pub fn to_redis_hash(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("state".into(), format!("{:?}", self.state));
        map.insert("created_at".into(), self.created_at.to_string());
        // Store user data for quick access
        map.insert("user_id".into(), self.user.id.to_string());
        map.insert("wallet_address".into(), self.user.wallet_address.clone());
        if let Some(ref username) = self.user.username {
            map.insert("username".into(), username.clone());
        }
        if let Some(ref display_name) = self.user.display_name {
            map.insert("display_name".into(), display_name.clone());
        }
        map.insert("wars_point".into(), self.user.wars_point.to_string());
        map
    }

    pub fn from_redis_hash(
        user_id: Uuid,
        data: &HashMap<String, String>,
    ) -> Result<Self, AppError> {
        let state_str = data
            .get("state")
            .ok_or_else(|| AppError::Deserialization("Missing state".into()))?;

        let state = match state_str.as_str() {
            "Pending" => JoinState::Pending,
            "Allowed" => JoinState::Allowed,
            "Rejected" => JoinState::Rejected,
            other => {
                return Err(AppError::Deserialization(format!(
                    "Invalid join state: {}",
                    other
                )));
            }
        };

        let created_at = data
            .get("created_at")
            .and_then(|v| v.parse().ok())
            .unwrap_or(Utc::now().timestamp());

        let user = User {
            id: user_id,
            wallet_address: data.get("wallet_address").cloned().unwrap_or_default(),
            username: data.get("username").cloned(),
            display_name: data.get("display_name").cloned(),
            wars_point: data
                .get("wars_point")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0),
        };

        Ok(JoinRequestEntry {
            user,
            state,
            created_at,
        })
    }
}

pub async fn get_lobby_join_requests(
    lobby_id: Uuid,
    state_filter: Option<JoinState>,
    redis: RedisClient,
) -> Result<Vec<JoinRequestEntry>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let zset_key = RedisKey::lobby_join_requests(KeyPart::Id(lobby_id));

    // Get all user IDs from the sorted set (ordered by timestamp)
    let user_ids: Vec<String> = conn
        .zrange(&zset_key, 0, -1)
        .await
        .map_err(AppError::RedisCommandError)?;

    if user_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut requests = Vec::new();
    let mut expired_users = Vec::new();

    for user_id_str in user_ids {
        if let Ok(user_id) = Uuid::parse_str(&user_id_str) {
            let user_key =
                RedisKey::lobby_join_request_user(KeyPart::Id(lobby_id), KeyPart::Id(user_id));

            let user_data: HashMap<String, String> = conn
                .hgetall(&user_key)
                .await
                .map_err(AppError::RedisCommandError)?;

            if user_data.is_empty() {
                // Entry expired - mark for cleanup
                expired_users.push(user_id_str);
            } else if let Ok(entry) = JoinRequestEntry::from_redis_hash(user_id, &user_data) {
                // Apply state filter if provided
                let passes_filter = match &state_filter {
                    Some(filter_state) => entry.state == *filter_state,
                    None => true,
                };

                if passes_filter {
                    requests.push(entry);
                }
            }
        }
    }

    // Clean up expired entries from the sorted set
    if !expired_users.is_empty() {
        let _: () = conn
            .zrem(&zset_key, &expired_users)
            .await
            .map_err(AppError::RedisCommandError)?;

        tracing::info!(
            "Cleaned up {} expired join request references from lobby {}",
            expired_users.len(),
            lobby_id
        );
    }

    Ok(requests)
}

pub async fn get_player_join_request(
    lobby_id: Uuid,
    user_id: Uuid,
    redis: RedisClient,
) -> Result<Option<JoinRequestEntry>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let user_key = RedisKey::lobby_join_request_user(KeyPart::Id(lobby_id), KeyPart::Id(user_id));

    let user_data: HashMap<String, String> = conn
        .hgetall(&user_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    if user_data.is_empty() {
        // If user data is empty (expired), also clean up the sorted set
        let zset_key = RedisKey::lobby_join_requests(KeyPart::Id(lobby_id));
        let _: () = conn
            .zrem(&zset_key, user_id.to_string())
            .await
            .map_err(AppError::RedisCommandError)?;
        return Ok(None);
    }

    let entry = JoinRequestEntry::from_redis_hash(user_id, &user_data)?;
    Ok(Some(entry))
}

pub async fn update_join_request(
    lobby_id: Uuid,
    user: User,
    new_state: JoinState,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let zset_key = RedisKey::lobby_join_requests(KeyPart::Id(lobby_id));
    let user_key = RedisKey::lobby_join_request_user(KeyPart::Id(lobby_id), KeyPart::Id(user.id));

    let now = Utc::now().timestamp();

    // Add user_id to the sorted set with current timestamp as score
    let _: () = conn
        .zadd(&zset_key, user.id.to_string(), now)
        .await
        .map_err(AppError::RedisCommandError)?;

    // Keep only the most recent 20 entries (remove oldest if over limit)
    let count: isize = conn
        .zcard(&zset_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    if count > MAX_JOIN_REQUESTS {
        // Remove the oldest entries (lowest scores)
        let to_remove = count - MAX_JOIN_REQUESTS;
        let oldest_members: Vec<String> = conn
            .zrange(&zset_key, 0, to_remove - 1)
            .await
            .map_err(AppError::RedisCommandError)?;

        if !oldest_members.is_empty() {
            // Remove from sorted set
            let _: () = conn
                .zrem(&zset_key, &oldest_members)
                .await
                .map_err(AppError::RedisCommandError)?;

            // Remove their individual hash entries
            for member in &oldest_members {
                if let Ok(old_user_id) = Uuid::parse_str(member) {
                    let old_user_key = RedisKey::lobby_join_request_user(
                        KeyPart::Id(lobby_id),
                        KeyPart::Id(old_user_id),
                    );
                    let _: () = conn
                        .del(&old_user_key)
                        .await
                        .map_err(AppError::RedisCommandError)?;
                }
            }

            tracing::info!(
                "Removed {} oldest join requests from lobby {} to maintain limit of {}",
                oldest_members.len(),
                lobby_id,
                MAX_JOIN_REQUESTS
            );
        }
    }

    // Create entry with new state and timestamp
    let entry = JoinRequestEntry {
        user,
        state: new_state,
        created_at: now,
    };

    let entry_data = entry.to_redis_hash();
    let fields: Vec<(&str, &str)> = entry_data
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    // Store the full entry data with 7-day expiry
    let _: () = conn
        .hset_multiple(&user_key, &fields)
        .await
        .map_err(AppError::RedisCommandError)?;

    let _: () = conn
        .expire(&user_key, 604800) // 7 days
        .await
        .map_err(AppError::RedisCommandError)?;

    tracing::info!(
        "Updated join request for user {} in lobby {} with state {:?} (position in queue)",
        entry.user.id,
        lobby_id,
        entry.state
    );

    Ok(())
}

pub async fn remove_join_request(
    lobby_id: Uuid,
    user_id: Uuid,
    redis: RedisClient,
) -> Result<(), AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let zset_key = RedisKey::lobby_join_requests(KeyPart::Id(lobby_id));
    let user_key = RedisKey::lobby_join_request_user(KeyPart::Id(lobby_id), KeyPart::Id(user_id));

    // Remove from sorted set
    let _: () = conn
        .zrem(&zset_key, user_id.to_string())
        .await
        .map_err(AppError::RedisCommandError)?;

    // Remove the user's entry hash
    let _: () = conn
        .del(&user_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    tracing::info!(
        "Removed join request for user {} from lobby {}",
        user_id,
        lobby_id
    );

    Ok(())
}

// Helper function to get join request count for a lobby
pub async fn _get_join_request_count(
    lobby_id: Uuid,
    redis: RedisClient,
) -> Result<usize, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let zset_key = RedisKey::lobby_join_requests(KeyPart::Id(lobby_id));

    let count: isize = conn
        .zcard(&zset_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    Ok(count as usize)
}

// Helper function to get oldest join request timestamp
pub async fn _get_oldest_join_request_time(
    lobby_id: Uuid,
    redis: RedisClient,
) -> Result<Option<i64>, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let zset_key = RedisKey::lobby_join_requests(KeyPart::Id(lobby_id));

    // Get the member with the lowest score (oldest)
    let oldest: Vec<(String, f64)> = conn
        .zrange_withscores(&zset_key, 0, 0)
        .await
        .map_err(AppError::RedisCommandError)?;

    if let Some((_, timestamp)) = oldest.first() {
        Ok(Some(*timestamp as i64))
    } else {
        Ok(None)
    }
}

pub async fn remove_all_lobby_join_requests(
    lobby_id: Uuid,
    redis: RedisClient,
) -> Result<usize, AppError> {
    let mut conn = redis.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => AppError::RedisCommandError(err),
        bb8::RunError::TimedOut => AppError::RedisPoolError("Redis connection timed out".into()),
    })?;

    let zset_key = RedisKey::lobby_join_requests(KeyPart::Id(lobby_id));

    // Get all user IDs from the sorted set
    let user_ids: Vec<String> = conn
        .zrange(&zset_key, 0, -1)
        .await
        .map_err(AppError::RedisCommandError)?;

    let mut deleted_count = 0;

    // Delete individual user hash entries
    for user_id_str in &user_ids {
        if let Ok(user_id) = Uuid::parse_str(user_id_str) {
            let user_key =
                RedisKey::lobby_join_request_user(KeyPart::Id(lobby_id), KeyPart::Id(user_id));

            let deleted: usize = conn
                .del(&user_key)
                .await
                .map_err(AppError::RedisCommandError)?;

            deleted_count += deleted;
        }
    }

    // Delete the main sorted set
    let _: () = conn
        .del(&zset_key)
        .await
        .map_err(AppError::RedisCommandError)?;

    tracing::info!(
        "Removed {} join request entries and main sorted set for lobby {}",
        deleted_count,
        lobby_id
    );

    Ok(deleted_count)
}
