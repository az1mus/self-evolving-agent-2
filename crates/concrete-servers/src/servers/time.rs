//! Time Server
//!
//! 返回当前时间和格式转换，用于测试无状态工具

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use mcp_server_framework::{MCPServer, Tool, ToolCall, ToolResult};
use serde_json::json;

/// Time Server
pub struct TimeServer {
    id: String,
}

impl TimeServer {
    /// 创建新的 Time Server
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    /// 获取当前 UTC 时间
    fn current_time(&self) -> ToolResult {
        let now = Utc::now();
        ToolResult::success(json!({
            "utc": now.to_rfc3339(),
            "timestamp": now.timestamp(),
        }))
    }

    /// 格式化时间
    fn format_time(&self, timestamp: i64, format: Option<String>) -> ToolResult {
        let dt: DateTime<Utc> = match Utc.timestamp_opt(timestamp, 0) {
            chrono::LocalResult::Single(dt) => dt,
            _ => return ToolResult::error_text("Invalid timestamp"),
        };

        let formatted = match format {
            Some(fmt) => match dt.format(&fmt).to_string() {
                s if s.contains("Invalid") || s.contains("Error") => {
                    return ToolResult::error_text(format!("Invalid format string: {}", fmt));
                }
                s => s,
            },
            None => dt.to_rfc3339(),
        };

        ToolResult::success(json!({
            "formatted": formatted,
            "timestamp": timestamp,
        }))
    }
}

#[async_trait]
impl MCPServer for TimeServer {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn tools(&self) -> Vec<Tool> {
        vec![
            Tool::with_schema(
                "current_time",
                "Get current UTC time",
                json!({"type": "object", "properties": {}}),
            ),
            Tool::with_schema(
                "format_time",
                "Format a Unix timestamp to a readable string",
                json!({
                    "type": "object",
                    "properties": {
                        "timestamp": {
                            "type": "integer",
                            "description": "Unix timestamp"
                        },
                        "format": {
                            "type": "string",
                            "description": "chrono format string (optional, defaults to RFC3339)"
                        }
                    },
                    "required": ["timestamp"]
                }),
            ),
        ]
    }

    async fn handle_tool_call(&self, call: ToolCall) -> ToolResult {
        match call.tool_name.as_str() {
            "current_time" => self.current_time(),
            "format_time" => {
                let timestamp = match call.arguments.get("timestamp").and_then(|v| v.as_i64()) {
                    Some(ts) => ts,
                    None => return ToolResult::error_text("Missing required parameter: timestamp"),
                };
                let format = call
                    .arguments
                    .get("format")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                self.format_time(timestamp, format)
            }
            _ => ToolResult::error_text(format!("Unknown tool: {}", call.tool_name)),
        }
    }

    async fn on_message(
        &self,
        _msg: mcp_server_framework::MCPMessage,
    ) -> Option<mcp_server_framework::MCPMessage> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_time_server_tools() {
        let server = TimeServer::new("time-1");
        let tools = server.tools();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "current_time");
        assert_eq!(tools[1].name, "format_time");
    }

    #[tokio::test]
    async fn test_current_time() {
        let server = TimeServer::new("time-1");
        let call = ToolCall::new("current_time", json!({}));
        let result = server.handle_tool_call(call).await;

        assert!(result.is_error.is_none());
        assert!(result.content.get("utc").is_some());
        assert!(result.content.get("timestamp").is_some());
    }

    #[tokio::test]
    async fn test_format_time() {
        let server = TimeServer::new("time-1");
        let call = ToolCall::new("format_time", json!({"timestamp": 1700000000}));
        let result = server.handle_tool_call(call).await;

        assert!(result.is_error.is_none());
        assert!(result.content.get("formatted").is_some());
    }

    #[tokio::test]
    async fn test_format_time_with_format() {
        let server = TimeServer::new("time-1");
        let call = ToolCall::new(
            "format_time",
            json!({"timestamp": 1700000000, "format": "%Y-%m-%d"}),
        );
        let result = server.handle_tool_call(call).await;

        assert!(result.is_error.is_none());
        assert_eq!(result.content["formatted"], "2023-11-14");
    }

    #[tokio::test]
    async fn test_format_time_missing_timestamp() {
        let server = TimeServer::new("time-1");
        let call = ToolCall::new("format_time", json!({}));
        let result = server.handle_tool_call(call).await;

        assert_eq!(result.is_error, Some(true));
    }
}
