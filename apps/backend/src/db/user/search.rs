use crate::{errors::AppError, models::User};

use super::UserRepository;

/// Search filters for user queries.
#[derive(Debug, Clone, Default)]
pub struct UserSearchFilters {
    pub username_contains: Option<String>,
    pub min_trust_rating: Option<f64>,
    pub max_trust_rating: Option<f64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl UserRepository {
    /// Search for users with filters (pagination and trust-rating filters supported).
    pub async fn search_users(&self, filters: UserSearchFilters) -> Result<Vec<User>, AppError> {
        let mut query = String::from("SELECT id FROM users WHERE 1=1");
        let mut param_count = 0;

        // Build WHERE conditions
        if filters.username_contains.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND username ILIKE ${}", param_count));
        }
        if filters.min_trust_rating.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND trust_rating >= ${}", param_count));
        }
        if filters.max_trust_rating.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND trust_rating <= ${}", param_count));
        }

        query.push_str(" ORDER BY created_at DESC");

        // Add LIMIT
        if filters.limit.is_some() {
            param_count += 1;
            query.push_str(&format!(" LIMIT ${}", param_count));
        }

        // Add OFFSET
        if filters.offset.is_some() {
            param_count += 1;
            query.push_str(&format!(" OFFSET ${}", param_count));
        }

        // Bind parameters in order
        let mut query_builder = sqlx::query_scalar(&query);

        if let Some(ref username) = filters.username_contains {
            query_builder = query_builder.bind(format!("%{}%", username));
        }
        if let Some(min_rating) = filters.min_trust_rating {
            query_builder = query_builder.bind(min_rating);
        }
        if let Some(max_rating) = filters.max_trust_rating {
            query_builder = query_builder.bind(max_rating);
        }
        if let Some(limit) = filters.limit {
            query_builder = query_builder.bind(limit);
        }
        if let Some(offset) = filters.offset {
            query_builder = query_builder.bind(offset);
        }

        let user_ids: Vec<uuid::Uuid> = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to search users: {}", e)))?;

        // Fetch full user data for each ID
        let mut users = Vec::new();
        for user_id in user_ids {
            if let Ok(user) = self.find_by_id(user_id).await {
                users.push(user);
            }
        }

        Ok(users)
    }

    /// Get all users (paginated). Use limit/offset to avoid large results.
    pub async fn get_all_users(&self, limit: i64, offset: i64) -> Result<Vec<User>, AppError> {
        let user_ids: Vec<uuid::Uuid> =
            sqlx::query_scalar("SELECT id FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2")
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| AppError::DatabaseError(format!("Failed to get all users: {}", e)))?;

        let mut users = Vec::new();
        for user_id in user_ids {
            if let Ok(user) = self.find_by_id(user_id).await {
                users.push(user);
            }
        }

        Ok(users)
    }

    /// Count total users (useful for pagination metadata).
    pub async fn count_users(&self) -> Result<i64, AppError> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to count users: {}", e)))?;

        Ok(count)
    }

    /// Get users by trust rating range.
    pub async fn get_users_by_trust_rating_range(
        &self,
        min_rating: f64,
        max_rating: f64,
        limit: i64,
    ) -> Result<Vec<User>, AppError> {
        let filters = UserSearchFilters {
            min_trust_rating: Some(min_rating),
            max_trust_rating: Some(max_rating),
            limit: Some(limit),
            ..Default::default()
        };

        self.search_users(filters).await
    }
}
