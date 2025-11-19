// Stacks Wars backend

pub mod auth;
pub mod db;
pub mod errors;
pub mod games;
pub mod http;
pub mod lobby;
mod middleware;
pub use middleware::cors_layer;
mod models;
pub mod state;
pub mod ws;

use axum::Router;
use state::AppState;
use std::net::SocketAddr;
use tokio::signal;

/// Start the HTTP API server
pub async fn start_server() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    // Initialize application state (PostgreSQL, Redis, Bot)
    let state = AppState::new()
        .await
        .expect("Failed to initialize application state");

    tracing::info!("PostgreSQL and Redis connection pools established");

    // Build HTTP router
    let app = Router::new()
        .merge(http::create_http_routes(state.clone()))
        // WebSocket routes (lobbies, games, bots, real-time endpoints)
        .merge(ws::create_ws_routes(state.clone()))
        .layer(cors_layer())
        .fallback(|| async { "404 Not Found" });

    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(3001);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("Failed to bind address");

    tracing::info!("Server listening on port {}", port);

    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal());

    if let Err(e) = server.await {
        tracing::error!("Server error: {}", e);
    }
}

/// Handle graceful shutdown on SIGTERM or Ctrl+C
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Ctrl+C received, shutting down");
        },
        _ = terminate => {
            tracing::info!("SIGTERM received, shutting down");
        },
    }
}
