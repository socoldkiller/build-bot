use crate::config::Config;
use crate::napbot::types::NapCatResponse;
use crate::tty::session::{SessionManager, SessionState};
use crate::websocket::HandleResult::TtyFailed;
use serde::Serialize;
use serde_json;
use serde_json::{Value, json};

/// Result of handling a message
#[derive(Debug)]
pub enum HandleResult {
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
        let response: NapCatResponse = match serde_json::from_str(message) {
            Ok(response) => response,
            Err(e) => return HandleResult::Error(format!("JSON parse error: {}", e)),
        };

        self.handle_response(&response).await
    }

    fn parse_command(&self, text: &str) -> Result<(String, String, String), HandleResult> {
        let parts: Vec<&str> = text.split_whitespace().collect();

        if parts.len() != 3 {
            return Err(HandleResult::Error(
                "Command must be exactly 3 parts: bot_nickname command tty_type".to_string()
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

                let (bot_name, command, tty_type) = match self.parse_command(&command_text) {
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
                        let startup_message = format!(
                            "({}):qq {} terminal(rust) start",
                            self.config.bot_nickname,
                            tty_type
                        );

                        self.to_nap_cat_message(
                            Some(response.group_id),
                            response.user_id,
                            &startup_message,
                        )
                        .await
                        .map_or_else(std::convert::identity, HandleResult::Message)
                    }
                    SessionState::Failed => {
                        self.to_nap_cat_message(
                            Some(response.group_id),
                            response.user_id,
                            "Failed to create tty",
                        )
                        .await
                        .map_or_else(std::convert::identity, TtyFailed)
                    }
                    SessionState::Existed => {
                        // Race condition: session was created by another thread
                        HandleResult::Error("Session already exists (race condition)".to_string())
                    }
                    SessionState::NotFound => {
                        // Should not happen
                        HandleResult::Error("Unexpected error: Session not found after creation attempt".to_string())
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
                        // Check for broken pipe error
                        if e.kind() == std::io::ErrorKind::BrokenPipe {
                            // Remove the broken session so user can create a new one
                            self.session_manager.remove_session(&session_key);
                            let output = format!("({}):goodbye! (session removed due to broken pipe)", response.sender.nickname);
                            self.to_nap_cat_message(Some(response.group_id), response.user_id, &output)
                                .await
                                .map_or_else(std::convert::identity, HandleResult::BrokenPipe)
                        } else {
                            // Other errors
                            let output = format!("({}):goodbye!", response.sender.nickname);
                            self.to_nap_cat_message(Some(response.group_id), response.user_id, &output)
                                .await
                                .map_or_else(std::convert::identity, HandleResult::Error)
                        }
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

        // Test invalid commands (should return HandleResult::Error)
        assert!(handler.parse_command("").is_err());
        assert!(handler.parse_command("bff").is_err());
        assert!(handler.parse_command("bff judge").is_err());
        assert!(handler.parse_command("bff judge bash extra").is_err());
        assert!(handler.parse_command("ls").is_err());
    }
}
