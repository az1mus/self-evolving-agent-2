//! Gateway 事件定义
//!
//! 定义 Gateway 中各种操作触发的事件，用于事件监听和日志记录

use crate::agent::{AgentDefinitionId, AgentInstanceId, AgentOutput};
use crate::session::SessionId;

/// Gateway 事件
#[derive(Debug, Clone)]
pub enum GatewayEvent {
    /// Agent 定义创建
    AgentDefinitionCreated { id: AgentDefinitionId, name: String },
    /// Agent 定义删除
    AgentDefinitionDeleted { id: AgentDefinitionId },
    /// Session 创建
    SessionCreated { id: SessionId, name: String },
    /// Session 切换
    SessionSwitched { id: SessionId },
    /// Session 删除
    SessionDeleted { id: SessionId },
    /// Agent 实例创建
    AgentInstanceCreated { id: AgentInstanceId, definition_id: AgentDefinitionId, session_id: SessionId },
    /// Agent 实例切换
    AgentInstanceSwitched { id: AgentInstanceId },
    /// Agent 实例删除
    AgentInstanceDeleted { id: AgentInstanceId },
    /// 消息发送
    MessageSent { agent_instance_id: AgentInstanceId, content: String },
    /// 消息接收
    MessageReceived { agent_instance_id: AgentInstanceId, output: AgentOutput },
    /// 错误
    Error { message: String },
}

/// 事件监听器
pub type EventListener = Box<dyn Fn(&GatewayEvent) + Send + Sync>;