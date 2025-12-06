use crate::config::Config;
use crate::napbot::types::NapCatResponse;
use crate::tty::session::{SessionManager, SessionState};
use crate::version;
use crate::websocket::HandleResult::TtyFailed;
use serde::Serialize;
use serde_json;
use serde_json::{Value, json};

/// Result of handling a message
#[derive(Debug)]
pub enum HandleResult {
    CreateTty(String),
    Message(String),
    Error(String),
    NotForThisBot(String),
    TtyFailed(String),
    BrokenPipe(String),
}

pub struct WebSocketHandler {
    session_manager: SessionManager,
    config: Config,
}

#[derive(Debug, Serialize)]
pub struct NapCatRequest {
    action: String,
    params: Value,
    echo: Option<String>,
}

impl WebSocketHandler {
    pub fn new(config: Config) -> Self {
        Self {
            session_manager: SessionManager::new(),
            config,
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
            echo: None,
        };
        serde_json::to_string(&request).map_err(|e| HandleResult::Error(e.to_string()))
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

    pub async fn handle_response(&self, response: &NapCatResponse) -> HandleResult {
        let session_key = format!("{}_{}", response.group_id, response.user_id);
        match self.session_manager.check_session(&session_key) {
            SessionState::NotFound => {
                let command_text = if let Some(message) = response.message.first() {
                    message.data.text.trim().to_string()
                } else {
                    return HandleResult::Error("No message in NapCatResponse".to_string());
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
                            response.sender.nickname , tty_type, version_info
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
                    SessionState::NotFound => {
                        // Should not happen
                        HandleResult::Error(
                            "Unexpected error: Session not found after creation attempt"
                                .to_string(),
                        )
                    }
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
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::BrokenPipe {
                            self.session_manager.remove_session(&session_key);
                        }

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

    #[test]
    fn test_parse_command() {
        let config = Config::default();
        let handler = WebSocketHandler::new(config);

        // Test valid three-part command
        let result = handler.parse_command("bff judge bash");
        assert!(result.is_ok());
        let (bot_nickname, command, tty_type) = result.unwrap();
        assert_eq!(bot_nickname, "bff");
        assert_eq!(command, "judge");
        assert_eq!(tty_type, "bash");

        // Test with extra whitespace
        let result = handler.parse_command("  bff   judge   bash  ");
        assert!(result.is_ok());
        let (bot_nickname, command, tty_type) = result.unwrap();
        assert_eq!(bot_nickname, "bff");
        assert_eq!(command, "judge");
        assert_eq!(tty_type, "bash");

        // Test with tabs (split_whitespace handles tabs)
        let result = handler.parse_command("bff\tjudge\tbash");
        assert!(result.is_ok());
        let (bot_nickname, command, tty_type) = result.unwrap();
        assert_eq!(bot_nickname, "bff");
        assert_eq!(command, "judge");
        assert_eq!(tty_type, "bash");

        // Test with mixed whitespace
        let result = handler.parse_command("bff\t judge \tbash");
        assert!(result.is_ok());
        let (bot_nickname, command, tty_type) = result.unwrap();
        assert_eq!(bot_nickname, "bff");
        assert_eq!(command, "judge");
        assert_eq!(tty_type, "bash");

        // Test invalid commands (should return HandleResult::Error)
        assert!(handler.parse_command("").is_err());
        assert!(handler.parse_command("bff").is_err());
        assert!(handler.parse_command("bff judge").is_err());
        assert!(handler.parse_command("bff judge bash extra").is_err());
        assert!(handler.parse_command("ls").is_err());

        // Test command with newlines (split_whitespace handles newlines)
        let result = handler.parse_command("bff\njudge\nbash");
        assert!(result.is_ok());
        let (bot_nickname, command, tty_type) = result.unwrap();
        assert_eq!(bot_nickname, "bff");
        assert_eq!(command, "judge");
        assert_eq!(tty_type, "bash");
    }

    #[tokio::test]
    async fn test_to_nap_cat_message_group() {
        let config = Config::default();
        let handler = WebSocketHandler::new(config);

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
        let handler = WebSocketHandler::new(config);

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
        let handler = WebSocketHandler::new(config);

        // This should still work fine with Unicode
        let result = handler.to_nap_cat_message(Some(123), 456, "test").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_message_invalid_json() {
        let config = Config::default();
        let handler = WebSocketHandler::new(config);

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
        let handler = WebSocketHandler::new(config);

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
        let handler = WebSocketHandler::new(config);

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
        let handler = WebSocketHandler::new(config);

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
        let handler = WebSocketHandler::new(config);

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
        let handler = WebSocketHandler::new(config);

        // First call should create a session
        let response = create_mock_response(100, 200, "testbot judge bash");
        let result = handler.handle_response(&response).await;

        // Since we can't mock TTy creation easily, we check for either success or failure
        // In real implementation, this would depend on whether TTy::new succeeds
        match result {
            HandleResult::CreateTty(msg) => {
                assert!(msg.contains("testbot") && msg.contains("bash"));
            }
            HandleResult::TtyFailed(_) => {
                // This is also acceptable if TTy creation fails in test environment
            }
            HandleResult::Error(msg) if msg.contains("Session already exists") => {
                // Race condition - acceptable
            }
            other => {
                panic!("Unexpected result for session creation: {:?}", other);
            }
        }

        // Second call with same user should find existing session
        // Note: This depends on whether the first call actually created a session
        // We'll test this separately with mocked dependencies
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
        let _handler = WebSocketHandler::new(config);

        // Just verify it can be created without panic
        assert!(true);
    }
}
