//! # Redis State Restructuring
//!
//! This module handles the migration of Redis data from the old scattered key structure
//! to the new organized state architecture.
//!
//! ## Migration Overview
//!
//! ### Old Structure (scattered keys):
//! ```
//! lobbies:{id}:info                 → LobbyInfo hash
//! lobbies:{id}:player:{user_id}     → Player hash
//! lobbies:{id}:used_words           → Set
//! lobbies:{id}:current_rule         → String
//! lobbies:{id}:rule_context         → String
//! lobbies:{id}:rule_index           → Int
//! lobbies:{id}:current_turn         → UUID
//! lobbies:{id}:eliminated_players   → Set
//! ```
//!
//! ### New Structure (organized state):
//! ```
//! lobbies:{id}:state                → LobbyState hash
//! lobbies:{id}:players:{user_id}    → PlayerState hash
//! lobbies:{id}:game_state           → LexiWarsGameState JSON
//! ```

pub mod game_states;
pub mod lobby_states;
pub mod player_states;

use crate::db::{lobby_state::LobbyStateRepository, player_state::PlayerStateRepository};
use crate::errors::AppError;
use crate::state::RedisClient;
use redis::AsyncCommands;

/// Run all Redis state restructuring migrations
///
/// # Arguments
/// * `redis` - The Redis client
/// * `lobby_state_repo` - Repository for lobby states
/// * `player_state_repo` - Repository for player states
/// * `dry_run` - If true, only report what would be migrated without making changes
///
/// # Returns
/// * `Ok((lobbies_migrated, players_migrated, game_states_migrated))` on success
pub async fn migrate_all_redis_state(
    redis: &RedisClient,
    lobby_state_repo: &LobbyStateRepository,
    player_state_repo: &PlayerStateRepository,
    dry_run: bool,
) -> Result<(usize, usize, usize), AppError> {
    println!("╔═══════════════════════════════════════════════╗");
    println!("║  Redis State Restructuring Migration         ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    if dry_run {
        println!("🔍 DRY RUN MODE - No changes will be made\n");
    }

    // Phase 1: Migrate lobby states
    println!("📊 Phase 1: Migrating lobby states...");
    let lobbies_migrated =
        lobby_states::migrate_lobby_states(redis, lobby_state_repo, dry_run).await?;
    println!("   ✅ {} lobby states migrated\n", lobbies_migrated);

    // Phase 2: Migrate player states
    println!("📊 Phase 2: Migrating player states...");
    let players_migrated =
        player_states::migrate_player_states(redis, player_state_repo, dry_run).await?;
    println!("   ✅ {} player states migrated\n", players_migrated);

    // Phase 3: Migrate game-specific states
    println!("📊 Phase 3: Migrating game-specific states...");
    let game_states_migrated = game_states::migrate_game_states(redis, dry_run).await?;
    println!("   ✅ {} game states migrated\n", game_states_migrated);

    println!("╔═══════════════════════════════════════════════╗");
    println!("║  🎉 Migration Complete!                      ║");
    println!(
        "║  ✅ {} lobbies migrated                      ",
        lobbies_migrated
    );
    println!(
        "║  ✅ {} players migrated                      ",
        players_migrated
    );
    println!(
        "║  ✅ {} game states migrated                  ",
        game_states_migrated
    );
    println!("╚═══════════════════════════════════════════════╝");

    if dry_run {
        println!("\n⚠️  This was a DRY RUN - no actual changes were made");
        println!("   Run again with --apply to perform the migration");
    }

    Ok((lobbies_migrated, players_migrated, game_states_migrated))
}

/// Get all lobby IDs from Redis (from old :info keys)
pub(crate) async fn get_all_lobby_ids(redis: &RedisClient) -> Result<Vec<String>, AppError> {
    let mut conn = redis
        .get()
        .await
        .map_err(|e| AppError::RedisError(format!("Failed to get Redis connection: {}", e)))?;
    let pattern = "lobbies:*:info";

    let keys: Vec<String> = conn
        .keys(pattern)
        .await
        .map_err(AppError::RedisCommandError)?;

    let mut lobby_ids = Vec::new();

    for key in keys {
        // Extract lobby_id from key: "lobbies:{uuid}:info"
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() == 3 && parts[0] == "lobbies" && parts[2] == "info" {
            lobby_ids.push(parts[1].to_string());
        }
    }

    Ok(lobby_ids)
}
