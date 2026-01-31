use crate::tty::tty::BotTTy;
use dashmap::DashMap;
use tokio::io;

pub type UserId = String;
pub type Session = BotTTy;

/// Session creation status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    NotFound,
    Created,
    Existed,
    Failed,
}

pub struct SessionManager {
    sessions: DashMap<UserId, Session>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    pub fn check_session<U: Into<String>>(&self, user_id: U) -> SessionState {
        let user_id_str = user_id.into();
        let state = self
            .sessions
            .get(&user_id_str)
            .map_or(SessionState::NotFound, |_| SessionState::Existed);
        state
    }

    /// Create a new session for the user
    pub fn create_session<S: Into<String>, U: Into<String>>(
        &self,
        user_id: U,
        tty_type: S,
    ) -> SessionState {
        let user_id_str = user_id.into();
        self.sessions.get(&user_id_str).map_or_else(
            || {
                let tty = BotTTy::new(tty_type);
                match tty {
                    Ok(tty) => {
                        self.sessions.insert(user_id_str.clone(), tty);
                        SessionState::Created
                    }
                    Err(_) => SessionState::Failed,
                }
            },
            |_| SessionState::Existed,
        )
    }

    pub async fn write_to_user<S: Into<String>, U: Into<String>>(
        &self,
        user_id: U,
        data: S,
    ) -> Result<(), io::Error> {
        let user_id_str = user_id.into();

        match self.sessions.get_mut(&user_id_str) {
            Some(mut session) => {
                session.write(data).await?;
                Ok(())
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Session not found for user: {}", user_id_str),
            )),
        }
    }

    pub async fn read_from_user<U: Into<String>>(&self, user_id: U) -> Result<String, io::Error> {
        let user_id_str = user_id.into();

        match self.sessions.get_mut(&user_id_str) {
            Some(mut tty) => {
                let output = tty.read().await?;
                Ok(output)
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Session not found for user: {}", user_id_str),
            )),
        }
    }

    pub async fn exec_command<S: Into<String>, U: Into<String>>(
        &self,
        user_id: U,
        command: S,
    ) -> Result<String, io::Error> {
        let user_id_str = user_id.into();
        self.write_to_user(&user_id_str, command).await?;
        self.read_from_user(&user_id_str).await
    }

    /// Remove a session for a user
    pub fn remove_session<U: Into<String>>(&self, user_id: U) -> bool {
        let user_id_str = user_id.into();
        self.sessions.remove(&user_id_str).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_new() {
        let _manager = SessionManager::new();
    }

    #[tokio::test]
    async fn test_write_to_user_not_found() {
        let manager = SessionManager::new();

        // 尝试向不存在的用户写入数据，应该返回 NotFound 错误
        let result = manager.write_to_user("nonexistent_user", "echo test").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    }

    #[tokio::test]
    async fn test_read_from_user_not_found() {
        let manager = SessionManager::new();
        let result = manager.read_from_user("nonexistent_user").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    }
}
