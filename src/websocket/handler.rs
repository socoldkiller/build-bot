use crate::napbot::types::NapCatResponse;
use crate::tty::session::SessionManager;
use serde::Serialize;
use serde_json;
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebSocketHandlerError {
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Session error: {0}")]
    Session(#[from] std::io::Error),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

pub struct WebSocketHandler {
    session_manager: SessionManager,
}

#[derive(Debug, Serialize)]
pub struct NapCatRequest {
    action: String,
    params: Value,
    echo: Option<String>,
}


impl WebSocketHandler {
    pub fn new() -> Self {
        Self {
            session_manager: SessionManager::new(),
        }
    }

    pub async fn to_nap_cat_message(
        &self,
        group_id: Option<i64>,
        user_id: i64,
        raw_message: &str,
    ) -> Result<String, WebSocketHandlerError> {

        let (action, params) = if let Some(group_id) = group_id {

            if group_id != 0 {
                (
                    "send_group_msg",
                    json!({
                        "group_id": group_id,
                        "user_id": user_id,
                        "message": raw_message
                    })
                )
            } else {
                (
                    "send_private_msg",
                    json!({
                        "user_id": user_id,
                        "message": raw_message
                    })
                )
            }
        } else {
            (
                "send_private_msg",
                json!({
                    "user_id": user_id,
                    "message": raw_message
                })
            )
        };

        let request = NapCatRequest{
            action:action.to_string(),
            params,
            echo: None,
        };
        let json_data = serde_json::to_string(&request)?;
        Ok(json_data)
    }

    pub async fn handle_message(&self, message: &str) -> Result<String, WebSocketHandlerError> {
        let response: NapCatResponse = serde_json::from_str(message)?;
        self.handle_response(&response).await
    }

    pub async fn handle_response(
        &self,
        response: &NapCatResponse,
    ) -> Result<String, WebSocketHandlerError> {
        let session_key = format!("{}_{}", response.group_id, response.user_id);

        self.session_manager
            .get_or_create(&session_key, "bash")
            .await
            .map_err(|e| WebSocketHandlerError::Session(e))?;

        let command = if let Some(message) = response.message.first() {
            message.data.text.clone()
        } else {
            return Err(WebSocketHandlerError::InvalidMessage(
                "No message in NapCatResponse".to_string(),
            ));
        };

        let output = self
            .session_manager
            .exec_command(&session_key, &command)
            .await
            .map_err(|e| WebSocketHandlerError::Session(e))?;

        self.to_nap_cat_message(Some(response.group_id), response.user_id, output.as_str())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;
    #[tokio::test]
    async fn test_handle_message() {
        let handler = WebSocketHandler::new();
        let message = fs::read_to_string("./test_napcat_response.json")
            .await
            .unwrap();
        let output = handler.handle_message(message.as_str()).await.unwrap();
        println!("{}", output);
    }
}
