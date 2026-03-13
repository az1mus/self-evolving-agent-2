//! Gateway 配置定义

use serde::{Deserialize, Serialize};

/// Gateway 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// 最大 Session 数量
    pub max_sessions: usize,
    /// 每个 Session 最大 Agent 数量
    pub max_agents_per_session: usize,
    /// 最大 Agent 定义数量
    pub max_agent_definitions: usize,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            max_sessions: 10,
            max_agents_per_session: 5,
            max_agent_definitions: 50,
        }
    }
}