use axum::extract::ws::{Message, WebSocket};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use futures::stream::SplitSink;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::{collections::HashMap, sync::Arc, time::Duration};
use teloxide::Bot;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub connections: ConnectionInfoMap,
    pub chat_connections: ChatConnectionInfoMap,
    pub redis: RedisClient,
    pub postgres: PgPool,
    pub bot: Bot,
}

impl AppState {
    /// Create a new AppState by connecting to PostgreSQL and Redis
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Redis connection pool
        let redis_url = std::env::var("REDIS_URL")?;
        let manager = RedisConnectionManager::new(redis_url)?;

        let redis_pool = Pool::builder()
            .max_size(100)
            .min_idle(Some(20))
            .connection_timeout(Duration::from_secs(5))
            .max_lifetime(Some(Duration::from_secs(300)))
            .idle_timeout(Some(Duration::from_secs(30)))
            .build(manager)
            .await?;

        // PostgreSQL connection pool
        let postgres_url = std::env::var("DATABASE_URL")?;
        let postgres_pool = PgPoolOptions::new()
            .max_connections(50)
            .min_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(300))
            .max_lifetime(Duration::from_secs(1800))
            .connect(&postgres_url)
            .await?;

        // Bot
        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")?;
        let bot = Bot::new(bot_token);

        let connections: ConnectionInfoMap = Default::default();
        let chat_connections: ChatConnectionInfoMap = Default::default();

        Ok(Self {
            connections,
            chat_connections,
            redis: redis_pool,
            postgres: postgres_pool,
            bot,
        })
    }
}

#[derive(Debug)]
pub struct ConnectionInfo {
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

#[derive(Debug)]
pub struct ChatConnectionInfo {
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

pub type ConnectionInfoMap = Arc<Mutex<HashMap<Uuid, Arc<ConnectionInfo>>>>;

// Single chat connection per player, but track which lobby they're chatting in
pub type ChatConnectionInfoMap = Arc<Mutex<HashMap<Uuid, Arc<ChatConnectionInfo>>>>;

pub type RedisClient = Pool<RedisConnectionManager>;
