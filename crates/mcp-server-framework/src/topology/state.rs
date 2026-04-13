use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use session_manager::ServerId;
use std::collections::{HashMap, HashSet};

/// Server 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server ID
    pub id: ServerId,
    /// 提供的工具列表
    pub tools: Vec<String>,
    /// 最后更新时间
    pub last_seen: DateTime<Utc>,
}

/// 本地拓扑状态
#[derive(Debug, Clone)]
pub struct LocalTopologyState {
    /// 已知的 Peer Servers
    pub known_peers: HashSet<ServerId>,
    /// 路由表 (capability -> server_id)
    pub routing_table: HashMap<String, ServerId>,
    /// Server 详细信息
    pub server_info: HashMap<ServerId, ServerInfo>,
    /// 拓扑版本号
    pub version: u64,
}

impl LocalTopologyState {
    /// 创建新的拓扑状态
    pub fn new() -> Self {
        Self {
            known_peers: HashSet::new(),
            routing_table: HashMap::new(),
            server_info: HashMap::new(),
            version: 0,
        }
    }

    /// 添加 Peer
    pub fn add_peer(&mut self, server_id: ServerId, tools: Vec<String>) {
        self.known_peers.insert(server_id.clone());

        self.server_info.insert(
            server_id.clone(),
            ServerInfo {
                id: server_id.clone(),
                tools: tools.clone(),
                last_seen: Utc::now(),
            },
        );

        // 更新路由表
        for tool in tools {
            self.routing_table.insert(tool, server_id.clone());
        }

        self.version += 1;
    }

    /// 移除 Peer
    pub fn remove_peer(&mut self, server_id: &ServerId) {
        self.known_peers.remove(server_id);
        self.server_info.remove(server_id);

        // 清理路由表
        self.routing_table.retain(|_, sid| sid != server_id);

        self.version += 1;
    }

    /// 更新 Peer 工具列表
    pub fn update_peer_tools(&mut self, server_id: &ServerId, tools: Vec<String>) {
        if let Some(info) = self.server_info.get_mut(server_id) {
            info.tools = tools.clone();
            info.last_seen = Utc::now();

            // 更新路由表
            for tool in tools {
                self.routing_table.insert(tool, server_id.clone());
            }

            self.version += 1;
        }
    }

    /// 更新最后见到时间
    pub fn touch_peer(&mut self, server_id: &ServerId) {
        if let Some(info) = self.server_info.get_mut(server_id) {
            info.last_seen = Utc::now();
        }
    }

    /// 合并其他节点的拓扑信息
    pub fn merge(&mut self, other: LocalTopologyState) {
        // 合并 known_peers
        for server_id in other.known_peers {
            self.known_peers.insert(server_id);
        }

        // 合并 server_info
        for (server_id, info) in other.server_info {
            // 只在信息更新时合并
            if let Some(existing) = self.server_info.get(&server_id) {
                if info.last_seen > existing.last_seen {
                    self.server_info.insert(server_id, info);
                }
            } else {
                self.server_info.insert(server_id, info);
            }
        }

        // 合并 routing_table
        for (capability, server_id) in other.routing_table {
            self.routing_table.insert(capability, server_id);
        }

        // 取较大的版本号
        self.version = self.version.max(other.version);
    }
}

impl Default for LocalTopologyState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_peer() {
        let mut state = LocalTopologyState::new();
        let server_id = "server-1".to_string();
        let tools = vec!["tool1".to_string(), "tool2".to_string()];

        state.add_peer(server_id.clone(), tools.clone());

        assert!(state.known_peers.contains(&server_id));
        assert_eq!(state.routing_table.len(), 2);
        assert!(state.server_info.contains_key(&server_id));
        assert_eq!(state.version, 1);
    }

    #[test]
    fn test_remove_peer() {
        let mut state = LocalTopologyState::new();
        let server_id = "server-1".to_string();

        state.add_peer(server_id.clone(), vec!["tool1".to_string()]);
        state.remove_peer(&server_id);

        assert!(!state.known_peers.contains(&server_id));
        assert!(!state.server_info.contains_key(&server_id));
        assert_eq!(state.routing_table.len(), 0);
        assert_eq!(state.version, 2);
    }

    #[test]
    fn test_merge() {
        let mut state1 = LocalTopologyState::new();
        let mut state2 = LocalTopologyState::new();

        let server1 = "server-1".to_string();
        let server2 = "server-2".to_string();

        state1.add_peer(server1.clone(), vec!["tool1".to_string()]);
        state2.add_peer(server2.clone(), vec!["tool2".to_string()]);

        state1.merge(state2);

        assert_eq!(state1.known_peers.len(), 2);
        assert_eq!(state1.routing_table.len(), 2);
    }
}
