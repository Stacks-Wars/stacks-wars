use stacks_wars_be::db::hydration;
use stacks_wars_be::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    println!("\n🚀 Initializing application state...");

    // Initialize app state (connects to both PostgreSQL and Redis)
    let state = AppState::new().await?;

    println!("✅ Connected to PostgreSQL and Redis\n");

    // Run hydration from Redis to PostgreSQL
    hydration::hydrate_all_from_redis(&state.redis, &state.postgres).await?;

    println!("\n✨ Hydration script completed successfully!");

    Ok(())
}
