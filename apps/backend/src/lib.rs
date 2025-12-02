pub mod auth;
mod db;
pub mod errors;
pub mod games;
mod http;
mod middleware;
mod models;
mod state;
pub mod ws;

use axum::{Router, middleware as axum_middleware};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use middleware::{cors_layer, create_global_rate_limiter, rate_limit_middleware};
use state::{AppState, ChatConnectionInfoMap, ConnectionInfoMap};
use std::{net::SocketAddr, time::Duration};
use teloxide::{Bot, prelude::*};
use tokio::signal;

use crate::{
    games::init::initialize_games,
    http::bot_commands::{Command, handle_command},
};

pub async fn start_server() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let manager = RedisConnectionManager::new(redis_url).unwrap();

    let bot_token = std::env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN must be set");
    let bot = Bot::new(bot_token);

    let redis_pool = Pool::builder()
        .max_size(100)
        .min_idle(Some(20))
        .connection_timeout(Duration::from_secs(5))
        .max_lifetime(Some(Duration::from_secs(300)))
        .idle_timeout(Some(Duration::from_secs(30)))
        .build(manager)
        .await
        .unwrap();

    // Initialize games in database
    if let Err(e) = initialize_games(redis_pool.clone()).await {
        tracing::error!("Failed to initialize games: {}", e);
        panic!("Failed to initialize games: {}", e);
    }

    let connections: ConnectionInfoMap = Default::default();
    let chat_connections: ChatConnectionInfoMap = Default::default();
    let state = AppState {
        connections,
        chat_connections,
        redis: redis_pool.clone(),
        bot: bot.clone(),
    };

    // Start Telegram bot command handler
    let bot_clone = bot.clone();
    let redis_clone = redis_pool.clone();
    tokio::spawn(async move {
        start_bot_command_handler(bot_clone, redis_clone).await;
    });

    // Create rate limiters
    let global_rate_limiter = create_global_rate_limiter();

    let app = Router::new()
        .merge(http::create_http_routes(state.clone()))
        .merge(ws::create_ws_routes(state))
        .layer(axum_middleware::from_fn(move |req, next| {
            rate_limit_middleware(global_rate_limiter.clone(), req, next)
        }))
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

async fn start_bot_command_handler(bot: Bot, redis: bb8::Pool<RedisConnectionManager>) {
    tracing::info!("Starting Telegram bot command handler");

    let handler = Update::filter_message()
        .filter_command::<Command>()
        .endpoint(move |bot: Bot, msg: Message, cmd: Command| {
            let redis_clone = redis.clone();
            async move { handle_command(bot, msg, cmd, redis_clone).await }
        });

    Dispatcher::builder(bot, handler)
        .default_handler(|_| async {})
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
