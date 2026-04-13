use crate::gossip::GossipMessage;
use session_manager::ServerId;
use std::collections::HashMap;

/// 拓扑同步器
pub struct TopologySync {
    /// 本地 Server ID
    server_id: ServerId,
    /// 拓扑版本号
    version: u64,
}

impl TopologySync {
    /// 创建新的拓扑同步器
    pub fn new(server_id: ServerId) -> Self {
        Self {
            server_id,
            version: 0,
        }
    }

    /// 创建拓扑同步消息
    pub fn create_sync_message(
        &mut self,
        known_peers: Vec<ServerId>,
        routing_table: HashMap<String, ServerId>,
    ) -> GossipMessage {
        self.version += 1;
        GossipMessage::topology_sync(
            self.server_id.clone(),
            known_peers,
            routing_table,
            self.version,
        )
    }

    /// 获取当前版本
    pub fn version(&self) -> u64 {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sync_message() {
        let server_id = "server-1".to_string();
        let mut sync = TopologySync::new(server_id.clone());

        let peer = "server-2".to_string();
        let mut routing = HashMap::new();
        routing.insert("tool1".to_string(), peer.clone());

        let msg = sync.create_sync_message(vec![peer], routing);

        if let GossipMessage::TopologySync { version, .. } = msg {
            assert_eq!(version, 1);
        } else {
            panic!("Expected TopologySync message");
        }

        // 版本应该递增
        assert_eq!(sync.version(), 1);
    }
}
