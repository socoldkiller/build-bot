use crate::tty::tty::TTy;
use dashmap::DashMap;
use tokio::io;

type UserId = String;
type Session = TTy;

struct SessionManager {
    sessions: DashMap<UserId, Session>,
}

impl SessionManager {
    fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    async fn get_or_create<S: Into<String>, U: Into<String>>(
        &self,
        user_id: U,
        tty_type: S,
    ) -> Result<(), io::Error> {
        let user_id_str = user_id.into();
        if !self.sessions.contains_key(&user_id_str) {
            let tty = TTy::new(tty_type)?;
            self.sessions.insert(user_id_str, tty);
        }
        Ok(())
    }

    async fn write_to_user<S: Into<String>, U: Into<String>>(
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

    /// 从指定用户的 tty 读取数据
    async fn read_from_user<U: Into<String>>(&self, user_id: U) -> Result<String, io::Error> {
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

    async fn exec_command<S: Into<String>, U: Into<String>>(
        &self,
        user_id: U,
        command: S,
    ) -> Result<String, io::Error> {
        let user_id_str = user_id.into();
        self.write_to_user(&user_id_str, command).await?;
        self.read_from_user(&user_id_str).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_new() {
        // 使用字符串作为 TtyProvider
        let _manager = SessionManager::new();
    }

    #[tokio::test]
    async fn test_get_or_create() {
        let manager = SessionManager::new();

        let result = manager.get_or_create("test_user", "echo").await;
        assert!(result.is_ok());

        let result = manager.get_or_create("test_user", "echo").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tty_echo_hello_world() {
        let manager = SessionManager::new();

        // 创建会话
        assert!(
            manager
                .get_or_create("test_echo_user", "bash")
                .await
                .is_ok()
        );

        // 使用新的 write_to_user 方法
        let write_result = manager
            .write_to_user("test_echo_user", "echo hello world")
            .await;
        assert!(
            write_result.is_ok(),
            "write failed: {:?}",
            write_result.err()
        );

        // 使用新的 read_from_user 方法
        let read_result = manager.read_from_user("test_echo_user").await;
        assert!(read_result.is_ok(), "read failed: {:?}", read_result.err());

        let output = read_result.unwrap();
        // 输出应该包含 "hello world"
        assert!(
            output.contains("hello world"),
            "Output does not contain 'hello world': {}",
            output
        );
    }

    #[tokio::test]
    async fn test_exec_command() {
        let manager = SessionManager::new();

        // 创建会话
        assert!(
            manager
                .get_or_create("test_exec_user", "bash")
                .await
                .is_ok()
        );

        // 使用 exec_command 方法执行命令
        let result = manager
            .exec_command("test_exec_user", "echo test command")
            .await;
        assert!(result.is_ok(), "exec_command failed: {:?}", result.err());

        let output = result.unwrap();
        // 输出应该包含 "test command"
        assert!(
            output.contains("test command"),
            "Output does not contain 'test command': {}",
            output
        );
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

        // 尝试从不存在的用户读取数据，应该返回 NotFound 错误
        let result = manager.read_from_user("nonexistent_user").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    }
}
