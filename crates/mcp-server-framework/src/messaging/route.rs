use crate::topology::TopologyQuery;
use session_manager::ServerId;

/// 路由决策
#[derive(Debug, Clone)]
pub enum RoutingDecision {
    /// 本地处理
    Local,
    /// 转发到指定 Server
    Forward(ServerId),
    /// 广播查询
    BroadcastQuery,
    /// 无路由
    NoRoute,
}

/// 根据 capability 做出路由决策
///
/// # 参数
/// - `query`: 拓扑查询器
/// - `capability`: 能力名称
///
/// # 返回
/// 路由决策
pub async fn make_routing_decision(
    query: &TopologyQuery,
    capability: &str,
    local_server_id: &ServerId,
) -> RoutingDecision {
    // 查询本地路由表
    if let Some(server_id) = query.find_server_by_capability(capability).await {
        if &server_id == local_server_id {
            RoutingDecision::Local
        } else {
            RoutingDecision::Forward(server_id)
        }
    } else {
        // 本地无路由,广播查询
        RoutingDecision::BroadcastQuery
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::LocalTopologyState;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_local_routing_decision() {
        let local_id = "server-1".to_string();
        let state = Arc::new(RwLock::new(LocalTopologyState::new()));
        let query = TopologyQuery::new(state.clone());

        // 本地提供该能力
        {
            let mut state = state.write().await;
            state
                .routing_table
                .insert("tool1".to_string(), local_id.clone());
        }

        let decision = make_routing_decision(&query, "tool1", &local_id).await;
        assert!(matches!(decision, RoutingDecision::Local));
    }

    #[tokio::test]
    async fn test_forward_routing_decision() {
        let local_id = "server-1".to_string();
        let remote_id = "server-2".to_string();
        let state = Arc::new(RwLock::new(LocalTopologyState::new()));
        let query = TopologyQuery::new(state.clone());

        {
            let mut state = state.write().await;
            state
                .routing_table
                .insert("tool1".to_string(), remote_id.clone());
        }

        let decision = make_routing_decision(&query, "tool1", &local_id).await;
        assert!(matches!(decision, RoutingDecision::Forward(id) if id == remote_id));
    }

    #[tokio::test]
    async fn test_no_route_decision() {
        let local_id = "server-1".to_string();
        let state = Arc::new(RwLock::new(LocalTopologyState::new()));
        let query = TopologyQuery::new(state);

        let decision = make_routing_decision(&query, "unknown", &local_id).await;
        assert!(matches!(decision, RoutingDecision::BroadcastQuery));
    }
}
