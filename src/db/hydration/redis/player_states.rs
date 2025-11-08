//! Migrate player states from old Player to new PlayerState

use crate::db::hydration::redis::get_all_lobby_ids;
use crate::db::player_state::PlayerStateRepository;
use crate::errors::AppError;
use crate::models::enums::{ClaimState, PlayerState as PlayerStatus};
use crate::models::game::Player;
use crate::models::redis::PlayerState;
use crate::state::RedisClient;
use redis::AsyncCommands;
use std::collections::HashMap;
use uuid::Uuid;

/// Migrate player states from old structure to new structure
///
/// Transforms: `lobbies:{id}:player:{user_id}` (Player) → `lobbies:{id}:players:{user_id}` (PlayerState)
///
/// Extracts only platform-generic fields:
/// - user_id, lobby_id, status
/// - tx_id, rank, prize, claim_state
/// - last_ping, joined_at, updated_at
///
/// Game-specific fields (used_words) are EXCLUDED and will be in GameState.
pub async fn migrate_player_states(
    redis: &RedisClient,
    player_state_repo: &PlayerStateRepository,
    dry_run: bool,
) -> Result<usize, AppError> {
    let lobby_ids = get_all_lobby_ids(redis).await?;

    if lobby_ids.is_empty() {
        println!("   ℹ️  No lobbies found");
        return Ok(0);
    }

    let mut migrated_count = 0;
    let mut skipped_count = 0;
    let mut error_count = 0;

    for lobby_id_str in &lobby_ids {
        let lobby_id = match Uuid::parse_str(lobby_id_str) {
            Ok(id) => id,
            Err(_) => {
                error_count += 1;
                continue;
            }
        };

        // Get all player keys for this lobby
        let mut conn = redis
            .get()
            .await
            .map_err(|e| AppError::RedisError(format!("Failed to get Redis connection: {}", e)))?;
        let pattern = format!("lobbies:{}:player:*", lobby_id);

        let player_keys: Vec<String> = conn
            .keys(&pattern)
            .await
            .map_err(AppError::RedisCommandError)?;

        for player_key in player_keys {
            // Extract user_id from key: "lobbies:{lobby_id}:player:{user_id}"
            let parts: Vec<&str> = player_key.split(':').collect();
            if parts.len() != 4 {
                println!("   ⚠️  Invalid player key format: {}", player_key);
                error_count += 1;
                continue;
            }

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

            // Parse old Player using existing method
            let old_player = match Player::from_redis_hash(&player_data) {
                Ok(p) => p,
                Err(e) => {
                    println!("   ⚠️  Failed to parse player {}: {}", user_id, e);
                    error_count += 1;
                    continue;
                }
            };

            // Convert ClaimState from old format to new format
            let claim_state = old_player.claim.and_then(|claim| match claim {
                crate::models::game::ClaimState::Claimed { tx_id } => {
                    Some(ClaimState::Claimed { tx_id })
                }
                crate::models::game::ClaimState::NotClaimed => Some(ClaimState::NotClaimed),
            });

            // Create new PlayerState (NO game-specific fields like used_words!)
            let player_state = PlayerState {
                user_id,
                lobby_id,
                status: match old_player.state {
                    crate::models::game::PlayerState::NotJoined => PlayerStatus::NotJoined,
                    crate::models::game::PlayerState::Joined => PlayerStatus::Joined,
                },
                tx_id: old_player.tx_id,
                rank: old_player.rank,
                prize: old_player.prize,
                claim_state,
                last_ping: old_player.last_ping,
                joined_at: chrono::Utc::now().timestamp(), // Old model doesn't have this
                updated_at: chrono::Utc::now().timestamp(),
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

                if migrated_count % 100 == 0 {
                    println!("   ✅ Migrated {} players...", migrated_count);
                }
            }
        }
    }

    println!("\n   Summary:");
    println!("   - Migrated: {}", migrated_count);
    println!("   - Skipped: {}", skipped_count);
    println!("   - Errors: {}", error_count);

    Ok(migrated_count)
}
