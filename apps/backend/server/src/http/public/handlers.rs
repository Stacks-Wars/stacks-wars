use axum::Json;
use serde_json::{Value, json};

/// Health check endpoint — returns 200 OK if the service is running.
pub async fn health_handler() -> &'static str {
    "OK"
}

/// Root endpoint with API information.
pub async fn root_handler() -> Json<Value> {
    Json(json!({
        "name": "Stacks Wars API",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running"
    }))
}
