use crate::gossip::{GossipHandler, HeartbeatConfig, HeartbeatManager};
use crate::messaging::MessageDispatcher;
use crate::protocol::MCPMessage;
use crate::runtime::{EventBus, ServerConfig, ServerMetrics};
use crate::server::{MCPServer, ServerBase, ServerState};
use crate::topology::{LocalTopologyState, TopologyQuery};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};

/// Server 运行错误
#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("Server already running")]
    AlreadyRunning,

    #[error("Server not started")]
    NotStarted,

    #[error("Runtime error: {0}")]
    Runtime(String),
}

/// Server 运行器
///
/// 管理整个 Server 的异步运行生命周期
pub struct ServerRunner<S: MCPServer> {
    server: Arc<RwLock<S>>,
    server_base: Arc<RwLock<ServerBase>>,
    config: ServerConfig,
    topology: Arc<RwLock<LocalTopologyState>>,
    event_bus: EventBus,
    metrics: Arc<ServerMetrics>,
    gossip_tx: mpsc::Sender<crate::gossip::GossipMessage>,
    gossip_rx: Option<mpsc::Receiver<crate::gossip::GossipMessage>>,
    mcp_tx: mpsc::Sender<MCPMessage>,
    mcp_rx: Option<mpsc::Receiver<MCPMessage>>,
}

impl<S: MCPServer + 'static> ServerRunner<S> {
    /// 创建新的 Server 运行器
    pub fn new(server: S, config: ServerConfig) -> Self {
        let server_base = Arc::new(RwLock::new(ServerBase::new(
            config.server_id.clone(),
            config.session_id,
        )));

        let topology = Arc::new(RwLock::new(LocalTopologyState::new()));
        let event_bus = EventBus::default();
        let metrics = Arc::new(ServerMetrics::new());

        let (gossip_tx, gossip_rx) = mpsc::channel(256);
        let (mcp_tx, mcp_rx) = mpsc::channel(256);

        Self {
            server: Arc::new(RwLock::new(server)),
            server_base,
            config,
            topology,
            event_bus,
            metrics,
            gossip_tx,
            gossip_rx: Some(gossip_rx),
            mcp_tx,
            mcp_rx: Some(mcp_rx),
        }
    }

    /// 获取消息发送通道
    pub fn mcp_sender(&self) -> mpsc::Sender<MCPMessage> {
        self.mcp_tx.clone()
    }

    /// 获取 Gossip 消息发送通道
    pub fn gossip_sender(&self) -> mpsc::Sender<crate::gossip::GossipMessage> {
        self.gossip_tx.clone()
    }

    /// 获取事件总线
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// 获取指标
    pub fn metrics(&self) -> &Arc<ServerMetrics> {
        &self.metrics
    }

    /// 启动 Server
    pub async fn start(&mut self) -> Result<(), RunnerError> {
        // 更新状态
        {
            let base = self.server_base.read().await;
            base.set_state(ServerState::Active).await;
        }

        // 发布启动事件
        self.event_bus
            .publish(crate::runtime::ServerEvent::ServerStarted {
                server_id: self.config.server_id.clone(),
            });

        // 启动 Heartbeat 后台任务
        let heartbeat_config = HeartbeatConfig {
            interval_secs: self.config.gossip.heartbeat_interval_secs,
            timeout_secs: self.config.gossip.heartbeat_timeout_secs,
        };
        let heartbeat_manager =
            HeartbeatManager::new(self.config.server_id.clone(), heartbeat_config);
        let _heartbeat_handle = heartbeat_manager.start_heartbeat_task(self.gossip_tx.clone());

        // 启动消息分发器
        let topology_query = TopologyQuery::new(self.topology.clone());
        let mcp_rx = self.mcp_rx.take().ok_or(RunnerError::AlreadyRunning)?;
        let gossip_rx = self.gossip_rx.take().ok_or(RunnerError::AlreadyRunning)?;

        let dispatcher =
            MessageDispatcher::new(self.server.clone(), topology_query, mcp_rx, gossip_rx);

        // 在后台运行消息分发
        tokio::spawn(async move {
            let mut dispatcher = dispatcher;
            dispatcher.run().await;
        });

        tracing::info!("Server {} started", self.config.server_id);
        Ok(())
    }

    /// 停止 Server
    pub async fn stop(&self) -> Result<(), RunnerError> {
        // 更新状态
        {
            let base = self.server_base.read().await;
            base.set_state(ServerState::Removed).await;
        }

        // 发布停止事件
        self.event_bus
            .publish(crate::runtime::ServerEvent::ServerStopped {
                server_id: self.config.server_id.clone(),
            });

        tracing::info!("Server {} stopped", self.config.server_id);
        Ok(())
    }

    /// 获取拓扑查询器
    pub fn topology_query(&self) -> TopologyQuery {
        TopologyQuery::new(self.topology.clone())
    }

    /// 获取 Gossip 处理器
    pub fn gossip_handler(&self) -> GossipHandler {
        GossipHandler::new(self.server_base.clone())
    }
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

    #[async_trait]
    impl MCPServer for TestServer {
        fn id(&self) -> ServerId {
            self.id.clone()
        }

        fn tools(&self) -> Vec<Tool> {
            vec![Tool::new("test", "Test tool")]
        }

        async fn handle_tool_call(&self, _call: ToolCall) -> ToolResult {
            ToolResult::text("ok")
        }

        async fn on_message(&self, _msg: MCPMessage) -> Option<MCPMessage> {
            None
        }
    }

    #[tokio::test]
    async fn test_server_runner_start_stop() {
        let config = ServerConfig::default();
        let server_id = config.server_id.clone();

        let server = TestServer { id: server_id };
        let mut runner = ServerRunner::new(server, config);

        runner.start().await.unwrap();

        // 验证状态
        let base = runner.server_base.read().await;
        assert_eq!(base.state().await, ServerState::Active);

        runner.stop().await.unwrap();

        let base = runner.server_base.read().await;
        assert_eq!(base.state().await, ServerState::Removed);
    }
}
