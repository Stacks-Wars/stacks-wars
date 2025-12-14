use crate::{
    errors::AppError,
    models::game::{Game, Order, Pagination},
};
use uuid::Uuid;

use super::GameRepository;

impl GameRepository {
    /// Find a game by UUID.
    pub async fn find_by_id(&self, game_id: Uuid) -> Result<Game, AppError> {
        let game = sqlx::query_as::<_, Game>(
            "SELECT id, name, path, description, image_url, min_players, max_players, category,
                    creator_id, is_active, updated_at, created_at
            FROM games
            WHERE id = $1",
        )
        .bind(game_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to query game: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

        Ok(game)
    }

    /// Find a game by its path (URL-friendly identifier).
    pub async fn find_by_path(&self, path: &str) -> Result<Game, AppError> {
        let game = sqlx::query_as::<_, Game>(
            "SELECT id, name, path, description, image_url, min_players, max_players, category,
                    creator_id, is_active, updated_at, created_at
            FROM games
            WHERE path = $1",
        )
        .bind(path)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to query game by path: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

        tracing::info!("Found game by path: {}", game.id);

        Ok(game)
    }

    /// Find a game by its name.
    pub async fn find_by_name(&self, name: &str) -> Result<Game, AppError> {
        let game = sqlx::query_as::<_, Game>(
            "SELECT id, name, path, description, image_url, min_players, max_players, category,
                    creator_id, is_active, updated_at, created_at
            FROM games
            WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to query game by name: {}", e)))?
        .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

        Ok(game)
    }

    /// List all games with pagination and sort order.
    pub async fn get_all_games(
        &self,
        pagination: Pagination,
        order: Order,
    ) -> Result<Vec<Game>, AppError> {
        let offset = pagination.offset();
        let limit = pagination.limit;
        let order_sql = order.to_sql();

        let query = format!(
            "SELECT id, name, path, description, image_url, min_players, max_players, category,
                creator_id, is_active, updated_at, created_at
            FROM games
            ORDER BY created_at {}
            LIMIT $1 OFFSET $2",
            order_sql
        );

        let games = sqlx::query_as::<_, Game>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to fetch games: {}", e)))?;

        Ok(games)
    }

    /// List active games with pagination.
    pub async fn get_active_games(&self, pagination: Pagination) -> Result<Vec<Game>, AppError> {
        let offset = pagination.offset();
        let limit = pagination.limit;

        let games = sqlx::query_as::<_, Game>(
            "SELECT id, name, path, description, image_url, min_players, max_players, category,
                creator_id, is_active, updated_at, created_at
            FROM games
            WHERE is_active = TRUE
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch active games: {}", e)))?;

        Ok(games)
    }

    /// Get games in a category (limited).
    pub async fn get_by_category(&self, category: &str, limit: i64) -> Result<Vec<Game>, AppError> {
        let games = sqlx::query_as::<_, Game>(
            "SELECT id, name, path, description, image_url, min_players, max_players, category,
                creator_id, is_active, updated_at, created_at
            FROM games
            WHERE category = $1 AND is_active = TRUE
            ORDER BY created_at DESC
            LIMIT $2",
        )
        .bind(category)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to fetch games by category: {}", e))
        })?;

        Ok(games)
    }

    /// List games created by a user (limited).
    pub async fn get_by_creator(
        &self,
        creator_id: Uuid,
        limit: i64,
    ) -> Result<Vec<Game>, AppError> {
        let games = sqlx::query_as::<_, Game>(
            "SELECT id, name, path, description, image_url, min_players, max_players, category,
                creator_id, is_active, updated_at, created_at
            FROM games
            WHERE creator_id = $1
            ORDER BY created_at DESC
            LIMIT $2",
        )
        .bind(creator_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch games by creator: {}", e)))?;

        tracing::info!("Found {} games by creator: {}", games.len(), creator_id);

        Ok(games)
    }

    /// Count games; optionally only active ones.
    pub async fn count_games(&self, active_only: bool) -> Result<i64, AppError> {
        let query = if active_only {
            "SELECT COUNT(*) FROM games WHERE is_active = TRUE"
        } else {
            "SELECT COUNT(*) FROM games"
        };

        let count = sqlx::query_scalar::<_, i64>(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to count games: {}", e)))?;

        Ok(count)
    }

    /// Return whether a game exists by UUID.
    pub async fn exists(&self, game_id: Uuid) -> Result<bool, AppError> {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM games WHERE id = $1)")
                .bind(game_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to check game existence: {}", e))
                })?;

        Ok(exists)
    }

    /// Check whether a game name already exists.
    pub async fn name_exists(&self, name: &str) -> Result<bool, AppError> {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM games WHERE name = $1)")
                .bind(name)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(format!("Failed to check game name existence: {}", e))
                })?;

        Ok(exists)
    }
}
