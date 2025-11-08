//! Binary to run Redis state restructuring migration
//!
//! This migrates Redis data from old scattered keys to new organized state structure.
//!
//! ## Usage
//!
//! ```bash
//! # Dry run (preview without making changes)
//! cargo run --bin migrate_redis
//!
//! # Actually perform the migration
//! cargo run --bin migrate_redis -- --apply
//! ```

use stacks_wars_be::{
    db::{
        hydration::redis::migrate_all_redis_state, lobby_state::LobbyStateRepository,
        player_state::PlayerStateRepository,
    },
    state::AppState,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Check for --apply flag
    let args: Vec<String> = env::args().collect();
    let dry_run = !args.contains(&"--apply".to_string());

    println!("\n╔═══════════════════════════════════════════════╗");
    println!("║  Redis State Restructuring Migration         ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    if dry_run {
        println!("🔍 Running in DRY RUN mode (preview only)");
        println!("   Use --apply flag to actually perform the migration\n");
    } else {
        println!("⚠️  Running in APPLY mode - changes will be made!");
        println!("   Press Ctrl+C within 5 seconds to cancel...\n");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    // Initialize app state
    println!("📡 Connecting to Redis...");
    let state = AppState::new().await?;
    println!("✅ Connected!\n");

    // Create repositories
    let lobby_state_repo = LobbyStateRepository::new(state.redis.clone());
    let player_state_repo = PlayerStateRepository::new(state.redis.clone());

    // Run migration
    let (lobbies, players, game_states) =
        migrate_all_redis_state(&state.redis, &lobby_state_repo, &player_state_repo, dry_run)
            .await?;

    println!("\n╔═══════════════════════════════════════════════╗");
    println!("║  Migration Summary                            ║");
    println!("╠═══════════════════════════════════════════════╣");
    println!("║  Lobbies migrated:      {:>5}                ║", lobbies);
    println!("║  Players migrated:      {:>5}                ║", players);
    println!(
        "║  Game states migrated:  {:>5}                ║",
        game_states
    );
    println!("╚═══════════════════════════════════════════════╝");

    if dry_run {
        println!("\n✅ Dry run complete! No changes were made.");
        println!("   Run with --apply to perform the actual migration:");
        println!("   cargo run --bin migrate_redis -- --apply");
    } else {
        println!("\n✅ Migration complete!");
        println!("\n⚠️  IMPORTANT: Old keys are still present.");
        println!("   The old keys (lobbies:*:info, lobbies:*:player:*) are still in Redis.");
        println!("   Once you've verified the migration worked, you can delete them.");
    }

    Ok(())
}
