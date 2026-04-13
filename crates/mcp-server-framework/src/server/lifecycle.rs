use crate::server::MCPServer;
use thiserror::Error;

/// Server 生命周期错误
#[derive(Debug, Error)]
pub enum LifecycleError {
    #[error("Server is already {0}")]
    InvalidState(String),

    #[error("Server operation failed: {0}")]
    OperationFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Server 生命周期管理
#[async_trait::async_trait]
pub trait ServerLifecycle: MCPServer {
    /// 启动 Server
    async fn start(&mut self) -> Result<(), LifecycleError>;

    /// 停止 Server
    async fn stop(&mut self) -> Result<(), LifecycleError>;

    /// 排空 Server (优雅关闭)
    ///
    /// 停止接收新请求,处理完当前请求后关闭
    async fn drain(&mut self) -> Result<(), LifecycleError>;
}

/// Server 运行句柄
///
/// 用于管理 Server 的异步运行
pub struct ServerHandle<S: MCPServer> {
    server: std::sync::Arc<tokio::sync::RwLock<S>>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl<S: MCPServer + 'static> ServerHandle<S> {
    /// 创建新的 Server 句柄
    pub fn new(server: S) -> Self {
        Self {
            server: std::sync::Arc::new(tokio::sync::RwLock::new(server)),
            shutdown_tx: None,
        }
    }

    /// 启动 Server
    pub async fn start(&mut self) -> Result<(), LifecycleError> {
        // 发送启动信号
        let server = self.server.write().await;

        // 这里可以添加启动逻辑
        // 例如启动消息监听循环、Gossip 后台任务等

        tracing::info!("Server {} started", server.id());
        Ok(())
    }

    /// 停止 Server
    pub async fn stop(&mut self) -> Result<(), LifecycleError> {
        // 发送停止信号
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        let server = self.server.read().await;
        tracing::info!("Server {} stopped", server.id());
        Ok(())
    }

    /// 获取 Server 的共享引用
    pub fn server(&self) -> std::sync::Arc<tokio::sync::RwLock<S>> {
        self.server.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{MCPMessage, Tool, ToolCall, ToolResult};
    use async_trait::async_trait;
    use session_manager::ServerId;

    struct MockServer {
        id: ServerId,
    }

    impl MockServer {
        fn new(id: ServerId) -> Self {
            Self { id }
        }
    }

    #[async_trait]
    impl MCPServer for MockServer {
        fn id(&self) -> ServerId {
            self.id.clone()
        }

        fn tools(&self) -> Vec<Tool> {
            vec![Tool::new("echo", "Echo tool")]
        }

        async fn handle_tool_call(&self, call: ToolCall) -> ToolResult {
            ToolResult::text(format!("Echo: {}", call.arguments))
        }

        async fn on_message(&self, _msg: MCPMessage) -> Option<MCPMessage> {
            None
        }
    }

    #[async_trait]
    impl ServerLifecycle for MockServer {
        async fn start(&mut self) -> Result<(), LifecycleError> {
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), LifecycleError> {
            Ok(())
        }

        async fn drain(&mut self) -> Result<(), LifecycleError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_server_handle_start_stop() {
        let server_id = "server-1".to_string();
        let server = MockServer::new(server_id);
        let mut handle = ServerHandle::new(server);

        handle.start().await.unwrap();
        handle.stop().await.unwrap();
    }
}
