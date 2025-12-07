use crate::games::{GameEngine, GameFactory, create_game_registry};
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

/// Active game engines by lobby ID
pub type ActiveGames = Arc<Mutex<HashMap<Uuid, Box<dyn GameEngine>>>>;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub connections: Connections,
    pub indices: Arc<Mutex<ConnectionIndices>>,
    pub game_registry: Arc<HashMap<Uuid, GameFactory>>,
    pub active_games: ActiveGames,
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
        let indices: Arc<Mutex<ConnectionIndices>> = Default::default();

        // Initialize game registry from games module
        let game_registry: Arc<HashMap<Uuid, GameFactory>> = Arc::new(create_game_registry());
        let active_games: ActiveGames = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            config,
            connections,
            indices,
            game_registry,
            active_games,
            redis: redis_pool,
            postgres: postgres_pool,
            bot,
        })
    }
}

/// Context type for WebSocket connections
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConnectionContext {
    /// Room connection for a specific lobby (game + chat)
    Room(Uuid),
    /// Lobby list connection with optional status filter
    Lobby(Option<Vec<String>>), // e.g., Some(vec!["waiting", "starting"])
}

impl ConnectionContext {
    /// Extract lobby_id if this is a Room context
    pub fn lobby_id(&self) -> Option<Uuid> {
        match self {
            ConnectionContext::Room(id) => Some(*id),
            ConnectionContext::Lobby(_) => None,
        }
    }
    
    /// Get context keys for indexing (can return multiple for status filters)
    pub fn context_keys(&self) -> Vec<String> {
        match self {
            ConnectionContext::Room(_) => vec!["room".to_string()],
            ConnectionContext::Lobby(Some(statuses)) => {
                // Create a key for each status filter
                statuses.iter()
                    .map(|status| format!("lobby:{}", status))
                    .collect()
            }
            ConnectionContext::Lobby(None) => vec!["lobby".to_string()],
        }
    }
}

#[derive(Debug)]
pub struct ConnectionInfo {
    pub connection_id: Uuid,
    pub user_id: Option<Uuid>,
    pub context: ConnectionContext,
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

impl ConnectionInfo {
    /// Get lobby_id if this connection is in a lobby context
    pub fn lobby_id(&self) -> Option<Uuid> {
        self.context.lobby_id()
    }
}

/// Global map of all websocket connections keyed by `connection_id`.
pub type Connections = Arc<Mutex<HashMap<Uuid, Arc<ConnectionInfo>>>>;

/// Connection indices for efficient lookups across different dimensions.
/// Enables O(1) broadcast operations by pre-indexing connections.
#[derive(Debug, Default)]
pub struct ConnectionIndices {
    /// Index by lobby_id -> set of connection_ids
    pub by_lobby: HashMap<Uuid, HashSet<Uuid>>,
    /// Index by user_id -> set of connection_ids (for multi-tab support)
    pub by_user: HashMap<Uuid, HashSet<Uuid>>,
    /// Index by context type -> set of connection_ids
    pub by_context: HashMap<String, HashSet<Uuid>>,
}

impl ConnectionIndices {
    /// Add a connection to all relevant indices
    pub fn insert(&mut self, conn: &ConnectionInfo) {
        // Index by lobby if applicable
        if let Some(lobby_id) = conn.lobby_id() {
            self.by_lobby
                .entry(lobby_id)
                .or_insert_with(HashSet::new)
                .insert(conn.connection_id);
        }

        // Index by user if authenticated
        if let Some(user_id) = conn.user_id {
            self.by_user
                .entry(user_id)
                .or_insert_with(HashSet::new)
                .insert(conn.connection_id);
        }

        // Index by context type(s) - can have multiple keys for status filters
        for context_key in conn.context.context_keys() {
            self.by_context
                .entry(context_key)
                .or_insert_with(HashSet::new)
                .insert(conn.connection_id);
        }
    }

    /// Remove a connection from all indices
    pub fn remove(&mut self, conn: &ConnectionInfo) {
        // Remove from lobby index
        if let Some(lobby_id) = conn.lobby_id() {
            if let Some(set) = self.by_lobby.get_mut(&lobby_id) {
                set.remove(&conn.connection_id);
                if set.is_empty() {
                    self.by_lobby.remove(&lobby_id);
                }
            }
        }

        // Remove from user index
        if let Some(user_id) = conn.user_id {
            if let Some(set) = self.by_user.get_mut(&user_id) {
                set.remove(&conn.connection_id);
                if set.is_empty() {
                    self.by_user.remove(&user_id);
                }
            }
        }

        // Remove from context index(es)
        for context_key in conn.context.context_keys() {
            if let Some(set) = self.by_context.get_mut(&context_key) {
                set.remove(&conn.connection_id);
                if set.is_empty() {
                    self.by_context.remove(&context_key);
                }
            }
        }
    }

    /// Remove all connections for a lobby and return count removed
    pub fn remove_lobby(&mut self, lobby_id: Uuid) -> usize {
        if let Some(set) = self.by_lobby.remove(&lobby_id) {
            set.len()
        } else {
            0
        }
    }

    /// Get all connection_ids for a lobby
    pub fn get_lobby_connections(&self, lobby_id: &Uuid) -> Option<&HashSet<Uuid>> {
        self.by_lobby.get(lobby_id)
    }

    /// Get all connection_ids for a user
    pub fn get_user_connections(&self, user_id: &Uuid) -> Option<&HashSet<Uuid>> {
        self.by_user.get(user_id)
    }

    /// Get all connection_ids for a specific context type
    pub fn get_context_connections(&self, context: &str) -> Option<&HashSet<Uuid>> {
        self.by_context.get(context)
    }
}

pub type RedisClient = Pool<RedisConnectionManager>;
