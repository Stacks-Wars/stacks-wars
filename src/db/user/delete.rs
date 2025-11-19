use crate::errors::AppError;
use uuid::Uuid;

use super::UserRepository;

impl UserRepository {
    /// Delete a user by ID (hard delete; cascades to related data).
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

    /// Delete a user by wallet address.
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

    /// Bulk delete users by IDs; returns number deleted.
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
