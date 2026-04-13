use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use session_manager::ServerId;
use std::collections::HashMap;

/// Gossip 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GossipMessage {
    /// 心跳消息
    Heartbeat {
        server_id: ServerId,
        timestamp: DateTime<Utc>,
    },

    /// 加入网络
    Join {
        server_id: ServerId,
        tools: Vec<String>,
    },

    /// 离开网络
    Leave { server_id: ServerId },

    /// 工具公告
    ToolAnnounce {
        server_id: ServerId,
        tools: Vec<String>,
    },

    /// 拓扑同步
    TopologySync {
        server_id: ServerId,
        known_peers: Vec<ServerId>,
        routing_table: HashMap<String, ServerId>,
        version: u64,
    },

    /// 可疑节点
    Suspect {
        server_id: ServerId,
        reporter: ServerId,
    },

    /// 欢迎消息 (响应 Join)
    Welcome {
        server_id: ServerId,
        known_peers: Vec<ServerId>,
        routing_table: HashMap<String, ServerId>,
    },
}

impl GossipMessage {
    /// 创建心跳消息
    pub fn heartbeat(server_id: ServerId) -> Self {
        Self::Heartbeat {
            server_id,
            timestamp: Utc::now(),
        }
    }

    /// 创建加入消息
    pub fn join(server_id: ServerId, tools: Vec<String>) -> Self {
        Self::Join { server_id, tools }
    }

    /// 创建离开消息
    pub fn leave(server_id: ServerId) -> Self {
        Self::Leave { server_id }
    }

    /// 创建工具公告消息
    pub fn tool_announce(server_id: ServerId, tools: Vec<String>) -> Self {
        Self::ToolAnnounce { server_id, tools }
    }

    /// 创建拓扑同步消息
    pub fn topology_sync(
        server_id: ServerId,
        known_peers: Vec<ServerId>,
        routing_table: HashMap<String, ServerId>,
        version: u64,
    ) -> Self {
        Self::TopologySync {
            server_id,
            known_peers,
            routing_table,
            version,
        }
    }

    /// 创建可疑消息
    pub fn suspect(server_id: ServerId, reporter: ServerId) -> Self {
        Self::Suspect {
            server_id,
            reporter,
        }
    }

    /// 创建欢迎消息
    pub fn welcome(
        server_id: ServerId,
        known_peers: Vec<ServerId>,
        routing_table: HashMap<String, ServerId>,
    ) -> Self {
        Self::Welcome {
            server_id,
            known_peers,
            routing_table,
        }
    }

    /// 获取消息来源 Server ID
    pub fn source(&self) -> Option<&ServerId> {
        match self {
            Self::Heartbeat { server_id, .. } => Some(server_id),
            Self::Join { server_id, .. } => Some(server_id),
            Self::Leave { server_id } => Some(server_id),
            Self::ToolAnnounce { server_id, .. } => Some(server_id),
            Self::TopologySync { server_id, .. } => Some(server_id),
            Self::Suspect { server_id, .. } => Some(server_id),
            Self::Welcome { server_id, .. } => Some(server_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_message() {
        let server_id = "server-1".to_string();
        let msg = GossipMessage::heartbeat(server_id.clone());

        assert_eq!(msg.source(), Some(&server_id));

        // 序列化测试
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"heartbeat\""));
    }

    #[test]
    fn test_join_message() {
        let server_id = "server-1".to_string();
        let tools = vec!["tool1".to_string(), "tool2".to_string()];
        let msg = GossipMessage::join(server_id.clone(), tools.clone());

        if let GossipMessage::Join {
            tools: msg_tools, ..
        } = &msg
        {
            assert_eq!(*msg_tools, tools);
        } else {
            panic!("Expected Join message");
        }
    }

    #[test]
    fn test_topology_sync_message() {
        let server_id = "server-1".to_string();
        let peer1 = "server-2".to_string();
        let peer2 = "server-3".to_string();
        let mut routing = HashMap::new();
        routing.insert("code_review".to_string(), peer1.clone());

        let msg = GossipMessage::topology_sync(server_id.clone(), vec![peer1, peer2], routing, 1);

        if let GossipMessage::TopologySync { version, .. } = &msg {
            assert_eq!(*version, 1);
        } else {
            panic!("Expected TopologySync message");
        }
    }
}
