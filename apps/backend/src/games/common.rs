// Generic game state management infrastructure
//
// This module provides reusable components for game developers:
// - GamePlayerState: Per-player game state (in-memory, separate from lobby PlayerState)
// - Turn management and rotation
// - Elimination tracking
// - Ranking/results system
// - Sync results to PlayerState after game completion
// - Save permanent game summaries to Redis

use crate::{
    db::{
        player_state::PlayerStateRepository, season::SeasonRepository,
        user_wars_points::UserWarsPointsRepository,
    },
    errors::AppError,
    state::{AppState, RedisClient},
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// Game-specific player state (stored separately from lobby PlayerState)
///
/// This tracks game-specific information like eliminations, scores, positions, etc.
/// Each game can extend this or create their own structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GamePlayerState {
    pub user_id: Uuid,
    pub is_eliminated: bool,
    pub position: Option<usize>, // Final rank/position (1st, 2nd, 3rd...)
    pub score: i32,
    pub eliminated_at: Option<i64>, // Unix timestamp
}

impl GamePlayerState {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            is_eliminated: false,
            position: None,
            score: 0,
            eliminated_at: None,
        }
    }

    pub fn eliminate(&mut self) {
        self.is_eliminated = true;
        self.eliminated_at = Some(chrono::Utc::now().timestamp());
    }

    pub fn is_active(&self) -> bool {
        !self.is_eliminated
    }
}

/// Turn-based game rotation system
///
/// Handles player turns with automatic rotation, skip eliminated players,
/// and optional countdown per turn.
#[derive(Debug, Clone)]
pub struct TurnRotation {
    players: VecDeque<Uuid>,
    current_index: usize,
    eliminated: HashMap<Uuid, bool>,
}

impl TurnRotation {
    /// Create new turn rotation from player list
    pub fn new(player_ids: Vec<Uuid>) -> Self {
        Self {
            players: player_ids.into_iter().collect(),
            current_index: 0,
            eliminated: HashMap::new(),
        }
    }

    /// Get current player's turn
    pub fn current_player(&self) -> Option<Uuid> {
        self.active_players().get(self.current_index).copied()
    }

    /// Get all active (non-eliminated) players
    pub fn active_players(&self) -> Vec<Uuid> {
        self.players
            .iter()
            .filter(|id| !self.eliminated.get(id).unwrap_or(&false))
            .copied()
            .collect()
    }

    /// Get count of active players
    pub fn active_count(&self) -> usize {
        self.active_players().len()
    }

    /// Move to next player's turn (skips eliminated players)
    pub fn next_turn(&mut self) -> Option<Uuid> {
        if self.active_count() == 0 {
            return None;
        }

        let active = self.active_players();
        self.current_index = (self.current_index + 1) % active.len();
        active.get(self.current_index).copied()
    }

    /// Eliminate a player from rotation
    pub fn eliminate_player(&mut self, player_id: Uuid) {
        self.eliminated.insert(player_id, true);

        // If we eliminated the current player, move to next
        if self.current_player() == Some(player_id) {
            self.next_turn();
        }
    }

    /// Check if game should end (only 0 or 1 players left)
    pub fn is_game_over(&self) -> bool {
        self.active_count() <= 1
    }

    /// Get winner (last player standing)
    pub fn get_winner(&self) -> Option<Uuid> {
        let active = self.active_players();
        if active.len() == 1 {
            active.first().copied()
        } else {
            None
        }
    }
}

/// Final game results with player rankings
///
/// This is the standard format for game results that the platform expects.
/// All games should return this structure at the end.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameResults {
    /// Ordered list of players by rank (1st place first)
    pub rankings: Vec<PlayerRanking>,

    /// Unix timestamp when game ended
    pub finished_at: i64,

    /// Optional game-specific metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Individual player ranking in final results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRanking {
    pub user_id: Uuid,
    pub rank: usize,        // 1-based: 1 = first place, 2 = second, etc.
    pub score: Option<i32>, // Optional score
    pub prize: Option<f64>, // Prize amount (calculated by platform)
}

impl GameResults {
    /// Create results from ordered list of player IDs (first = winner)
    pub fn from_ordered_players(player_ids: Vec<Uuid>) -> Self {
        let rankings = player_ids
            .into_iter()
            .enumerate()
            .map(|(idx, user_id)| PlayerRanking {
                user_id,
                rank: idx + 1, // 1-based ranking
                score: None,
                prize: None, // Platform will calculate
            })
            .collect();

        Self {
            rankings,
            finished_at: chrono::Utc::now().timestamp(),
            metadata: None,
        }
    }

    /// Create results from game player states (ordered by elimination)
    pub fn from_game_states(mut states: Vec<GamePlayerState>) -> Self {
        // Sort by: active players first, then by elimination time (last eliminated = higher rank)
        states.sort_by(|a, b| {
            match (a.is_eliminated, b.is_eliminated) {
                (false, true) => std::cmp::Ordering::Less, // Active beats eliminated
                (true, false) => std::cmp::Ordering::Greater,
                (false, false) => std::cmp::Ordering::Equal,
                (true, true) => {
                    // Both eliminated: later elimination = higher rank
                    b.eliminated_at.cmp(&a.eliminated_at)
                }
            }
        });

        let rankings = states
            .into_iter()
            .enumerate()
            .map(|(idx, state)| PlayerRanking {
                user_id: state.user_id,
                rank: idx + 1,
                score: Some(state.score),
                prize: None,
            })
            .collect();

        Self {
            rankings,
            finished_at: chrono::Utc::now().timestamp(),
            metadata: None,
        }
    }
}

/// Generic game bootstrap message
///
/// Sent to clients when they connect to an in-progress or finished game.
/// Contains current game state so UI can be populated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameBootstrap<TState> {
    pub game_id: Uuid,
    pub status: GameStatus,
    pub current_state: TState,
    pub players: Vec<Uuid>,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}

/// Game status enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum GameStatus {
    InProgress,
    Finished,
}

/// Game summary stored permanently in Redis after game completion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSummary {
    /// Final game results with rankings
    pub results: GameResults,

    /// Game-specific metadata (e.g., coin flips, words played, etc.)
    pub metadata: serde_json::Value,

    /// Unix timestamp when game finished
    pub finished_at: i64,
}

/// Result of saving a player's game result
#[derive(Debug, Clone)]
pub struct PlayerResult {
    pub rank: usize,
    pub prize: Option<f64>,
    pub wars_point: f64,
}

/// Save a player's game result to Redis and PostgreSQL
///
/// This function:
/// 1. Calculates wars_point using the provided context
/// 2. Saves rank, prize, wars_point to Redis PlayerState
/// 3. Saves wars_point to PostgreSQL user_wars_points for current season
/// 4. Returns the calculated values
pub async fn save_player_result(
    state: &AppState,
    lobby_id: Uuid,
    ctx: &WarsPointContext,
) -> Result<PlayerResult, AppError> {
    let wars_point = calculate_wars_point(ctx);

    // Save to Redis PlayerState
    let player_repo = PlayerStateRepository::new(state.redis.clone());
    player_repo
        .set_result(lobby_id, ctx.user_id, ctx.rank, ctx.prize, wars_point)
        .await?;

    // Save wars_point to PostgreSQL user_wars_points for current season
    let season_repo = SeasonRepository::new(state.postgres.clone());
    if let Ok(season_id) = season_repo.get_current_season().await {
        let wars_points_repo = UserWarsPointsRepository::new(state.postgres.clone());
        let _ = wars_points_repo
            .upsert_wars_points(ctx.user_id, season_id, wars_point)
            .await;
    }

    Ok(PlayerResult {
        rank: ctx.rank,
        prize: ctx.prize,
        wars_point,
    })
}

/// Save permanent game summary to Redis
///
/// This persists the final game results and metadata so players can view
/// game history even after the game instance is dropped from memory.
/// The summary is stored at key: `game:{lobby_id}:state`
pub async fn save_game_summary(
    redis: &RedisClient,
    lobby_id: Uuid,
    results: &GameResults,
    metadata: serde_json::Value,
) -> Result<(), AppError> {
    let mut conn = redis
        .get()
        .await
        .map_err(|e| AppError::RedisError(format!("Failed to get Redis connection: {}", e)))?;

    let key = format!("game:{}:state", lobby_id);
    let summary = GameSummary {
        results: results.clone(),
        metadata,
        finished_at: chrono::Utc::now().timestamp(),
    };

    let json =
        serde_json::to_string(&summary).map_err(|e| AppError::Serialization(e.to_string()))?;

    let _: () = conn
        .set(&key, json)
        .await
        .map_err(AppError::RedisCommandError)?;

    tracing::info!("Saved game summary for lobby {}", lobby_id);
    Ok(())
}

/// Load game summary from Redis (for viewing completed games)
pub async fn load_game_summary(
    redis: &RedisClient,
    lobby_id: Uuid,
) -> Result<Option<GameSummary>, AppError> {
    let mut conn = redis
        .get()
        .await
        .map_err(|e| AppError::RedisError(format!("Failed to get Redis connection: {}", e)))?;

    let key = format!("game:{}:state", lobby_id);
    let json: Option<String> = conn.get(&key).await.map_err(AppError::RedisCommandError)?;

    match json {
        Some(json) => {
            let summary: GameSummary =
                serde_json::from_str(&json).map_err(|e| AppError::Serialization(e.to_string()))?;
            Ok(Some(summary))
        }
        None => Ok(None),
    }
}

// ============================================================================
// Wars Points Calculation
// ============================================================================

/// Context for calculating wars points and saving player results
///
/// Pass this to `save_player_result` to calculate wars points and persist results.
#[derive(Debug, Clone)]
pub struct WarsPointContext {
    /// The user being calculated for
    pub user_id: Uuid,
    /// Player's final rank (1 = winner)
    pub rank: usize,
    /// Prize amount won (calculated by game)
    pub prize: Option<f64>,
    /// Total number of participants in the game
    pub participants: usize,
    /// Entry amount per player (if any)
    pub entry_amount: Option<f64>,
    /// Total prize pool
    pub current_amount: Option<f64>,
    /// Whether this is a sponsored lobby
    pub is_sponsored: bool,
    /// The creator's user ID (for sponsor bonus)
    pub creator_id: Option<Uuid>,
    /// Number of active players remaining (for sponsor bonus calculation)
    pub active_players: usize,
}

/// Calculate wars points for a player
///
/// This is the standard formula used across all games:
/// - Base points: (participants - rank + 1) * 2
/// - Pool bonus (non-sponsored): (current_amount / participants) + (entry_amount / 5)
/// - Sponsor bonus (sponsored + creator): 2.5 * active_players
/// - Maximum cap: 50 points
pub fn calculate_wars_point(ctx: &WarsPointContext) -> f64 {
    // Base points: higher rank = more points
    let base_point = (ctx.participants.saturating_sub(ctx.rank).saturating_add(1) * 2) as f64;
    let mut total_point = base_point;

    // Pool bonus for non-sponsored games
    if !ctx.is_sponsored {
        if let (Some(entry_amount), Some(current_amount)) = (ctx.entry_amount, ctx.current_amount) {
            if entry_amount > 0.0 {
                let pool_bonus = (current_amount / ctx.participants as f64) + (entry_amount / 5.0);
                total_point += pool_bonus;
            }
        }
    }

    // Sponsor bonus if this is a sponsored lobby and the player is the creator
    if ctx.is_sponsored {
        if let Some(creator_id) = ctx.creator_id {
            if ctx.user_id == creator_id {
                let sponsor_bonus = 2.5 * ctx.active_players as f64;
                total_point += sponsor_bonus;
            }
        }
    }

    // Cap at 50 points maximum
    total_point.min(50.0)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_rotation() {
        let players = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let mut rotation = TurnRotation::new(players.clone());

        assert_eq!(rotation.current_player(), Some(players[0]));
        assert_eq!(rotation.active_count(), 3);

        rotation.next_turn();
        assert_eq!(rotation.current_player(), Some(players[1]));

        rotation.eliminate_player(players[1]);
        assert_eq!(rotation.active_count(), 2);
        assert_eq!(rotation.current_player(), Some(players[2]));

        rotation.eliminate_player(players[2]);
        assert_eq!(rotation.active_count(), 1);
        assert!(rotation.is_game_over());
        assert_eq!(rotation.get_winner(), Some(players[0]));
    }

    #[test]
    fn test_game_results() {
        let players = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let results = GameResults::from_ordered_players(players.clone());

        assert_eq!(results.rankings.len(), 3);
        assert_eq!(results.rankings[0].rank, 1);
        assert_eq!(results.rankings[0].user_id, players[0]);
        assert_eq!(results.rankings[2].rank, 3);
    }
}
