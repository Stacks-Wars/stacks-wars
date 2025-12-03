/// Helper utilities for testing lobby and game flows
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

/// WebSocket connection wrapper for testing
pub struct WsConnection {
    sender: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    receiver: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
}

impl WsConnection {
    /// Connect to a lobby WebSocket with authentication
    pub async fn connect_to_lobby(
        base_url: &str,
        lobby_id: Uuid,
        token: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let ws_url = base_url.replace("http://", "ws://");
        let url = format!(
            "{}/lobby/{}?authorization=Bearer%20{}",
            ws_url, lobby_id, token
        );

        let request = tokio_tungstenite::tungstenite::http::Request::builder()
            .uri(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header(
                "Sec-WebSocket-Key",
                tokio_tungstenite::tungstenite::handshake::client::generate_key(),
            )
            .header("Sec-WebSocket-Version", "13")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header(
                "Host",
                url.split("//")
                    .nth(1)
                    .and_then(|s| s.split('/').next())
                    .unwrap_or("localhost"),
            )
            .body(())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let (ws_stream, _) = connect_async(request).await?;

        let (sender, receiver) = ws_stream.split();

        Ok(Self { sender, receiver })
    }

    /// Send a JSON message
    pub async fn send_json(&mut self, msg: &Value) -> Result<(), Box<dyn std::error::Error>> {
        self.sender
            .send(Message::Text(msg.to_string().into()))
            .await?;
        Ok(())
    }

    /// Receive the next JSON message
    pub async fn recv_json(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        if let Some(msg) = self.receiver.next().await {
            let msg = msg?;
            if let Message::Text(text) = msg {
                let value: Value = serde_json::from_str(&text)?;
                return Ok(value);
            }
        }
        Err("No message received".into())
    }

    /// Receive the next message with a timeout
    pub async fn recv_json_timeout(
        &mut self,
        timeout: std::time::Duration,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        tokio::time::timeout(timeout, self.recv_json())
            .await
            .map_err(|_| Box::<dyn std::error::Error>::from("Timeout waiting for message"))?
    }

    /// Close the WebSocket connection
    pub async fn close(mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.sender.close().await?;
        Ok(())
    }
}
