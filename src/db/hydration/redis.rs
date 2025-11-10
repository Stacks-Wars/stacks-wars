//! # Redis State Restructuring
//!
//! Migrates Redis data from old scattered key structure to new organized state architecture.
//!
//! ## Migration:
//! - `lobbies:{id}:info` → `lobbies:{id}:state` (LobbyState)
//! - `lobbies:{id}:player:{user_id}` → `lobbies:{id}:players:{user_id}` (PlayerState)

pub mod lobby_states;
pub mod player_states;

use crate::db::{lobby_state::LobbyStateRepository, player_state::PlayerStateRepository};
use crate::errors::AppError;
use crate::state::RedisClient;

/// Run all Redis state restructuring migrations
///
/// Migrates lobbies and players independently by scanning Redis keys directly.
pub async fn migrate_all_redis_state(
    redis: &RedisClient,
    lobby_state_repo: &LobbyStateRepository,
    player_state_repo: &PlayerStateRepository,
    dry_run: bool,
) -> Result<(usize, usize), AppError> {
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

    println!("╔═══════════════════════════════════════════════╗");
    println!("║  🎉 Migration Complete!                      ║");
    println!(
        "║  ✅ {} lobbies migrated                       ",
        lobbies_migrated
    );
    println!(
        "║  ✅ {} players migrated                       ",
        players_migrated
    );
    println!("╚═══════════════════════════════════════════════╝");

    if dry_run {
        println!("\n⚠️  This was a DRY RUN - no actual changes were made");
        println!("   Run again with --apply to perform the migration");
    }

    Ok((lobbies_migrated, players_migrated))
}
