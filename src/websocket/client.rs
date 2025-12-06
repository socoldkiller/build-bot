use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use thiserror::Error;
use tungstenite::Utf8Bytes;

#[derive(Error, Debug)]
pub enum WebSocketClientError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),

    #[error("Channel error: {0}")]
    Channel(String),

    #[error("Connection closed")]
    ConnectionClosed,

}

pub struct WebSocketClient {
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WebSocketClient {
    pub async fn connect(url: &str) -> Result<Self, WebSocketClientError> {
        let (ws_stream, _) = connect_async(url).await?;
        Ok(Self { ws_stream })
    }


    pub async fn send(&mut self, message: &str) -> Result<(), WebSocketClientError> {
        self.ws_stream
            .send(Message::Text(Utf8Bytes::from(message.to_string())))
            .await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<String, WebSocketClientError> {
        match self.ws_stream.next().await {
            Some(Ok(Message::Text(text))) => Ok(text.parse().unwrap()),
            Some(Ok(Message::Close(_))) => Err(WebSocketClientError::ConnectionClosed),
            Some(Err(e)) => Err(WebSocketClientError::WebSocket(e)),
            None => Err(WebSocketClientError::ConnectionClosed),
            _ => Err(WebSocketClientError::Channel("Unexpected message type".to_string())),
        }
    }


    pub async fn close(&mut self) -> Result<(), WebSocketClientError> {
        self.ws_stream.close(None).await?;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_client_creation() {
        let _client: Result<WebSocketClient, WebSocketClientError>;
    }
}
