use crate::{
    errors::AppError,
    models::db::game::{Game, Order, Pagination},
};
use uuid::Uuid;

use super::GameRepository;

impl GameRepository {
    /// Find a game by ID
    ///
    /// # Arguments
    /// * `game_id` - UUID of the game
    ///
    /// # Returns
    /// * `Ok(Game)` - Game data
    /// * `Err(AppError::NotFound)` - Game doesn't exist
    ///
    /// # Examples
    /// ```rust,ignore
    /// let game = repo.find_by_id(game_id).await?;
    /// println!("Game: {}", game.name);
    /// ```
    pub async fn find_by_id(&self, game_id: Uuid) -> Result<Game, AppError> {
        let game = sqlx::query_as::<_, Game>(
            "SELECT id, name, description, image_url, min_players, max_players, category,
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

    /// Find a game by name
    ///
    /// # Arguments
    /// * `name` - Game name (case-sensitive)
    ///
    /// # Returns
    /// * `Ok(Game)` - Game data
    /// * `Err(AppError::NotFound)` - Game doesn't exist
    pub async fn find_by_name(&self, name: &str) -> Result<Game, AppError> {
        let game = sqlx::query_as::<_, Game>(
            "SELECT id, name, description, image_url, min_players, max_players, category,
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

    /// Get all games with pagination
    ///
    /// # Arguments
    /// * `pagination` - Page and limit
    /// * `order` - Sort order (Ascending or Descending by created_at)
    ///
    /// # Returns
    /// * `Ok(Vec<Game>)` - List of games
    ///
    /// # Examples
    /// ```rust,ignore
    /// use crate::models::game::{Pagination, Order};
    ///
    /// let pagination = Pagination { page: 1, limit: 20 };
    /// let games = repo.get_all_games(pagination, Order::Descending).await?;
    /// ```
    pub async fn get_all_games(
        &self,
        pagination: Pagination,
        order: Order,
    ) -> Result<Vec<Game>, AppError> {
        let offset = pagination.offset();
        let limit = pagination.limit;
        let order_sql = order.to_sql();

        let query = format!(
            "SELECT id, name, description, image_url, min_players, max_players, category,
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

    /// Get active games only
    ///
    /// # Arguments
    /// * `pagination` - Page and limit
    ///
    /// # Returns
    /// * `Ok(Vec<Game>)` - Active games
    pub async fn get_active_games(&self, pagination: Pagination) -> Result<Vec<Game>, AppError> {
        let offset = pagination.offset();
        let limit = pagination.limit;

        let games = sqlx::query_as::<_, Game>(
            "SELECT id, name, description, image_url, min_players, max_players, category,
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

    /// Get games by category
    ///
    /// # Arguments
    /// * `category` - Game category
    /// * `limit` - Maximum results
    ///
    /// # Returns
    /// * `Ok(Vec<Game>)` - Games in category
    pub async fn get_by_category(&self, category: &str, limit: i64) -> Result<Vec<Game>, AppError> {
        let games = sqlx::query_as::<_, Game>(
            "SELECT id, name, description, image_url, min_players, max_players, category,
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

    /// Get games by creator
    ///
    /// # Arguments
    /// * `creator_id` - Creator user ID
    /// * `limit` - Maximum results
    ///
    /// # Returns
    /// * `Ok(Vec<Game>)` - Games created by user
    pub async fn get_by_creator(
        &self,
        creator_id: Uuid,
        limit: i64,
    ) -> Result<Vec<Game>, AppError> {
        let games = sqlx::query_as::<_, Game>(
            "SELECT id, name, description, image_url, min_players, max_players, category,
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

        Ok(games)
    }

    /// Count total games
    ///
    /// # Arguments
    /// * `active_only` - If true, count only active games
    ///
    /// # Returns
    /// * `Ok(i64)` - Number of games
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

    /// Check if a game exists by ID
    ///
    /// # Arguments
    /// * `game_id` - Game ID
    ///
    /// # Returns
    /// * `Ok(bool)` - true if game exists
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

    /// Check if a game name is taken
    ///
    /// # Arguments
    /// * `name` - Game name
    ///
    /// # Returns
    /// * `Ok(bool)` - true if name exists
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
