use crate::gossip::GossipMessage;
use crate::server::ServerBase;
use session_manager::ServerId;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Gossip 消息处理器
pub struct GossipHandler {
    server_base: Arc<RwLock<ServerBase>>,
}

impl GossipHandler {
    /// 创建新的 Gossip 处理器
    pub fn new(server_base: Arc<RwLock<ServerBase>>) -> Self {
        Self { server_base }
    }

    /// 处理 Gossip 消息
    pub async fn handle(&self, msg: GossipMessage) -> Option<GossipMessage> {
        match msg {
            GossipMessage::Heartbeat { server_id, .. } => {
                self.on_heartbeat(&server_id).await;
                None
            }
            GossipMessage::Join { server_id, tools } => self.on_join(&server_id, &tools).await,
            GossipMessage::Leave { server_id } => {
                self.on_leave(&server_id).await;
                None
            }
            GossipMessage::ToolAnnounce { server_id, tools } => {
                self.on_tool_announce(&server_id, &tools).await;
                None
            }
            GossipMessage::TopologySync {
                server_id,
                known_peers,
                routing_table,
                version,
            } => {
                self.on_topology_sync(&server_id, known_peers, routing_table, version)
                    .await;
                None
            }
            GossipMessage::Suspect {
                server_id,
                reporter: _,
            } => {
                self.on_suspect(&server_id).await;
                None
            }
            GossipMessage::Welcome {
                server_id: _,
                known_peers,
                routing_table,
            } => {
                self.on_welcome(known_peers, routing_table).await;
                None
            }
        }
    }

    /// 处理心跳消息
    async fn on_heartbeat(&self, server_id: &ServerId) {
        let base = self.server_base.write().await;
        base.add_peer(server_id.clone()).await;

        tracing::debug!("Received heartbeat from {}", server_id);
    }

    /// 处理加入消息
    async fn on_join(&self, server_id: &ServerId, tools: &[String]) -> Option<GossipMessage> {
        let base = self.server_base.write().await;

        // 添加到 known_peers
        base.add_peer(server_id.clone()).await;

        // 更新路由表
        for tool in tools {
            base.update_routing(tool.clone(), server_id.clone()).await;
        }

        tracing::info!("Server {} joined with {} tools", server_id, tools.len());

        // 发送 Welcome 消息
        let known_peers: Vec<_> = base.known_peers().await.into_iter().collect();
        let routing_table = base.get_routing_table().await;

        Some(GossipMessage::welcome(
            base.id().clone(),
            known_peers,
            routing_table,
        ))
    }

    /// 处理离开消息
    async fn on_leave(&self, server_id: &ServerId) {
        let base = self.server_base.write().await;
        base.remove_peer(server_id).await;

        tracing::info!("Server {} left", server_id);
    }

    /// 处理工具公告消息
    async fn on_tool_announce(&self, server_id: &ServerId, tools: &[String]) {
        let base = self.server_base.write().await;

        for tool in tools {
            base.update_routing(tool.clone(), server_id.clone()).await;
        }

        tracing::info!("Server {} announced {} tools", server_id, tools.len());
    }

    /// 处理拓扑同步消息
    async fn on_topology_sync(
        &self,
        _server_id: &ServerId,
        known_peers: Vec<ServerId>,
        routing_table: std::collections::HashMap<String, ServerId>,
        _version: u64,
    ) {
        let base = self.server_base.write().await;

        // 合并 known_peers
        for peer in known_peers {
            base.add_peer(peer).await;
        }

        // 合并路由表
        for (capability, server_id) in routing_table {
            base.update_routing(capability, server_id).await;
        }

        tracing::debug!("Topology synchronized");
    }

    /// 处理可疑消息
    async fn on_suspect(&self, server_id: &ServerId) {
        tracing::warn!("Server {} suspected to be failed", server_id);

        // 在实际实现中,应该启动失效检测流程
    }

    /// 处理欢迎消息
    async fn on_welcome(
        &self,
        known_peers: Vec<ServerId>,
        routing_table: std::collections::HashMap<String, ServerId>,
    ) {
        let base = self.server_base.write().await;

        // 合并 known_peers
        for peer in known_peers {
            base.add_peer(peer).await;
        }

        // 合并路由表
        for (capability, server_id) in routing_table {
            base.update_routing(capability, server_id).await;
        }

        tracing::info!("Received welcome message, topology updated");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use session_manager::SessionId;

    #[tokio::test]
    async fn test_handle_heartbeat() {
        let server_id = "server-1".to_string();
        let session_id = SessionId::new_v4();
        let base = Arc::new(RwLock::new(ServerBase::new(server_id, session_id)));
        let handler = GossipHandler::new(base.clone());

        let peer_id = "server-2".to_string();
        let msg = GossipMessage::heartbeat(peer_id.clone());
        handler.handle(msg).await;

        let base = base.read().await;
        let peers = base.known_peers().await;
        assert!(peers.contains(&peer_id));
    }
}
