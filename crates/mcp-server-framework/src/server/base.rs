use crate::protocol::Tool;
use session_manager::{ServerId, SessionId};
use std::collections::HashSet;

/// Server 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// 待启动
    Pending,
    /// 运行中
    Active,
    /// 正在排空 (优雅关闭)
    Draining,
    /// 已移除
    Removed,
}

/// Server 元数据
#[derive(Debug, Clone)]
pub struct ServerMeta {
    /// 已知的 Peer Server
    pub known_peers: HashSet<ServerId>,
    /// 本地工具列表
    pub local_tools: Vec<Tool>,
    /// 路由表缓存 (capability -> server_id)
    pub routing_table: std::collections::HashMap<String, ServerId>,
    /// Session 引用
    pub session_id: SessionId,
}

/// Server 基础结构
///
/// 提供所有 Server 的公共功能
pub struct ServerBase {
    /// Server ID
    id: ServerId,
    /// Server 状态
    state: std::sync::Arc<tokio::sync::RwLock<ServerState>>,
    /// Server 元数据
    meta: std::sync::Arc<std::sync::RwLock<ServerMeta>>,
}

impl ServerBase {
    /// 创建新的 Server 基础结构
    pub fn new(id: ServerId, session_id: SessionId) -> Self {
        Self {
            id,
            state: std::sync::Arc::new(tokio::sync::RwLock::new(ServerState::Pending)),
            meta: std::sync::Arc::new(std::sync::RwLock::new(ServerMeta {
                known_peers: HashSet::new(),
                local_tools: Vec::new(),
                routing_table: std::collections::HashMap::new(),
                session_id,
            })),
        }
    }

    /// 获取 Server ID
    pub fn id(&self) -> &ServerId {
        &self.id
    }

    /// 获取 Session ID
    pub fn session_id(&self) -> SessionId {
        self.meta.read().unwrap().session_id
    }

    /// 获取当前状态
    pub async fn state(&self) -> ServerState {
        *self.state.read().await
    }

    /// 设置状态
    pub async fn set_state(&self, new_state: ServerState) {
        *self.state.write().await = new_state;
    }

    /// 获取已知 Peers
    pub async fn known_peers(&self) -> HashSet<ServerId> {
        self.meta.read().unwrap().known_peers.clone()
    }

    /// 添加 Peer
    pub async fn add_peer(&self, peer_id: ServerId) {
        self.meta.write().unwrap().known_peers.insert(peer_id);
    }

    /// 移除 Peer
    pub async fn remove_peer(&self, peer_id: &ServerId) {
        self.meta.write().unwrap().known_peers.remove(peer_id);
    }

    /// 获取工具列表
    pub async fn tools(&self) -> Vec<Tool> {
        self.meta.read().unwrap().local_tools.clone()
    }

    /// 设置工具列表
    pub async fn set_tools(&self, tools: Vec<Tool>) {
        self.meta.write().unwrap().local_tools = tools;
    }

    /// 更新路由表
    pub async fn update_routing(&self, capability: String, server_id: ServerId) {
        self.meta
            .write()
            .unwrap()
            .routing_table
            .insert(capability, server_id);
    }

    /// 查询路由表
    pub async fn find_server_by_capability(&self, capability: &str) -> Option<ServerId> {
        self.meta
            .read()
            .unwrap()
            .routing_table
            .get(capability)
            .cloned()
    }

    /// 获取路由表
    pub async fn get_routing_table(&self) -> std::collections::HashMap<String, ServerId> {
        self.meta.read().unwrap().routing_table.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_base_state() {
        let server_id = "server-1".to_string();
        let session_id = SessionId::new_v4();
        let base = ServerBase::new(server_id, session_id);

        assert_eq!(base.state().await, ServerState::Pending);

        base.set_state(ServerState::Active).await;
        assert_eq!(base.state().await, ServerState::Active);
    }

    #[tokio::test]
    async fn test_server_base_peers() {
        let server_id = "server-1".to_string();
        let session_id = SessionId::new_v4();
        let base = ServerBase::new(server_id, session_id);

        let peer1 = "server-2".to_string();
        let peer2 = "server-3".to_string();

        base.add_peer(peer1.clone()).await;
        base.add_peer(peer2.clone()).await;

        let peers = base.known_peers().await;
        assert_eq!(peers.len(), 2);
        assert!(peers.contains(&peer1));
        assert!(peers.contains(&peer2));

        base.remove_peer(&peer1).await;
        let peers = base.known_peers().await;
        assert_eq!(peers.len(), 1);
        assert!(!peers.contains(&peer1));
    }

    #[tokio::test]
    async fn test_server_base_routing() {
        let server_id = "server-1".to_string();
        let session_id = SessionId::new_v4();
        let base = ServerBase::new(server_id, session_id);

        let target = "server-2".to_string();
        base.update_routing("code_review".to_string(), target.clone())
            .await;

        let found = base.find_server_by_capability("code_review").await;
        assert_eq!(found, Some(target));

        let not_found = base.find_server_by_capability("unknown").await;
        assert_eq!(not_found, None);
    }
}
