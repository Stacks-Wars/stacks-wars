use thiserror::Error;

/// Errors surfaced by [`crate::GameHost`] and [`crate::GameEngine`].
#[derive(Error, Debug, Clone)]
pub enum CoreError {
    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("deserialization error: {0}")]
    Deserialization(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("redis error: {0}")]
    Redis(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("internal error")]
    Internal,
}
