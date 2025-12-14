// Migrate player states from old Player model to new PlayerState

use crate::db::hydration::types::{ClaimState, Player, PlayerState as PlayerStatus};
use crate::db::player_state::PlayerStateRepository;
use crate::errors::AppError;
use crate::models::PlayerState;
use crate::state::RedisClient;
use redis::AsyncCommands;
use std::collections::HashMap;
use uuid::Uuid;

/// Migrate player states from old structure to new structure
///
/// Scans all `lobbies:*:player:*` keys and transforms them to `lobbies:{id}:players:{user_id}`.
pub async fn migrate_player_states(
    redis: &RedisClient,
    player_state_repo: &PlayerStateRepository,
    dry_run: bool,
) -> Result<usize, AppError> {
    let mut conn = redis
        .get()
        .await
        .map_err(|e| AppError::RedisError(format!("Failed to get Redis connection: {}", e)))?;

    // Scan all player keys directly
    let pattern = "lobbies:*:player:*";
    let player_keys: Vec<String> = conn
        .keys(pattern)
        .await
        .map_err(AppError::RedisCommandError)?;

    if player_keys.is_empty() {
        println!("   ℹ️  No player keys found");
        return Ok(0);
    }

    println!("   📋 Found {} player keys to migrate", player_keys.len());

    let mut migrated_count = 0;
    let mut skipped_count = 0;
    let mut error_count = 0;

    for player_key in player_keys {
        // Extract lobby_id and user_id from key: "lobbies:{lobby_id}:player:{user_id}"
        let parts: Vec<&str> = player_key.split(':').collect();
        if parts.len() != 4 || parts[0] != "lobbies" || parts[2] != "player" {
            println!("   ⚠️  Invalid player key format: {}", player_key);
            error_count += 1;
            continue;
        }

        let lobby_id = match Uuid::parse_str(parts[1]) {
            Ok(id) => id,
            Err(_) => {
                println!("   ⚠️  Invalid lobby ID in key: {}", player_key);
                error_count += 1;
                continue;
            }
        };

        let user_id = match Uuid::parse_str(parts[3]) {
            Ok(id) => id,
            Err(_) => {
                println!("   ⚠️  Invalid user ID in key: {}", player_key);
                error_count += 1;
                continue;
            }
        };

        // Check if new state already exists
        if player_state_repo.exists(lobby_id, user_id).await? {
            if dry_run {
                println!(
                    "   [DRY RUN] Would skip player {} in lobby {} (already exists)",
                    user_id, lobby_id
                );
            }
            skipped_count += 1;
            continue;
        }

        // Get old Player data
        let player_data: HashMap<String, String> = conn
            .hgetall(&player_key)
            .await
            .map_err(AppError::RedisCommandError)?;

        if player_data.is_empty() {
            println!("   ⚠️  Empty player data: {}", player_key);
            error_count += 1;
            continue;
        }

        // Parse old Player - if it fails, create a basic player state with Joined status
        let old_player = match Player::from_redis_hash(&player_data) {
            Ok(p) => p,
            Err(_) => {
                // For old data with incompatible states (ready, notready, etc.),
                // just map all players as Joined since they were in the lobby
                let player_state = PlayerState {
                    user_id,
                    username: player_data.get("username").cloned(),
                    display_name: player_data.get("display_name").cloned(),
                    wallet_address: player_data
                        .get("wallet_address")
                        .cloned()
                        .unwrap_or_default(),
                    trust_rating: player_data
                        .get("trust_rating")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(10.0),
                    lobby_id,
                    status: crate::models::player_state::PlayerStatus::Joined, // All old players mapped as Joined
                    tx_id: player_data.get("tx_id").map(|s| s.clone()),
                    rank: player_data.get("rank").and_then(|s| s.parse().ok()),
                    prize: player_data.get("prize").and_then(|s| s.parse().ok()),
                    claim_state: None,
                    last_ping: player_data.get("last_ping").and_then(|s| s.parse().ok()),
                    joined_at: chrono::Utc::now().timestamp(),
                    updated_at: chrono::Utc::now().timestamp(),
                    is_creator: false,
                };

                if dry_run {
                    println!(
                        "   [DRY RUN] Would migrate player {} in lobby {} (as Joined)",
                        user_id, lobby_id
                    );
                } else {
                    player_state_repo.upsert_state(player_state).await?;
                    migrated_count += 1;

                    if migrated_count % 10 == 0 {
                        println!("   ✅ Migrated {} players...", migrated_count);
                    }
                }
                continue;
            }
        };

        // Convert ClaimState from old format to new format
        let claim_state = old_player.claim.and_then(|claim| match claim {
            ClaimState::Claimed { tx_id } => {
                Some(crate::models::player_state::ClaimState::Claimed { tx_id })
            }
            ClaimState::NotClaimed => Some(crate::models::player_state::ClaimState::NotClaimed),
        });

        // Create new PlayerState
        let player_state = PlayerState {
            user_id,
            username: player_data.get("username").cloned(),
            display_name: player_data.get("display_name").cloned(),
            wallet_address: player_data
                .get("wallet_address")
                .cloned()
                .unwrap_or_default(),
            trust_rating: player_data
                .get("trust_rating")
                .and_then(|s| s.parse().ok())
                .unwrap_or(10.0),
            lobby_id,
            status: match old_player.state {
                PlayerStatus::NotJoined => crate::models::player_state::PlayerStatus::NotJoined,
                PlayerStatus::Joined => crate::models::player_state::PlayerStatus::Joined,
            },
            tx_id: old_player.tx_id,
            rank: old_player.rank,
            prize: old_player.prize,
            claim_state,
            last_ping: old_player.last_ping,
            joined_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            is_creator: false,
        };

        if dry_run {
            println!(
                "   [DRY RUN] Would migrate player {} in lobby {}",
                user_id, lobby_id
            );
        } else {
            // Create new state
            player_state_repo.upsert_state(player_state).await?;
            migrated_count += 1;

            if migrated_count % 10 == 0 {
                println!("   ✅ Migrated {} players...", migrated_count);
            }
        }
    }

    if skipped_count > 0 {
        println!(
            "   ⏭️  Skipped {} players (already migrated)",
            skipped_count
        );
    }
    if error_count > 0 {
        println!("   ❌ Failed to migrate {} players", error_count);
    }

    Ok(migrated_count)
}
