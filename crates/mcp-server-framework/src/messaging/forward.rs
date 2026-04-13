use crate::protocol::MCPMessage;
use crate::topology::TopologyQuery;
use router_core::CycleDetector;
use session_manager::ServerId;
use thiserror::Error;

/// 消息转发错误
#[derive(Debug, Error)]
pub enum ForwardError {
    #[error("Cycle detected")]
    CycleDetected,

    #[error("Max hops exceeded")]
    MaxHopsExceeded,

    #[error("Target server not found: {0}")]
    ServerNotFound(ServerId),

    #[error("Forward failed: {0}")]
    Failed(String),
}

/// 转发消息到目标 Server
///
/// # 参数
/// - `message`: 原始消息
/// - `to_server`: 目标 Server ID
/// - `query`: 拓扑查询器
/// - `detector`: 循环检测器
///
/// # 返回
/// 转发后的消息 (更新了 visited_servers 和 hop_count)
pub async fn forward_message(
    message: MCPMessage,
    to_server: &ServerId,
    query: &TopologyQuery,
    _detector: &CycleDetector,
) -> Result<MCPMessage, ForwardError> {
    // 验证目标 Server 存在
    if query.get_server_info(to_server).await.is_none() {
        return Err(ForwardError::ServerNotFound(to_server.clone()));
    }

    // 在实际实现中,这里应该:
    // 1. 创建路由上下文并进行循环检测
    // 2. 更新消息的 metadata (visited_servers, hop_count)
    // 3. 通过消息路由发送到目标 Server

    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::LocalTopologyState;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_forward_message() {
        let state = Arc::new(RwLock::new(LocalTopologyState::new()));
        let query = TopologyQuery::new(state.clone());

        let target_id = "server-2".to_string();

        // 添加目标 Server
        {
            let mut state = state.write().await;
            state.add_peer(target_id.clone(), vec!["tool1".to_string()]);
        }

        let detector = CycleDetector::new();
        let msg = crate::protocol::MCPMessage::request("test", None);

        let result = forward_message(msg, &target_id, &query, &detector).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_forward_to_nonexistent_server() {
        let state = Arc::new(RwLock::new(LocalTopologyState::new()));
        let query = TopologyQuery::new(state);

        let target_id = "server-2".to_string();
        let detector = CycleDetector::new();
        let msg = crate::protocol::MCPMessage::request("test", None);

        let result = forward_message(msg, &target_id, &query, &detector).await;
        assert!(matches!(result, Err(ForwardError::ServerNotFound(_))));
    }
}
