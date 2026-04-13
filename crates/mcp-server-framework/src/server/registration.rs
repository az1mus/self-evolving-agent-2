use crate::server::MCPServer;
use session_manager::{SessionId, SessionManager};
use thiserror::Error;

/// Server 注册错误
#[derive(Debug, Error)]
pub enum RegistrationError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Server already registered: {0}")]
    AlreadyRegistered(String),

    #[error("Registration failed: {0}")]
    Failed(String),
}

/// 将 Server 注册到 Session
///
/// # 参数
/// - `server`: 要注册的 Server
/// - `session_manager`: Session 管理器
/// - `session_id`: Session ID
///
/// # 返回
/// 成功返回 Ok(())
pub async fn register_to_session<S: MCPServer>(
    server: &S,
    session_manager: &SessionManager,
    session_id: SessionId,
) -> Result<(), RegistrationError> {
    let server_id = server.id();
    let tools = server.tools();
    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();

    // 加载 Session
    let session = session_manager
        .load_session(session_id)
        .map_err(|e| RegistrationError::SessionNotFound(e.to_string()))?;

    // 使用 ServerLifecycle 注册 Server
    let lifecycle = session_manager::ServerLifecycle::new(session_manager);

    lifecycle
        .register_server(
            session.session_id,
            server_id.clone(),
            tool_names,
            std::collections::HashMap::new(),
        )
        .map_err(|e| RegistrationError::Failed(e.to_string()))?;

    tracing::info!("Server {} registered with {} tools", server_id, tools.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{MCPMessage, Tool, ToolCall, ToolResult};
    use async_trait::async_trait;
    use session_manager::ServerId;

    struct TestServer {
        id: ServerId,
    }

    impl TestServer {
        fn new() -> Self {
            Self {
                id: "test-server-1".to_string(),
            }
        }
    }

    #[async_trait]
    impl MCPServer for TestServer {
        fn id(&self) -> ServerId {
            self.id.clone()
        }

        fn tools(&self) -> Vec<Tool> {
            vec![
                Tool::new("tool1", "First tool"),
                Tool::new("tool2", "Second tool"),
            ]
        }

        async fn handle_tool_call(&self, _call: ToolCall) -> ToolResult {
            ToolResult::text("ok")
        }

        async fn on_message(&self, _msg: MCPMessage) -> Option<MCPMessage> {
            None
        }
    }

    #[tokio::test]
    async fn test_register_to_session() {
        // 创建临时目录
        let temp_dir = tempfile::tempdir().unwrap();
        let session_manager = SessionManager::new(temp_dir.path());

        // 创建并初始化 session
        let session = session_manager.create_session().unwrap();

        let server = TestServer::new();
        let result = register_to_session(&server, &session_manager, session.session_id).await;

        assert!(result.is_ok());
    }
}
