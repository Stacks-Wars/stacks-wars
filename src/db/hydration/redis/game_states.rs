//! Migrate game-specific states from scattered keys to consolidated GameState

use crate::db::hydration::redis::get_all_lobby_ids;
use crate::errors::AppError;
use crate::games::GameState;
use crate::games::lexi_wars::LexiWarsGameState;
use crate::state::RedisClient;
use redis::AsyncCommands;
use uuid::Uuid;

/// Migrate game-specific states from scattered keys to consolidated JSON
///
/// Transforms scattered keys:
/// ```
/// lobbies:{id}:used_words           → Set
/// lobbies:{id}:current_rule         → String
/// lobbies:{id}:rule_context         → String
/// lobbies:{id}:rule_index           → Int
/// lobbies:{id}:current_turn         → UUID
/// lobbies:{id}:eliminated_players   → Set
/// ```
///
/// Into single JSON key:
/// ```
/// lobbies:{id}:game_state → LexiWarsGameState (JSON)
/// ```
pub async fn migrate_game_states(redis: &RedisClient, dry_run: bool) -> Result<usize, AppError> {
    let lobby_ids = get_all_lobby_ids(redis).await?;

    if lobby_ids.is_empty() {
        println!("   ℹ️  No lobbies found");
        return Ok(0);
    }

    let mut migrated_count = 0;
    let mut skipped_count = 0;
    let mut error_count = 0;

    for lobby_id_str in lobby_ids {
        let lobby_id = match Uuid::parse_str(&lobby_id_str) {
            Ok(id) => id,
            Err(_) => {
                error_count += 1;
                continue;
            }
        };

        let mut conn = redis
            .get()
            .await
            .map_err(|e| AppError::RedisError(format!("Failed to get Redis connection: {}", e)))?;

        // Check if new game_state already exists
        let new_key = format!("lobbies:{}:game_state", lobby_id);
        let exists: bool = conn
            .exists(&new_key)
            .await
            .map_err(AppError::RedisCommandError)?;

        if exists {
            if dry_run {
                println!(
                    "   [DRY RUN] Would skip lobby {} (game_state already exists)",
                    lobby_id
                );
            }
            skipped_count += 1;
            continue;
        }

        // Collect game-specific data from scattered keys
        let used_words_key = format!("lobbies:{}:used_words", lobby_id);
        let current_rule_key = format!("lobbies:{}:current_rule", lobby_id);
        let rule_context_key = format!("lobbies:{}:rule_context", lobby_id);
        let rule_index_key = format!("lobbies:{}:rule_index", lobby_id);
        let current_turn_key = format!("lobbies:{}:current_turn", lobby_id);
        let eliminated_key = format!("lobbies:{}:eliminated_players", lobby_id);

        // Get used words (Set)
        let used_words: Vec<String> = conn.smembers(&used_words_key).await.unwrap_or_default();

        // Get current rule (String)
        let current_rule: Option<String> = conn.get(&current_rule_key).await.ok();

        // Get rule context (String)
        let rule_context: Option<String> = conn.get(&rule_context_key).await.ok();

        // Get rule index (Int)
        let rule_index: usize = conn.get(&rule_index_key).await.unwrap_or(0);

        // Get current turn (UUID String)
        let current_turn: Option<Uuid> = conn
            .get(&current_turn_key)
            .await
            .ok()
            .and_then(|s: String| Uuid::parse_str(&s).ok());

        // Get eliminated players (Set of UUID strings)
        let eliminated_uuids: Vec<String> =
            conn.smembers(&eliminated_key).await.unwrap_or_default();

        let eliminated_players: Vec<Uuid> = eliminated_uuids
            .into_iter()
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect();

        // Create LexiWarsGameState
        let game_state = LexiWarsGameState {
            used_words,
            current_rule,
            rule_context,
            rule_index,
            current_turn,
            eliminated_players,
        };

        if dry_run {
            println!(
                "   [DRY RUN] Would migrate game state for lobby {} ({} words, {} eliminated)",
                lobby_id,
                game_state.used_words.len(),
                game_state.eliminated_players.len()
            );
        } else {
            // Serialize to JSON and store
            let json = game_state.to_json()?;
            let _: () = conn
                .set(&new_key, json)
                .await
                .map_err(AppError::RedisCommandError)?;

            migrated_count += 1;

            if migrated_count % 100 == 0 {
                println!("   ✅ Migrated {} game states...", migrated_count);
            }
        }
    }

    println!("\n   Summary:");
    println!("   - Migrated: {}", migrated_count);
    println!("   - Skipped: {}", skipped_count);
    println!("   - Errors: {}", error_count);

    Ok(migrated_count)
}
