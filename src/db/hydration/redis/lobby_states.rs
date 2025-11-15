//! Migrate lobby states from old LobbyInfo to new LobbyState

use crate::db::lobby_state::LobbyStateRepository;
use crate::errors::AppError;
use crate::models::redis::LobbyState;
use crate::state::RedisClient;
use redis::AsyncCommands;
use std::collections::HashMap;
use uuid::Uuid;

/// Migrate lobby states from old structure to new structure
///
/// Scans all `lobbies:*:info` keys and transforms them to `lobbies:{id}:state`.
pub async fn migrate_lobby_states(
    redis: &RedisClient,
    lobby_state_repo: &LobbyStateRepository,
    dry_run: bool,
) -> Result<usize, AppError> {
    let mut conn = redis
        .get()
        .await
        .map_err(|e| AppError::RedisError(format!("Failed to get Redis connection: {}", e)))?;

    // Scan all lobby info keys directly
    let pattern = "lobbies:*:info";
    let lobby_keys: Vec<String> = conn
        .keys(pattern)
        .await
        .map_err(AppError::RedisCommandError)?;

    if lobby_keys.is_empty() {
        println!("   ℹ️  No lobby keys found");
        return Ok(0);
    }

    println!("   📋 Found {} lobby keys to migrate", lobby_keys.len());

    let mut migrated_count = 0;
    let mut skipped_count = 0;
    let mut error_count = 0;

    for lobby_key in lobby_keys {
        // Extract lobby_id from key: "lobbies:{uuid}:info"
        let parts: Vec<&str> = lobby_key.split(':').collect();
        if parts.len() != 3 || parts[0] != "lobbies" || parts[2] != "info" {
            println!("   ⚠️  Invalid lobby key format: {}", lobby_key);
            error_count += 1;
            continue;
        }

        let lobby_id = match Uuid::parse_str(parts[1]) {
            Ok(id) => id,
            Err(_) => {
                println!("   ⚠️  Invalid lobby ID: {}", lobby_key);
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
        let lobby_info: HashMap<String, String> = conn
            .hgetall(&lobby_key)
            .await
            .map_err(AppError::RedisCommandError)?;

        if lobby_info.is_empty() {
            println!("   ⚠️  Empty lobby info for {}", lobby_id);
            error_count += 1;
            continue;
        }

        // Extract state fields
        let status = crate::models::redis::LobbyStatus::Finished;

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
            status: crate::models::redis::LobbyStatus::Finished,
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
