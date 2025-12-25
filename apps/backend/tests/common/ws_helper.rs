/// WebSocket helper utilities for testing
/// Provides connection wrappers for:
/// - /ws/room/:lobby_id
/// - /ws/lobbies
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::Message};

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
    /// Connect to a room (lobby + game) WebSocket with authentication
    pub async fn connect_to_room(
        base_url: &str,
        lobby_path: &str,
        token: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let ws_url = base_url.replace("http://", "ws://");
        let url = format!("{}/ws/room/{}", ws_url, lobby_path);

        let request = tokio_tungstenite::tungstenite::http::Request::builder()
            .uri(&url)
            .header("Cookie", format!("auth_token={}", token))
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

    pub async fn connect_to_lobby(
        base_url: &str,
        token: Option<&str>,
        status_filter: Option<&[&str]>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let ws_url = base_url.replace("http://", "ws://");
        let mut url = format!("{}/ws/lobbies", ws_url);

        let mut query_params = Vec::new();

        if let Some(statuses) = status_filter {
            query_params.push(format!("status={}", statuses.join(",")));
        }

        if !query_params.is_empty() {
            url.push('?');
            url.push_str(&query_params.join("&"));
        }

        let mut request_builder = tokio_tungstenite::tungstenite::http::Request::builder()
            .uri(&url)
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
            );

        if let Some(tok) = token {
            request_builder = request_builder.header("Cookie", format!("auth_token={}", tok));
        }

        let request = request_builder
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
