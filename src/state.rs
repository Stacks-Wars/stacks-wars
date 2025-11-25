use crate::games::GameFactory;
use axum::extract::ws::{Message, WebSocket};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use futures::stream::SplitSink;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use teloxide::Bot;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub redis_url: String,
    pub database_url: String,
    pub telegram_bot_token: String,
    pub telegram_chat_id: String,
}

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub connections: Connections,
    pub lobby_connections: LobbyConnections,
    pub chat_connections: ChatConnectionInfoMap,
    pub game_registry: Arc<HashMap<Uuid, GameFactory>>,
    pub redis: RedisClient,
    pub postgres: PgPool,
    pub bot: Bot,
}

impl AppState {
    /// Create a new AppState by connecting to PostgreSQL and Redis
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Read essential configuration from the environment and group it.
        let redis_url = std::env::var("REDIS_URL")?;
        let database_url = std::env::var("DATABASE_URL")?;
        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")?;
        let jwt_secret = std::env::var("JWT_SECRET")?;
        let telegram_chat_id = std::env::var("TELEGRAM_CHAT_ID")?;

        let config = AppConfig {
            jwt_secret,
            redis_url: redis_url.clone(),
            database_url: database_url.clone(),
            telegram_bot_token: bot_token.clone(),
            telegram_chat_id,
        };

        // Redis connection pool built from config.redis_url
        let manager = RedisConnectionManager::new(config.redis_url.clone())?;
        let redis_pool = Pool::builder()
            .max_size(100)
            .min_idle(Some(20))
            .connection_timeout(Duration::from_secs(5))
            .max_lifetime(Some(Duration::from_secs(300)))
            .idle_timeout(Some(Duration::from_secs(30)))
            .build(manager)
            .await?;

        // PostgreSQL connection pool built from config.database_url
        let postgres_pool = PgPoolOptions::new()
            .max_connections(50)
            .min_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(300))
            .max_lifetime(Duration::from_secs(1800))
            .connect(&config.database_url)
            .await?;

        // Bot
        let bot = Bot::new(config.telegram_bot_token.clone());

        let connections: Connections = Default::default();
        let lobby_connections: LobbyConnections = Default::default();
        let chat_connections: ChatConnectionInfoMap = Default::default();

        // Initialize empty game registry (games register themselves during startup)
        let game_registry: Arc<HashMap<Uuid, GameFactory>> = Arc::new(HashMap::new());

        Ok(Self {
            config,
            connections,
            lobby_connections,
            chat_connections,
            game_registry,
            redis: redis_pool,
            postgres: postgres_pool,
            bot,
        })
    }
}

#[derive(Debug)]
pub struct ChatConnectionInfo {
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

#[derive(Debug)]
pub struct ConnectionInfo {
    pub connection_id: Uuid,
    pub user_id: Option<Uuid>,
    pub lobby_id: Uuid,
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

/// Global map of all websocket connections keyed by `connection_id`.
pub type Connections = Arc<Mutex<HashMap<Uuid, Arc<ConnectionInfo>>>>;

/// For fast broadcasting: for each lobby_id we store the set of connection_ids.
pub type LobbyConnections = Arc<Mutex<HashMap<Uuid, HashSet<Uuid>>>>;

// Single chat connection per player, but track which lobby they're chatting in
pub type ChatConnectionInfoMap = Arc<Mutex<HashMap<Uuid, Arc<ChatConnectionInfo>>>>;

pub type RedisClient = Pool<RedisConnectionManager>;
