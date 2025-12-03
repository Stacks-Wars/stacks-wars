// WebSocket connection helpers for send operations
use crate::state::ConnectionInfo;
use axum::extract::ws::Message;
use futures::SinkExt;
use serde::Serialize;
use std::sync::Arc;

/// Send a serializable message to the provided connection.
pub async fn send_json<M: Serialize>(
    conn: &Arc<ConnectionInfo>,
    msg: &M,
) -> Result<(), serde_json::Error> {
    let json = serde_json::to_string(msg)?;
    let mut s = conn.sender.lock().await;
    let _ = s.send(Message::Text(json.into())).await;
    Ok(())
}

/// Send raw text to the provided connection.
pub async fn send_text(conn: &Arc<ConnectionInfo>, text: String) {
    let mut s = conn.sender.lock().await;
    let _ = s.send(Message::Text(text.into())).await;
}
