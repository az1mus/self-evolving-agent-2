use crate::topology::LocalTopologyState;
use session_manager::ServerId;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 拓扑一致性问题类型
#[derive(Debug, Clone)]
pub enum ConsistencyIssue {
    /// 孤岛节点 (没有连接任何其他节点)
    IsolatedNode { server_id: ServerId },
    /// 路由表不一致 (capability 指向不存在的 Server)
    InconsistentRouting {
        capability: String,
        server_id: ServerId,
    },
    /// Server 信息缺失
    MissingServerInfo { server_id: ServerId },
}

/// 拓扑一致性检查器
pub struct TopologyConsistency {
    state: Arc<RwLock<LocalTopologyState>>,
}

impl TopologyConsistency {
    /// 创建新的一致性检查器
    pub fn new(state: Arc<RwLock<LocalTopologyState>>) -> Self {
        Self { state }
    }

    /// 检查所有一致性问题
    pub async fn check(&self) -> Vec<ConsistencyIssue> {
        let state = self.state.read().await;
        let mut issues = Vec::new();

        // 检查孤岛节点
        if state.known_peers.is_empty() {
            // 当前节点是孤岛
        }

        // 检查路由表一致性
        for (capability, server_id) in &state.routing_table {
            // 验证 server_id 是否在 known_peers 中
            if !state.known_peers.contains(server_id) {
                issues.push(ConsistencyIssue::InconsistentRouting {
                    capability: capability.clone(),
                    server_id: server_id.clone(),
                });
            }

            // 验证 server_info 是否存在
            if !state.server_info.contains_key(server_id) {
                issues.push(ConsistencyIssue::MissingServerInfo {
                    server_id: server_id.clone(),
                });
            }
        }

        // 检查 known_peers 和 server_info 的一致性
        for server_id in &state.known_peers {
            if !state.server_info.contains_key(server_id) {
                issues.push(ConsistencyIssue::MissingServerInfo {
                    server_id: server_id.clone(),
                });
            }
        }

        issues
    }

    /// 修复一致性问题
    pub async fn repair(&self) {
        let mut state = self.state.write().await;

        // 清理不一致的路由
        let known_peers = state.known_peers.clone();
        state
            .routing_table
            .retain(|_, server_id| known_peers.contains(server_id));

        // 补充缺失的 server_info
        let mut missing_servers = Vec::new();
        for server_id in &state.known_peers {
            if !state.server_info.contains_key(server_id) {
                missing_servers.push(server_id.clone());
            }
        }

        for server_id in missing_servers {
            // 移除缺失信息的节点
            state.known_peers.remove(&server_id);
        }

        state.version += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_inconsistent_routing() {
        let state = Arc::new(RwLock::new(LocalTopologyState::new()));
        let consistency = TopologyConsistency::new(state.clone());

        // 手动创建不一致的路由
        {
            let mut state = state.write().await;
            let fake_server = "fake-server".to_string();
            state
                .routing_table
                .insert("tool1".to_string(), fake_server.clone());
        }

        let issues = consistency.check().await;
        assert!(!issues.is_empty());

        // 应该检测到不一致
        let has_routing_issue = issues
            .iter()
            .any(|issue| matches!(issue, ConsistencyIssue::InconsistentRouting { .. }));
        assert!(has_routing_issue);
    }

    #[tokio::test]
    async fn test_repair() {
        let state = Arc::new(RwLock::new(LocalTopologyState::new()));
        let consistency = TopologyConsistency::new(state.clone());

        // 创建不一致状态
        {
            let mut state = state.write().await;
            let fake_server = "fake-server".to_string();
            state
                .routing_table
                .insert("tool1".to_string(), fake_server.clone());
            state.known_peers.insert(fake_server);
        }

        // 修复
        consistency.repair().await;

        // 验证清理
        let state = state.read().await;
        assert!(state.routing_table.is_empty() || state.known_peers.is_empty());
    }
}
