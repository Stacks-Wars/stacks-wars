use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::errors::AppError;

/// HTTP-facing API error with a stable JSON body.
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

#[derive(Serialize)]
struct ApiErrorBody {
    error: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        let (status, message) = err.to_response();
        Self { status, message }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorBody {
            error: self.message.clone(),
        };
        (self.status, Json(body)).into_response()
    }
}
