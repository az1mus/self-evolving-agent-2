use crate::protocol::{MCPMessage, Tool, ToolCall, ToolResult};
use async_trait::async_trait;
use session_manager::ServerId;

/// MCP Server 核心接口
///
/// 所有 MCP Server 必须实现此 trait
#[async_trait]
pub trait MCPServer: Send + Sync {
    /// 获取 Server ID
    fn id(&self) -> ServerId;

    /// 获取 Server 提供的工具列表
    fn tools(&self) -> Vec<Tool>;

    /// 处理工具调用
    async fn handle_tool_call(&self, call: ToolCall) -> ToolResult;

    /// 处理 MCP 消息 (可选)
    ///
    /// 返回 None 表示消息不需要响应
    async fn on_message(&self, msg: MCPMessage) -> Option<MCPMessage>;
}
