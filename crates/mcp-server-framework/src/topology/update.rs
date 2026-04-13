use crate::gossip::GossipMessage;
use crate::topology::LocalTopologyState;
use session_manager::ServerId;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 拓扑更新器
pub struct TopologyUpdate {
    state: Arc<RwLock<LocalTopologyState>>,
    gossip_tx: tokio::sync::mpsc::Sender<GossipMessage>,
    local_server_id: ServerId,
}

impl TopologyUpdate {
    /// 创建新的拓扑更新器
    pub fn new(
        state: Arc<RwLock<LocalTopologyState>>,
        gossip_tx: tokio::sync::mpsc::Sender<GossipMessage>,
        local_server_id: ServerId,
    ) -> Self {
        Self {
            state,
            gossip_tx,
            local_server_id,
        }
    }

    /// 添加 Peer 并广播
    pub async fn add_peer(&self, server_id: ServerId, tools: Vec<String>) {
        {
            let mut state = self.state.write().await;
            state.add_peer(server_id.clone(), tools.clone());
        }

        // 广播 ToolAnnounce
        let msg = GossipMessage::tool_announce(server_id, tools);
        if let Err(e) = self.gossip_tx.send(msg).await {
            tracing::error!("Failed to broadcast tool announcement: {}", e);
        }
    }

    /// 移除 Peer 并广播
    pub async fn remove_peer(&self, server_id: &ServerId) {
        {
            let mut state = self.state.write().await;
            state.remove_peer(server_id);
        }

        // 广播 Leave
        let msg = GossipMessage::leave(server_id.clone());
        if let Err(e) = self.gossip_tx.send(msg).await {
            tracing::error!("Failed to broadcast leave: {}", e);
        }
    }

    /// 更新路由表
    pub async fn update_routing(&self, capability: String, server_id: ServerId) {
        let mut state = self.state.write().await;
        state.routing_table.insert(capability, server_id);
        state.version += 1;
    }

    /// 广播拓扑同步
    pub async fn broadcast_topology_sync(&self) {
        let (known_peers, routing_table, version) = {
            let state = self.state.read().await;
            (
                state.known_peers.iter().cloned().collect(),
                state.routing_table.clone(),
                state.version,
            )
        };

        let msg = GossipMessage::topology_sync(
            self.local_server_id.clone(),
            known_peers,
            routing_table,
            version,
        );

        if let Err(e) = self.gossip_tx.send(msg).await {
            tracing::error!("Failed to broadcast topology sync: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_peer() {
        let state = Arc::new(RwLock::new(LocalTopologyState::new()));
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let local_id = "server-1".to_string();

        let updater = TopologyUpdate::new(state.clone(), tx, local_id);
        let peer_id = "server-2".to_string();
        let tools = vec!["tool1".to_string()];

        updater.add_peer(peer_id.clone(), tools.clone()).await;

        // 验证状态更新
        let state = state.read().await;
        assert!(state.known_peers.contains(&peer_id));

        // 验证消息广播
        let msg = rx.recv().await.unwrap();
        if let GossipMessage::ToolAnnounce {
            tools: msg_tools, ..
        } = msg
        {
            assert_eq!(msg_tools, tools);
        } else {
            panic!("Expected ToolAnnounce message");
        }
    }
}
