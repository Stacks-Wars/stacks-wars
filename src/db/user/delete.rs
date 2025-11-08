use crate::errors::AppError;
use uuid::Uuid;

use super::UserRepository;

impl UserRepository {
    /// Delete a user by ID (hard delete)
    ///
    /// **Warning**: This permanently deletes the user and all related data
    /// due to CASCADE foreign keys (wars_points, lobbies, etc.).
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user to delete
    ///
    /// # Returns
    /// * `Ok(())` - User successfully deleted
    /// * `Err(AppError::NotFound)` - User doesn't exist
    /// * `Err(AppError::DatabaseError)` - Delete operation failed
    ///
    /// # Examples
    /// ```rust,ignore
    /// repo.delete_user(user_id).await?;
    /// ```
    pub async fn delete_user(&self, user_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to delete user: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("User not found".into()));
        }

        tracing::warn!("Deleted user: {}", user_id);
        Ok(())
    }

    /// Delete a user by wallet address
    ///
    /// # Arguments
    /// * `wallet_address` - Wallet address of the user
    ///
    /// # Returns
    /// * `Ok(())` - User successfully deleted
    /// * `Err(AppError::NotFound)` - No user with that wallet
    pub async fn delete_user_by_wallet(&self, wallet_address: &str) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM users WHERE wallet_address = $1")
            .bind(wallet_address)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to delete user: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("User not found".into()));
        }

        tracing::warn!("Deleted user with wallet: {}", wallet_address);
        Ok(())
    }

    /// Bulk delete users by IDs
    ///
    /// Useful for administrative cleanup operations.
    ///
    /// # Arguments
    /// * `user_ids` - Vec of user IDs to delete
    ///
    /// # Returns
    /// * `Ok(u64)` - Number of users actually deleted
    /// * `Err(AppError::DatabaseError)` - Delete operation failed
    ///
    /// # Examples
    /// ```rust,ignore
    /// let deleted_count = repo.bulk_delete_users(vec![id1, id2, id3]).await?;
    /// println!("Deleted {} users", deleted_count);
    /// ```
    pub async fn bulk_delete_users(&self, user_ids: Vec<Uuid>) -> Result<u64, AppError> {
        if user_ids.is_empty() {
            return Ok(0);
        }

        let result = sqlx::query("DELETE FROM users WHERE id = ANY($1)")
            .bind(&user_ids)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to bulk delete users: {}", e)))?;

        let deleted = result.rows_affected();
        tracing::warn!("Bulk deleted {} users", deleted);
        Ok(deleted)
    }
}
