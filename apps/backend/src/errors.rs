use axum::http::StatusCode;
use redis::RedisError;
use thiserror::Error;

use crate::models::game::PlayerCountError;
use crate::models::lobby::LobbyAmountError;
use crate::models::season::DateRangeError;
use crate::models::username::UsernameError;
use crate::models::wallet_address::WalletAddressError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Redis error: {0}")]
    RedisError(String),

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

    #[error("Invalid Input: {0}")]
    InvalidInput(String),

    #[error("Invalid Input: {0}")]
    AlreadyExists(String),

    #[error("Env error: {0}")]
    EnvError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Internal server error")]
    InternalError,

    #[error("Not found")]
    NotFound(String),

    #[error("Invalid wallet address: {0}")]
    WalletAddressError(#[from] WalletAddressError),

    #[error("Invalid username: {0}")]
    UsernameError(#[from] UsernameError),

    #[error("Invalid date range: {0}")]
    DateRangeError(#[from] DateRangeError),

    #[error("Invalid player count: {0}")]
    PlayerCountError(#[from] PlayerCountError),

    #[error("Invalid lobby amount: {0}")]
    LobbyAmountError(#[from] LobbyAmountError),
}

impl AppError {
    pub fn to_response(&self) -> (StatusCode, String) {
        match self {
            AppError::RedisError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.clone()),
            AppError::RedisPoolError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.clone()),
            AppError::RedisCommandError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::JwtError(e) => (StatusCode::UNAUTHORIZED, e.to_string()),
            AppError::Serialization(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Deserialization(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::InvalidInput(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::AlreadyExists(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::EnvError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected server error".into(),
            ),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::WalletAddressError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::UsernameError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::DateRangeError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::PlayerCountError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::LobbyAmountError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        }
    }
}
