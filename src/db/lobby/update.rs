use chrono::Utc;
use sqlx::query;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{db::Lobby, redis::LobbyStatus},
};

use super::LobbyRepository;

impl LobbyRepository {
    /// Update lobby status
    ///
    /// # Status Flow
    /// Waiting → Starting → InProgress → Finished
    pub async fn update_status(
        &self,
        lobby_id: Uuid,
        status: LobbyStatus,
    ) -> Result<Lobby, AppError> {
        let lobby = sqlx::query_as::<_, Lobby>(
            r#"
            UPDATE lobbies
            SET status = $1, updated_at = $2
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(status)
        .bind(Utc::now().naive_utc())
        .bind(lobby_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update lobby status: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Lobby {} not found", lobby_id)))?;

        tracing::info!("Updated lobby {} status to {:?}", lobby_id, lobby.status);
        Ok(lobby)
    }

    /// Update lobby name
    pub async fn update_name(&self, lobby_id: Uuid, name: String) -> Result<Lobby, AppError> {
        let lobby = sqlx::query_as::<_, Lobby>(
            r#"
            UPDATE lobbies
            SET name = $1, updated_at = $2
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(&name)
        .bind(Utc::now().naive_utc())
        .bind(lobby_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update lobby name: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Lobby {} not found", lobby_id)))?;

        Ok(lobby)
    }

    /// Update lobby description
    pub async fn update_description(
        &self,
        lobby_id: Uuid,
        description: Option<String>,
    ) -> Result<Lobby, AppError> {
        let lobby = sqlx::query_as::<_, Lobby>(
            r#"
            UPDATE lobbies
            SET description = $1, updated_at = $2
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(description)
        .bind(Utc::now().naive_utc())
        .bind(lobby_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update lobby description: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Lobby {} not found", lobby_id)))?;

        Ok(lobby)
    }

    /// Update lobby entry amount
    pub async fn update_entry_amount(
        &self,
        lobby_id: Uuid,
        entry_amount: Option<f64>,
    ) -> Result<Lobby, AppError> {
        let lobby = sqlx::query_as::<_, Lobby>(
            r#"
            UPDATE lobbies
            SET entry_amount = $1, updated_at = $2
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(entry_amount)
        .bind(Utc::now().naive_utc())
        .bind(lobby_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to update lobby entry amount: {}", e))
        })?
        .ok_or_else(|| AppError::NotFound(format!("Lobby {} not found", lobby_id)))?;

        Ok(lobby)
    }

    /// Update current pool amount
    ///
    /// Used to track total value in the lobby pool as players join.
    pub async fn update_current_amount(
        &self,
        lobby_id: Uuid,
        current_amount: Option<f64>,
    ) -> Result<Lobby, AppError> {
        let lobby = sqlx::query_as::<_, Lobby>(
            r#"
            UPDATE lobbies
            SET current_amount = $1, updated_at = $2
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(current_amount)
        .bind(Utc::now().naive_utc())
        .bind(lobby_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to update lobby current amount: {}", e))
        })?
        .ok_or_else(|| AppError::NotFound(format!("Lobby {} not found", lobby_id)))?;

        Ok(lobby)
    }

    /// Increment current amount by a specific value
    ///
    /// Atomically adds to the pool when a player joins.
    pub async fn increment_current_amount(
        &self,
        lobby_id: Uuid,
        amount: f64,
    ) -> Result<Lobby, AppError> {
        let lobby = sqlx::query_as::<_, Lobby>(
            r#"
            UPDATE lobbies
            SET current_amount = COALESCE(current_amount, 0) + $1, updated_at = $2
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(amount)
        .bind(Utc::now().naive_utc())
        .bind(lobby_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to increment lobby amount: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Lobby {} not found", lobby_id)))?;

        Ok(lobby)
    }

    /// Update token information
    pub async fn update_token_info(
        &self,
        lobby_id: Uuid,
        token_symbol: Option<String>,
        token_contract_id: Option<String>,
    ) -> Result<Lobby, AppError> {
        let lobby = sqlx::query_as::<_, Lobby>(
            r#"
            UPDATE lobbies
            SET token_symbol = $1, token_contract_id = $2, updated_at = $3
            WHERE id = $4
            RETURNING *
            "#,
        )
        .bind(token_symbol)
        .bind(token_contract_id)
        .bind(Utc::now().naive_utc())
        .bind(lobby_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update lobby token info: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Lobby {} not found", lobby_id)))?;

        Ok(lobby)
    }

    /// Update contract address
    pub async fn update_contract_address(
        &self,
        lobby_id: Uuid,
        contract_address: Option<String>,
    ) -> Result<Lobby, AppError> {
        let lobby = sqlx::query_as::<_, Lobby>(
            r#"
            UPDATE lobbies
            SET contract_address = $1, updated_at = $2
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(contract_address)
        .bind(Utc::now().naive_utc())
        .bind(lobby_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to update lobby contract address: {}", e))
        })?
        .ok_or_else(|| AppError::NotFound(format!("Lobby {} not found", lobby_id)))?;

        Ok(lobby)
    }

    /// Toggle lobby privacy status
    pub async fn set_private(&self, lobby_id: Uuid, is_private: bool) -> Result<Lobby, AppError> {
        let lobby = sqlx::query_as::<_, Lobby>(
            r#"
            UPDATE lobbies
            SET is_private = $1, updated_at = $2
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(is_private)
        .bind(Utc::now().naive_utc())
        .bind(lobby_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update lobby privacy: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Lobby {} not found", lobby_id)))?;

        Ok(lobby)
    }

    /// Toggle lobby sponsored status
    pub async fn set_sponsored(
        &self,
        lobby_id: Uuid,
        is_sponsored: bool,
    ) -> Result<Lobby, AppError> {
        let lobby = sqlx::query_as::<_, Lobby>(
            r#"
            UPDATE lobbies
            SET is_sponsored = $1, updated_at = $2
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(is_sponsored)
        .bind(Utc::now().naive_utc())
        .bind(lobby_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to update lobby sponsored status: {}", e))
        })?
        .ok_or_else(|| AppError::NotFound(format!("Lobby {} not found", lobby_id)))?;

        Ok(lobby)
    }

    /// Bulk update lobbies to finished status
    ///
    /// Useful for cleanup operations or scheduled tasks.
    pub async fn mark_lobbies_as_finished(&self, lobby_ids: &[Uuid]) -> Result<u64, AppError> {
        if lobby_ids.is_empty() {
            return Ok(0);
        }

        let result = query(
            r#"
            UPDATE lobbies
            SET status = $1, updated_at = $2
            WHERE id = ANY($3)
            "#,
        )
        .bind(LobbyStatus::Finished)
        .bind(Utc::now().naive_utc())
        .bind(lobby_ids)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(format!("Failed to bulk update lobby statuses: {}", e))
        })?;

        Ok(result.rows_affected())
    }
}
