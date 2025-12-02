use axum::extract::ws::{Message, WebSocket};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use futures::stream::SplitSink;
use std::{collections::HashMap, sync::Arc};
use teloxide::Bot;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub connections: ConnectionInfoMap,
    pub chat_connections: ChatConnectionInfoMap,
    pub redis: RedisClient,
    pub bot: Bot,
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
