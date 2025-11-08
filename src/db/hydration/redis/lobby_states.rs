//! Migrate lobby states from old LobbyInfo to new LobbyState

use crate::db::hydration::redis::get_all_lobby_ids;
use crate::db::lobby_state::LobbyStateRepository;
use crate::errors::AppError;
use crate::models::enums::LobbyState as LobbyStatus;
use crate::models::redis::LobbyState;
use crate::state::RedisClient;
use redis::AsyncCommands;
use std::collections::HashMap;
use uuid::Uuid;

/// Migrate lobby states from old structure to new structure
///
/// Transforms: `lobbies:{id}:info` (LobbyInfo) → `lobbies:{id}:state` (LobbyState)
///
/// Extracts only the runtime state fields:
/// - status
/// - participant_count
/// - created_at, updated_at, started_at, finished_at
/// - creator_last_ping
/// - tg_msg_id
///
/// Configuration fields (name, entry_amount, etc.) remain in PostgreSQL only.
pub async fn migrate_lobby_states(
    redis: &RedisClient,
    lobby_state_repo: &LobbyStateRepository,
    dry_run: bool,
) -> Result<usize, AppError> {
    let lobby_ids = get_all_lobby_ids(redis).await?;

    if lobby_ids.is_empty() {
        println!("   ℹ️  No lobbies found to migrate");
        return Ok(0);
    }

    println!("   Found {} lobbies to migrate", lobby_ids.len());

    let mut migrated_count = 0;
    let mut skipped_count = 0;
    let mut error_count = 0;

    for lobby_id_str in lobby_ids {
        let lobby_id = match Uuid::parse_str(&lobby_id_str) {
            Ok(id) => id,
            Err(_) => {
                println!("   ⚠️  Invalid lobby ID: {}", lobby_id_str);
                error_count += 1;
                continue;
            }
        };

        // Check if new state already exists
        if lobby_state_repo.exists(lobby_id).await? {
            if dry_run {
                println!("   [DRY RUN] Would skip {} (already exists)", lobby_id);
            }
            skipped_count += 1;
            continue;
        }

        // Get old LobbyInfo
        let mut conn = redis
            .get()
            .await
            .map_err(|e| AppError::RedisError(format!("Failed to get Redis connection: {}", e)))?;
        let old_key = format!("lobbies:{}:info", lobby_id);

        let lobby_info: HashMap<String, String> = conn
            .hgetall(&old_key)
            .await
            .map_err(AppError::RedisCommandError)?;

        if lobby_info.is_empty() {
            println!("   ⚠️  Empty lobby info for {}", lobby_id);
            error_count += 1;
            continue;
        }

        // Extract state fields
        let status = lobby_info
            .get("state")
            .and_then(|s| s.parse::<LobbyStatus>().ok())
            .unwrap_or(LobbyStatus::Waiting);

        let participant_count = lobby_info
            .get("participants")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        let created_at = lobby_info
            .get("created_at")
            .and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp())
            })
            .unwrap_or_else(|| chrono::Utc::now().timestamp());

        let creator_last_ping = lobby_info
            .get("creator_last_ping")
            .and_then(|s| s.parse::<u64>().ok());

        let tg_msg_id = lobby_info
            .get("tg_msg_id")
            .and_then(|s| s.parse::<i32>().ok());

        // Create new LobbyState
        let lobby_state = LobbyState {
            lobby_id,
            status: status.clone(),
            participant_count,
            created_at,
            updated_at: chrono::Utc::now().timestamp(),
            started_at: None,  // Will be set when game starts
            finished_at: None, // Will be set when game finishes
            creator_last_ping,
            tg_msg_id,
        };

        if dry_run {
            println!(
                "   [DRY RUN] Would migrate lobby {} (status: {:?}, {} participants)",
                lobby_id, status, participant_count
            );
        } else {
            // Create new state
            lobby_state_repo.upsert_state(lobby_state).await?;
            println!(
                "   ✅ Migrated lobby {} (status: {:?}, {} participants)",
                lobby_id, status, participant_count
            );
        }

        migrated_count += 1;
    }

    println!("\n   Summary:");
    println!("   - Migrated: {}", migrated_count);
    println!("   - Skipped: {}", skipped_count);
    println!("   - Errors: {}", error_count);

    Ok(migrated_count)
}
