use crate::gossip::GossipMessage;
use crate::protocol::MCPMessage;
use crate::server::MCPServer;
use crate::topology::TopologyQuery;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// 消息分发器
pub struct MessageDispatcher<S: MCPServer> {
    server: Arc<RwLock<S>>,
    #[allow(dead_code)]
    topology_query: TopologyQuery,
    mcp_rx: mpsc::Receiver<MCPMessage>,
    gossip_rx: mpsc::Receiver<GossipMessage>,
}

impl<S: MCPServer + 'static> MessageDispatcher<S> {
    /// 创建新的消息分发器
    pub fn new(
        server: Arc<RwLock<S>>,
        topology_query: TopologyQuery,
        mcp_rx: mpsc::Receiver<MCPMessage>,
        gossip_rx: mpsc::Receiver<GossipMessage>,
    ) -> Self {
        Self {
            server,
            topology_query,
            mcp_rx,
            gossip_rx,
        }
    }

    /// 运行消息分发循环
    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                Some(msg) = self.mcp_rx.recv() => {
                    self.dispatch_mcp(msg).await;
                }
                Some(msg) = self.gossip_rx.recv() => {
                    self.dispatch_gossip(msg).await;
                }
                else => break,
            }
        }
    }

    /// 分发 MCP 消息
    async fn dispatch_mcp(&self, msg: MCPMessage) {
        tracing::debug!("Dispatching MCP message: {:?}", msg.message_type);

        let server = self.server.read().await;

        // 调用 Server 的消息处理器
        if let Some(_response) = server.on_message(msg).await {
            // 发送响应 (如果需要)
            tracing::debug!("Sending response message");
            // 在实际实现中,这里应该通过消息路由发送响应
        }
    }

    /// 分发 Gossip 消息
    async fn dispatch_gossip(&self, msg: GossipMessage) {
        tracing::debug!("Dispatching Gossip message from {:?}", msg.source());
        // Gossip 消息由 GossipHandler 处理
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{MCPMessage, Tool, ToolCall, ToolResult};
    use crate::topology::LocalTopologyState;
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
            vec![]
        }

        async fn handle_tool_call(&self, _call: ToolCall) -> ToolResult {
            ToolResult::text("ok")
        }

        async fn on_message(&self, _msg: MCPMessage) -> Option<MCPMessage> {
            None
        }
    }

    #[tokio::test]
    async fn test_dispatcher_creation() {
        let server_id = "server-1".to_string();
        let server = Arc::new(RwLock::new(TestServer { id: server_id }));
        let topology = Arc::new(RwLock::new(LocalTopologyState::new()));
        let query = TopologyQuery::new(topology);

        let (mcp_tx, mcp_rx) = mpsc::channel(10);
        let (gossip_tx, gossip_rx) = mpsc::channel(10);

        let _dispatcher = MessageDispatcher::new(server, query, mcp_rx, gossip_rx);

        // 测试发送消息
        let msg = MCPMessage::request("test", None);
        mcp_tx.send(msg).await.unwrap();

        let gossip_msg = GossipMessage::heartbeat("server-2".to_string());
        gossip_tx.send(gossip_msg).await.unwrap();
    }
}
