use crate::topology::{LocalTopologyState, ServerInfo};
use session_manager::ServerId;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 拓扑查询器
pub struct TopologyQuery {
    state: Arc<RwLock<LocalTopologyState>>,
}

impl TopologyQuery {
    /// 创建新的拓扑查询器
    pub fn new(state: Arc<RwLock<LocalTopologyState>>) -> Self {
        Self { state }
    }

    /// 根据 capability 查找 Server
    pub async fn find_server_by_capability(&self, capability: &str) -> Option<ServerId> {
        let state = self.state.read().await;
        state.routing_table.get(capability).cloned()
    }

    /// 列出所有 Peers
    pub async fn list_peers(&self) -> Vec<ServerId> {
        let state = self.state.read().await;
        state.known_peers.iter().cloned().collect()
    }

    /// 获取 Server 信息
    pub async fn get_server_info(&self, server_id: &ServerId) -> Option<ServerInfo> {
        let state = self.state.read().await;
        state.server_info.get(server_id).cloned()
    }

    /// 列出所有工具
    pub async fn list_all_tools(&self) -> Vec<String> {
        let state = self.state.read().await;
        state.routing_table.keys().cloned().collect()
    }

    /// 获取拓扑版本
    pub async fn get_version(&self) -> u64 {
        let state = self.state.read().await;
        state.version
    }

    /// 统计信息
    pub async fn stats(&self) -> TopologyStats {
        let state = self.state.read().await;
        TopologyStats {
            peer_count: state.known_peers.len(),
            tool_count: state.routing_table.len(),
            version: state.version,
        }
    }
}

/// 拓扑统计信息
#[derive(Debug, Clone)]
pub struct TopologyStats {
    pub peer_count: usize,
    pub tool_count: usize,
    pub version: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_server_by_capability() {
        let state = Arc::new(RwLock::new(LocalTopologyState::new()));
        let query = TopologyQuery::new(state.clone());

        let server_id = "server-1".to_string();
        {
            let mut state = state.write().await;
            state.add_peer(server_id.clone(), vec!["tool1".to_string()]);
        }

        let found = query.find_server_by_capability("tool1").await;
        assert_eq!(found, Some(server_id));

        let not_found = query.find_server_by_capability("unknown").await;
        assert_eq!(not_found, None);
    }

    #[tokio::test]
    async fn test_list_peers() {
        let state = Arc::new(RwLock::new(LocalTopologyState::new()));
        let query = TopologyQuery::new(state.clone());

        let server1 = "server-1".to_string();
        let server2 = "server-2".to_string();

        {
            let mut state = state.write().await;
            state.add_peer(server1.clone(), vec![]);
            state.add_peer(server2.clone(), vec![]);
        }

        let peers = query.list_peers().await;
        assert_eq!(peers.len(), 2);
        assert!(peers.contains(&server1));
        assert!(peers.contains(&server2));
    }

    #[tokio::test]
    async fn test_stats() {
        let state = Arc::new(RwLock::new(LocalTopologyState::new()));
        let query = TopologyQuery::new(state.clone());

        let server_id = "server-1".to_string();
        {
            let mut state = state.write().await;
            state.add_peer(server_id, vec!["tool1".to_string(), "tool2".to_string()]);
        }

        let stats = query.stats().await;
        assert_eq!(stats.peer_count, 1);
        assert_eq!(stats.tool_count, 2);
    }
}
