use axum::http::StatusCode;
use redis::RedisError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Redis pool error: {0}")]
    RedisPoolError(String),

    #[error("Redis command error: {0}")]
    RedisCommandError(#[from] RedisError),

    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Env error: {0}")]
    EnvError(String),

    #[error("Internal server error")]
    InternalError,

    #[error("Not found")]
    NotFound(String),
}

impl AppError {
    pub fn to_response(&self) -> (StatusCode, String) {
        match self {
            AppError::RedisPoolError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.clone()),
            AppError::RedisCommandError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::JwtError(e) => (StatusCode::UNAUTHORIZED, e.to_string()),
            AppError::Serialization(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Deserialization(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::EnvError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error".into(),
            ),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
        }
    }
}
