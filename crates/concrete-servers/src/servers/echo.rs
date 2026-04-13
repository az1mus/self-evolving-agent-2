//! Echo Server
//!
//! 最基础的 Server 实现，返回输入内容，用于测试消息传递

use async_trait::async_trait;
use mcp_server_framework::{MCPServer, Tool, ToolCall, ToolResult};
use serde_json::json;

/// Echo Server
pub struct EchoServer {
    id: String,
}

impl EchoServer {
    /// 创建新的 Echo Server
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    /// 处理 echo 工具调用
    fn echo(&self, text: String) -> ToolResult {
        ToolResult::text(text)
    }
}

#[async_trait]
impl MCPServer for EchoServer {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn tools(&self) -> Vec<Tool> {
        vec![Tool::with_schema(
            "echo",
            "Echo the input text back",
            json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to echo back"
                    }
                },
                "required": ["text"]
            }),
        )]
    }

    async fn handle_tool_call(&self, call: ToolCall) -> ToolResult {
        match call.tool_name.as_str() {
            "echo" => {
                // 提取 text 参数
                if let Some(text) = call.arguments.get("text").and_then(|v| v.as_str()) {
                    self.echo(text.to_string())
                } else {
                    ToolResult::error_text("Missing required parameter: text")
                }
            }
            _ => ToolResult::error_text(format!("Unknown tool: {}", call.tool_name)),
        }
    }

    async fn on_message(&self, _msg: mcp_server_framework::MCPMessage) -> Option<mcp_server_framework::MCPMessage> {
        // Echo Server 不需要处理消息
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_server_framework::ToolCall;

    #[tokio::test]
    async fn test_echo_server_tools() {
        let server = EchoServer::new("echo-1");
        let tools = server.tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "echo");
    }

    #[tokio::test]
    async fn test_echo_tool() {
        let server = EchoServer::new("echo-1");
        let call = ToolCall::new("echo", json!({"text": "hello world"}));
        let result = server.handle_tool_call(call).await;

        assert!(result.is_error.is_none());
        let content = result.content;
        assert_eq!(content["type"], "text");
        assert_eq!(content["text"], "hello world");
    }

    #[tokio::test]
    async fn test_echo_missing_param() {
        let server = EchoServer::new("echo-1");
        let call = ToolCall::new("echo", json!({}));
        let result = server.handle_tool_call(call).await;

        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let server = EchoServer::new("echo-1");
        let call = ToolCall::new("unknown", json!({}));
        let result = server.handle_tool_call(call).await;

        assert_eq!(result.is_error, Some(true));
    }
}
