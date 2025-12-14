use crate::state::AppState;
use axum::{Json, Router, routing::get};
use serde_json::{Value, json};

/// Public routes - no authentication or rate limiting required
///
/// These routes are for health checks, metrics, and other public endpoints.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_handler))
        .route("/", get(root_handler))
}

/// Health check endpoint
///
/// Returns 200 OK if the service is running.
async fn health_handler() -> &'static str {
    "OK"
}

/// Root endpoint with API information
async fn root_handler() -> Json<Value> {
    Json(json!({
        "name": "Stacks Wars API",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running"
    }))
}
