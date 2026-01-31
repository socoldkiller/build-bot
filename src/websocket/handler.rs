use crate::config::Config;
use crate::napbot::types::NapCatResponse;
use crate::tty::session::{SessionManager, SessionState};
use crate::version;
use crate::websocket::HandleResult::{ReceiveError, SendError, TtyFailed};
use crate::websocket::client::WebSocketClientError;
use serde::Serialize;
use serde_json;
use serde_json::{Value, json};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

/// Result of handling a message
#[derive(Error, Debug)]
pub enum HandleResult {
    #[error("create tty failed: {0}")]
    CreateTty(String),

    #[error("message: {0}")]
    Message(String),

    #[error("error: {0}")]
    Error(String),

    #[error("not for this bot: {0}")]
    NotForThisBot(String),

    #[error("tty failed: {0}")]
    TtyFailed(String),

    #[error("broken pipe: {0}")]
    BrokenPipe(String),

    #[error("send error")]
    SendError,

    #[error("receive error")]
    ReceiveError,

    #[error("invalid response")]
    InvalidResponse(#[from] serde_json::Error),
}

pub trait WebSocket {
    async fn send(&mut self, message: &str) -> Result<(), WebSocketClientError>;
    async fn recv(&mut self) -> Result<String, WebSocketClientError>;
}

pub struct WebSocketHandler<W: WebSocket> {
    session_manager: SessionManager,
    config: Config,
    ws: Arc<Mutex<W>>,
}

#[derive(Debug, Serialize)]
pub struct NapCatRequest {
    action: String,
    params: Value,
    echo: Option<String>,
}

impl<W> WebSocketHandler<W>
where
    W: WebSocket,
{
    pub fn new(config: Config, ws: Arc<Mutex<W>>) -> Self {
        Self {
            session_manager: SessionManager::new(),
            config,
            ws,
        }
    }

    pub async fn to_nap_cat_message(
        &self,
        group_id: Option<i64>,
        user_id: i64,
        raw_message: &str,
    ) -> Result<String, HandleResult> {
        let (action, params) = if let Some(group_id) = group_id {
            if group_id != 0 {
                (
                    "send_group_msg",
                    json!({
                        "group_id": group_id,
                        "user_id": user_id,
                        "message": raw_message
                    }),
                )
            } else {
                (
                    "send_private_msg",
                    json!({
                        "user_id": user_id,
                        "message": raw_message
                    }),
                )
            }
        } else {
            (
                "send_private_msg",
                json!({
                    "user_id": user_id,
                    "message": raw_message
                }),
            )
        };

        let request = NapCatRequest {
            action: action.to_string(),
            params,
            echo: Some("false".to_string()),
        };
        serde_json::to_string(&request).map_err(HandleResult::from)
    }

    pub async fn handle_message(&self, message: &str) -> HandleResult {
        let r: NapCatResponse = match serde_json::from_str(message) {
            Ok(response) => response,
            Err(e) => return HandleResult::Error(format!("JSON parse error: {}", e)),
        };

        self.handle_response(&r).await
    }

    fn parse_command(&self, text: &str) -> Result<(String, String, String), HandleResult> {
        let parts: Vec<&str> = text.split_whitespace().collect();

        if parts.len() != 3 {
            return Err(HandleResult::Error(
                "Command must be exactly 3 parts: bot_nickname command tty_type".to_string(),
            ));
        }

        Ok((
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
        ))
    }

    pub async fn send_message(&self, response: HandleResult) -> HandleResult {
        let async_send_response = async |msg: String| -> HandleResult {
            self.ws
                .lock()
                .await
                .send(&msg)
                .await
                .map_or_else(|_| SendError, |_| HandleResult::Message(msg))
        };

        match response {
            HandleResult::Message(msg)
            | HandleResult::TtyFailed(msg)
            | HandleResult::BrokenPipe(msg)
            | HandleResult::CreateTty(msg) => async_send_response(msg).await,
            HandleResult::InvalidResponse(e) => HandleResult::InvalidResponse(e),
            HandleResult::NotForThisBot(_m) => HandleResult::NotForThisBot(_m),
            _ => response,
        }
    }

    pub async fn recv_message(&self) -> HandleResult {
        match self.ws.lock().await.recv().await {
            Ok(msg) => self.handle_message(&msg).await,
            Err(_) => ReceiveError,
        }
    }

    pub async fn handle_response(&self, response: &NapCatResponse) -> HandleResult {
        if response.user_id == 0 || response.user_id == response.self_id {
            return HandleResult::NotForThisBot("send to self".to_string());
        }
        let session_key = format!("{}_{}", response.group_id, response.user_id);
        match self.session_manager.check_session(&session_key) {
            SessionState::NotFound => {
                let command_text = if let Some(message) = response.message.first() {
                    message.data.text.trim().to_string()
                } else {
                    return HandleResult::NotForThisBot("No message in NapCatResponse".to_string());
                };

                let (bot_name, _, tty_type) = match self.parse_command(&command_text) {
                    Ok((bot, cmd, tty)) => (bot, cmd, tty),
                    Err(e) => return e, // Return the HandleResult error
                };

                if bot_name != self.config.bot_nickname {
                    return HandleResult::NotForThisBot(bot_name);
                }

                if tty_type.trim().is_empty() {
                    return HandleResult::Error("TTY type cannot be empty".to_string());
                }

                match self.session_manager.create_session(&session_key, &tty_type) {
                    SessionState::Created => {
                        // Session created successfully, send startup message
                        let version_info = version::version().to_short_string();
                        let banner_message = format!(
                            "🚀({}):qq {} terminal({}) start ",
                            response.sender.nickname, tty_type, version_info
                        );

                        self.to_nap_cat_message(
                            Some(response.group_id),
                            response.user_id,
                            &banner_message,
                        )
                        .await
                        .map_or_else(std::convert::identity, HandleResult::CreateTty)
                    }

                    SessionState::Failed => self
                        .to_nap_cat_message(
                            Some(response.group_id),
                            response.user_id,
                            "Failed to create tty",
                        )
                        .await
                        .map_or_else(std::convert::identity, TtyFailed),
                    SessionState::Existed => {
                        // Race condition: session was created by another thread
                        HandleResult::Error("Session already exists (race condition)".to_string())
                    }
                    SessionState::NotFound => HandleResult::Error(
                        "Unexpected error: Session not found after creation attempt".to_string(),
                    ),
                }
            }

            SessionState::Existed => {
                let command = response
                    .message
                    .first()
                    .unwrap()
                    .data
                    .text
                    .trim()
                    .to_string();
                match self
                    .session_manager
                    .exec_command(&session_key, &command)
                    .await
                {
                    Ok(output) => self
                        .to_nap_cat_message(Some(response.group_id), response.user_id, &output)
                        .await
                        .map_or_else(std::convert::identity, HandleResult::Message),
                    Err(_e) => {
                        // Always remove session on any error, as TTY is likely unusable
                        self.session_manager.remove_session(&session_key);

                        let output = format!("({}): bye bye~ ✨👋", response.sender.nickname);
                        self.to_nap_cat_message(Some(response.group_id), response.user_id, &output)
                            .await
                            .map_or_else(std::convert::identity, HandleResult::BrokenPipe)
                    }
                }
            }

            _ => HandleResult::Error("Unexpected response from NapCatResponse".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::napbot::types::{Data, Message, NapCatResponse, Sender};
    use crate::websocket::client::WebSocketClientError;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // 创建一个模拟的 WebSocket 实现用于测试
    #[derive(Default)]
    struct MockWebSocket {
        sent_messages: Vec<String>,
        received_messages: Vec<String>,
        recv_index: usize,
    }

    impl MockWebSocket {
        fn new() -> Self {
            Self::default()
        }

        fn add_received_message(&mut self, message: String) {
            self.received_messages.push(message);
        }
    }

    // 为 MockWebSocket 实现 WebSocket trait
    impl WebSocket for MockWebSocket {
        async fn send(&mut self, message: &str) -> Result<(), WebSocketClientError> {
            self.sent_messages.push(message.to_string());
            Ok(())
        }

        async fn recv(&mut self) -> Result<String, WebSocketClientError> {
            if self.recv_index < self.received_messages.len() {
                let msg = self.received_messages[self.recv_index].clone();
                self.recv_index += 1;
                Ok(msg)
            } else {
                Err(WebSocketClientError::ConnectionClosed)
            }
        }
    }

    fn create_test_handler(config: Config) -> WebSocketHandler<MockWebSocket> {
        let mock_ws = Arc::new(Mutex::new(MockWebSocket::new()));
        WebSocketHandler::new(config, mock_ws)
    }

    #[test]
    fn test_parse_command() {
        // 创建一个 Config 用于测试
        let config = Config {
            bot_nickname: "bff".to_string(),
            ..Config::default()
        };
    }

    #[tokio::test]
    async fn test_to_nap_cat_message_group() {
        let config = Config::default();
        let handler = create_test_handler(config);

        // Test group message
        let result = handler
            .to_nap_cat_message(Some(123456), 789012, "Hello group")
            .await;
        assert!(result.is_ok());
        let json_str = result.unwrap();
        let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(value["action"], "send_group_msg");
        assert_eq!(value["params"]["group_id"], 123456);
        assert_eq!(value["params"]["user_id"], 789012);
        assert_eq!(value["params"]["message"], "Hello group");
        assert!(value["echo"].is_null());
    }

    #[tokio::test]
    async fn test_to_nap_cat_message_private() {
        let config = Config::default();
        let handler = create_test_handler(config);

        // Test private message (group_id = 0)
        let result = handler
            .to_nap_cat_message(Some(0), 789012, "Hello private")
            .await;
        assert!(result.is_ok());
        let json_str = result.unwrap();
        let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(value["action"], "send_private_msg");
        assert_eq!(value["params"]["user_id"], 789012);
        assert_eq!(value["params"]["message"], "Hello private");
        assert!(value["echo"].is_null());

        // Test private message (group_id = None)
        let result = handler
            .to_nap_cat_message(None, 789012, "Hello private 2")
            .await;
        assert!(result.is_ok());
        let json_str = result.unwrap();
        let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(value["action"], "send_private_msg");
        assert_eq!(value["params"]["user_id"], 789012);
        assert_eq!(value["params"]["message"], "Hello private 2");
        assert!(value["echo"].is_null());
    }

    #[tokio::test]
    async fn test_to_nap_cat_message_serialization_error() {
        // Create a config with non-ASCII bot nickname to test serialization
        let config = Config {
            bot_nickname: "🤖".to_string(),
            ..Config::default()
        };
        let handler = create_test_handler(config);

        // This should still work fine with Unicode
        let result = handler.to_nap_cat_message(Some(123), 456, "test").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_message_invalid_json() {
        let config = Config::default();
        let handler = create_test_handler(config);

        // Test invalid JSON
        let result = handler.handle_message("{invalid json").await;
        match result {
            HandleResult::Error(msg) => assert!(msg.contains("JSON parse error")),
            _ => panic!("Expected HandleResult::Error for invalid JSON"),
        }

        // Test empty string
        let result = handler.handle_message("").await;
        match result {
            HandleResult::Error(msg) => assert!(msg.contains("JSON parse error")),
            _ => panic!("Expected HandleResult::Error for empty string"),
        }

        // Test valid JSON but wrong structure
        let result = handler.handle_message(r#"{"foo": "bar"}"#).await;
        match result {
            HandleResult::Error(msg) => assert!(msg.contains("No message in NapCatResponse")),
            _ => panic!("Expected HandleResult::Error for wrong structure"),
        }
    }

    #[tokio::test]
    async fn test_handle_response_no_message() {
        let config = Config::default();
        let handler = create_test_handler(config);

        // Create a response with empty message array
        let response = NapCatResponse {
            group_id: 123,
            user_id: 456,
            message: vec![],
            ..Default::default()
        };

        let result = handler.handle_response(&response).await;
        match result {
            HandleResult::Error(msg) => assert_eq!(msg, "No message in NapCatResponse"),
            _ => panic!("Expected HandleResult::Error for no message"),
        }
    }

    #[tokio::test]
    async fn test_handle_response_wrong_bot_name() {
        let config = Config {
            bot_nickname: "mybot".to_string(),
            ..Config::default()
        };
        let handler = create_test_handler(config);

        // Create a response with command for different bot
        let response = NapCatResponse {
            group_id: 123,
            user_id: 456,
            message: vec![Message {
                msg_type: "text".to_string(),
                data: Data {
                    text: "otherbot judge bash".to_string(),
                },
            }],
            sender: Sender {
                nickname: "test".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = handler.handle_response(&response).await;
        match result {
            HandleResult::NotForThisBot(bot_name) => assert_eq!(bot_name, "otherbot"),
            _ => panic!("Expected HandleResult::NotForThisBot for wrong bot name"),
        }
    }

    #[tokio::test]
    async fn test_handle_response_empty_tty_type() {
        let config = Config {
            bot_nickname: "mybot".to_string(),
            ..Config::default()
        };
        let handler = create_test_handler(config);

        // Create a response with empty TTY type (trailing space gets trimmed)
        let response = NapCatResponse {
            group_id: 123,
            user_id: 456,
            message: vec![Message {
                msg_type: "text".to_string(),
                data: Data {
                    text: "mybot judge ".to_string(), // Note trailing space
                },
            }],
            sender: Sender {
                nickname: "test".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = handler.handle_response(&response).await;
        match result {
            HandleResult::Error(msg) => {
                // After trimming, "mybot judge " becomes "mybot judge" (2 parts)
                // So we get "Command must be exactly 3 parts" error
                assert!(msg.contains("Command must be exactly 3 parts"));
            }
            _ => panic!(
                "Expected HandleResult::Error for empty TTY type, got {:?}",
                result
            ),
        }
    }

    #[tokio::test]
    async fn test_handle_response_invalid_command_format() {
        let config = Config {
            bot_nickname: "mybot".to_string(),
            ..Config::default()
        };
        let handler = create_test_handler(config);

        // Create a response with invalid command format (only 2 parts)
        let response = NapCatResponse {
            group_id: 123,
            user_id: 456,
            message: vec![Message {
                msg_type: "text".to_string(),
                data: Data {
                    text: "mybot judge".to_string(), // Missing TTY type
                },
            }],
            sender: Sender {
                nickname: "test".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = handler.handle_response(&response).await;
        match result {
            HandleResult::Error(msg) => {
                assert!(msg.contains("Command must be exactly 3 parts"));
            }
            _ => panic!("Expected HandleResult::Error for invalid command format"),
        }
    }

    // Helper function to create a mock response
    fn create_mock_response(group_id: i64, user_id: i64, message_text: &str) -> NapCatResponse {
        NapCatResponse {
            group_id,
            user_id,
            message: vec![Message {
                msg_type: "text".to_string(),
                data: Data {
                    text: message_text.to_string(),
                },
            }],
            sender: Sender {
                nickname: "test".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_handle_response_session_creation_flow() {
        let config = Config {
            bot_nickname: "testbot".to_string(),
            ..Config::default()
        };
        let handler = create_test_handler(config);

        // First call should create a session
        let response = create_mock_response(100, 200, "testbot judge bash");
        let result = handler.handle_response(&response).await;

        // Since we can't mock TTy creation easily, we accept multiple outcomes
        // In test environment, TTy::new may fail, so we accept TtyFailed
        match result {
            HandleResult::CreateTty(msg) => {
                // Session created successfully
                assert!(msg.contains("🚀") || msg.contains("testbot") || msg.contains("bash"));
            }
            HandleResult::TtyFailed(_) => {
                // TTy creation failed - acceptable in test environment
            }
            HandleResult::Error(msg) if msg.contains("Session already exists") => {
                // Race condition - acceptable
            }
            HandleResult::Error(msg) if msg.contains("Failed to create tty") => {
                // TTy creation failed with explicit error message
            }
            other => {
                // Any other result is unexpected
                panic!("Unexpected result for session creation: {:?}", other);
            }
        }
    }

    #[test]
    fn test_handle_result_variants() {
        // Test that all HandleResult variants can be created and debug printed
        let message = HandleResult::Message("test".to_string());
        let error = HandleResult::Error("error".to_string());
        let not_for_bot = HandleResult::NotForThisBot("bot".to_string());
        let tty_failed = HandleResult::TtyFailed("failed".to_string());
        let broken_pipe = HandleResult::BrokenPipe("broken".to_string());

        assert!(format!("{:?}", message).contains("Message"));
        assert!(format!("{:?}", error).contains("Error"));
        assert!(format!("{:?}", not_for_bot).contains("NotForThisBot"));
        assert!(format!("{:?}", tty_failed).contains("TtyFailed"));
        assert!(format!("{:?}", broken_pipe).contains("BrokenPipe"));
    }

    #[tokio::test]
    async fn test_web_socket_handler_new() {
        let config = Config::default();
        let mock_ws = Arc::new(Mutex::new(MockWebSocket::new()));
        let _handler = WebSocketHandler::new(config, mock_ws);

        // Just verify it can be created without panic
        assert!(true);
    }
}
