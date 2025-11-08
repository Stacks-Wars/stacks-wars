//! # PostgreSQL Hydration from Redis
//!
//! This module provides functions to populate PostgreSQL with existing data from Redis.
//! This is a one-time migration needed because PostgreSQL was added later to the architecture.
//!
//! ## Strategy
//!
//! 1. Read existing data from Redis (users, lobbies, etc.) using existing structs
//! 2. Insert into PostgreSQL using raw SQL (repositories have different signatures)
//! 3. Maintain existing Redis keys for backward compatibility during transition
//! 4. Once complete, restructure Redis keys to new format (Phase 4)
//!
//! ## Usage
//!
//! ```bash
//! cargo run --bin hydrate
//! ```

use crate::errors::AppError;
use crate::models::game::LobbyInfo;
use crate::models::redis_key::{KeyPart, RedisKey};
use crate::state::RedisClient;
use ::redis::AsyncCommands;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

pub mod redis;

/// Hydrate users table from Redis
///
/// Reads from: `users:data:{user_id}` (Redis hash)
/// Writes to: `users` table (PostgreSQL)
///
/// Uses the User struct to parse Redis data correctly.
pub async fn hydrate_users_from_redis(
    redis: &RedisClient,
    pool: &PgPool,
) -> Result<usize, AppError> {
    let mut conn = redis
        .get()
        .await
        .map_err(|e| AppError::RedisError(format!("Failed to get Redis connection: {}", e)))?;

    // Get all user keys: users:data:*
    let pattern = RedisKey::user(KeyPart::Wildcard);
    let keys: Vec<String> = conn
        .keys(&pattern)
        .await
        .map_err(AppError::RedisCommandError)?;

    let mut hydrated_count = 0;

    println!(
        "Found {} user keys matching pattern: {}",
        keys.len(),
        pattern
    );

    for key in keys {
        // Extract user_id from key "users:data:{uuid}"
        let user_id = key
            .strip_prefix("users:data:")
            .and_then(|id| Uuid::parse_str(id).ok());

        if user_id.is_none() {
            println!("⚠️  Skipping invalid user key: {}", key);
            continue;
        }
        let user_id = user_id.unwrap();

        // Get user data from Redis hash
        let user_data: HashMap<String, String> = conn
            .hgetall(&key)
            .await
            .map_err(AppError::RedisCommandError)?;

        if user_data.is_empty() {
            println!("⚠️  Empty data for user {}, skipping", user_id);
            continue;
        }

        // Parse into User struct (this contains all the fields we need)
        // User struct has: id, wallet_address, wars_point, username, display_name
        let wallet_address = match user_data.get("wallet_address") {
            Some(addr) => addr.clone(),
            None => {
                println!("⚠️  Missing wallet_address for user {}, skipping", user_id);
                continue;
            }
        };

        let username = user_data.get("username").cloned();
        let display_name = user_data.get("display_name").cloned();

        // Insert into PostgreSQL
        // Note: wars_point is NOT in the users table anymore - it's in user_wars_points
        let result = sqlx::query(
            r#"
            INSERT INTO users (id, wallet_address, username, display_name, trust_rating, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $6)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(&wallet_address)
        .bind(&username)
        .bind(&display_name)
        .bind(10.0) // Default trust rating
        .bind(chrono::Utc::now().naive_utc())
        .execute(pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to insert user {}: {}", user_id, e))
        })?;

        if result.rows_affected() > 0 {
            hydrated_count += 1;
            println!(
                "✅ Hydrated user: {} ({})",
                username.as_deref().unwrap_or("unknown"),
                user_id
            );
        } else {
            println!("  User {} already exists, skipping", user_id);
        }
    }

    Ok(hydrated_count)
}

/// Hydrate lobbies table from Redis
///
/// Reads from: `lobbies:{lobby_id}:info` (Redis hash)
/// Writes to: `lobbies` table (PostgreSQL)
///
/// Uses the LobbyInfo struct to parse Redis data correctly.
/// Business rules:
/// - is_private: Set to true for all (doesn't exist in LobbyInfo)
/// - is_sponsored: true if entry_amount is 0 and current_amount > 0
pub async fn hydrate_lobbies_from_redis(
    redis: &RedisClient,
    pool: &PgPool,
) -> Result<usize, AppError> {
    let mut conn = redis
        .get()
        .await
        .map_err(|e| AppError::RedisError(format!("Failed to get Redis connection: {}", e)))?;

    // Get all lobby info keys: lobbies:*:info
    let pattern = RedisKey::lobby(KeyPart::Wildcard);
    let keys: Vec<String> = conn
        .keys(&pattern)
        .await
        .map_err(AppError::RedisCommandError)?;

    let mut hydrated_count = 0;

    println!(
        "Found {} lobby keys matching pattern: {}",
        keys.len(),
        pattern
    );

    for key in keys {
        // Extract lobby_id from key "lobbies:{uuid}:info"
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() != 3 || parts[2] != "info" {
            println!("⚠️  Invalid lobby key format: {}", key);
            continue;
        }

        let lobby_id = match Uuid::parse_str(parts[1]) {
            Ok(id) => id,
            Err(_) => {
                println!("⚠️  Invalid lobby ID in key: {}", key);
                continue;
            }
        };

        // Get lobby data from Redis hash
        let lobby_data: HashMap<String, String> = conn
            .hgetall(&key)
            .await
            .map_err(AppError::RedisCommandError)?;

        if lobby_data.is_empty() {
            println!("⚠️  Empty data for lobby {}, skipping", lobby_id);
            continue;
        }

        // Parse using LobbyInfo::from_redis_hash_partial
        // This returns (LobbyInfo, creator_id, game_id)
        let (lobby_info, creator_id, game_id) =
            match LobbyInfo::from_redis_hash_partial(&lobby_data) {
                Ok(data) => data,
                Err(e) => {
                    println!("⚠️  Failed to parse lobby {}: {}", lobby_id, e);
                    continue;
                }
            };

        // Extract fields from LobbyInfo
        let name = lobby_info.name;
        let description = lobby_info.description;
        let entry_amount = lobby_info.entry_amount.unwrap_or(0.0);
        let current_amount = lobby_info.current_amount.unwrap_or(0.0);
        let token_symbol = lobby_info.token_symbol;
        let token_contract_id = lobby_info.token_id; // token_id in LobbyInfo
        let contract_address = lobby_info.contract_address;

        // Business rules per user's instructions:
        // - is_private: Set to true for all (doesn't exist in LobbyInfo)
        let is_private = true;

        // - is_sponsored: true if entry_amount is 0 and current_amount > 0
        let is_sponsored = entry_amount == 0.0 && current_amount > 0.0;

        // Convert LobbyState enum to PostgreSQL enum string
        let status = format!("{:?}", lobby_info.state).to_lowercase();

        // Insert into PostgreSQL using raw SQL
        let result = sqlx::query(
            r#"
            INSERT INTO lobbies (
                id, name, description, creator_id, game_id,
                entry_amount, current_amount, token_symbol, token_contract_id, contract_address,
                is_private, is_sponsored, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $14)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(lobby_id)
        .bind(&name)
        .bind(description)
        .bind(creator_id)
        .bind(game_id)
        .bind(entry_amount)
        .bind(current_amount)
        .bind(token_symbol)
        .bind(token_contract_id)
        .bind(contract_address)
        .bind(is_private)
        .bind(is_sponsored)
        .bind(&status)
        .bind(chrono::Utc::now().naive_utc())
        .execute(pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to insert lobby {}: {}", lobby_id, e))
        })?;

        if result.rows_affected() > 0 {
            hydrated_count += 1;
            println!(
                "✅ Hydrated lobby: {} ({}) - private={}, sponsored={}",
                name, lobby_id, is_private, is_sponsored
            );
        } else {
            println!("  Lobby {} already exists, skipping", lobby_id);
        }
    }

    Ok(hydrated_count)
}

/// Hydrate all tables from Redis (one-time migration)
///
/// This is a one-time operation to populate PostgreSQL with data from Redis.
/// Should be run once during migration to the new architecture.
pub async fn hydrate_all_from_redis(redis: &RedisClient, pool: &PgPool) -> Result<(), AppError> {
    println!("╔═══════════════════════════════════════════════╗");
    println!("║  Starting PostgreSQL Hydration from Redis   ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    // Hydrate users first (lobbies depend on users)
    println!("📊 Phase 1: Hydrating users table...");
    let user_count = hydrate_users_from_redis(redis, pool).await?;
    println!("   {} users migrated\n", user_count);

    // Hydrate lobbies
    println!("📊 Phase 2: Hydrating lobbies table...");
    let lobby_count = hydrate_lobbies_from_redis(redis, pool).await?;
    println!("   {} lobbies migrated\n", lobby_count);

    println!("╔═══════════════════════════════════════════════╗");
    println!("║  🎉 Hydration Complete!                      ║");
    println!(
        "║  ✅ {} users migrated                        ",
        user_count
    );
    println!(
        "║  ✅ {} lobbies migrated                      ",
        lobby_count
    );
    println!("╚═══════════════════════════════════════════════╝");

    Ok(())
}
