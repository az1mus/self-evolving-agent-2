//! Counter Server
//!
//! 维护计数器状态的有状态 Server

use async_trait::async_trait;
use mcp_server_framework::{MCPServer, Tool, ToolCall, ToolResult};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Counter Server
///
/// 维护命名计数器，支持增减和查询
pub struct CounterServer {
    id: String,
    counters: Arc<RwLock<HashMap<String, i64>>>,
}

impl CounterServer {
    /// 创建新的 Counter Server
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            counters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 递增计数器
    async fn increment(&self, key: &str, delta: i64) -> i64 {
        let mut counters = self.counters.write().await;
        let counter = counters.entry(key.to_string()).or_insert(0);
        *counter += delta;
        *counter
    }

    /// 递减计数器
    async fn decrement(&self, key: &str, delta: i64) -> i64 {
        let mut counters = self.counters.write().await;
        let counter = counters.entry(key.to_string()).or_insert(0);
        *counter -= delta;
        *counter
    }

    /// 获取计数器值
    async fn get(&self, key: &str) -> Option<i64> {
        let counters = self.counters.read().await;
        counters.get(key).copied()
    }

    /// 列出所有计数器
    async fn list(&self) -> HashMap<String, i64> {
        let counters = self.counters.read().await;
        counters.clone()
    }
}

#[async_trait]
impl MCPServer for CounterServer {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn tools(&self) -> Vec<Tool> {
        vec![
            Tool::with_schema(
                "increment",
                "Increment a counter by a delta (default 1)",
                json!({
                    "type": "object",
                    "properties": {
                        "key": {"type": "string", "description": "Counter key name"},
                        "delta": {"type": "integer", "description": "Amount to increment (default: 1)"}
                    },
                    "required": ["key"]
                }),
            ),
            Tool::with_schema(
                "decrement",
                "Decrement a counter by a delta (default 1)",
                json!({
                    "type": "object",
                    "properties": {
                        "key": {"type": "string", "description": "Counter key name"},
                        "delta": {"type": "integer", "description": "Amount to decrement (default: 1)"}
                    },
                    "required": ["key"]
                }),
            ),
            Tool::with_schema(
                "get",
                "Get the current value of a counter",
                json!({
                    "type": "object",
                    "properties": {
                        "key": {"type": "string", "description": "Counter key name"}
                    },
                    "required": ["key"]
                }),
            ),
            Tool::with_schema(
                "list_counters",
                "List all counters and their values",
                json!({"type": "object", "properties": {}}),
            ),
        ]
    }

    async fn handle_tool_call(&self, call: ToolCall) -> ToolResult {
        match call.tool_name.as_str() {
            "increment" => {
                let key = match call.arguments.get("key").and_then(|v| v.as_str()) {
                    Some(k) => k,
                    None => return ToolResult::error_text("Missing required parameter: key"),
                };
                let delta = call.arguments.get("delta").and_then(|v| v.as_i64()).unwrap_or(1);
                let value = self.increment(key, delta).await;
                ToolResult::success(json!({"key": key, "value": value}))
            }
            "decrement" => {
                let key = match call.arguments.get("key").and_then(|v| v.as_str()) {
                    Some(k) => k,
                    None => return ToolResult::error_text("Missing required parameter: key"),
                };
                let delta = call.arguments.get("delta").and_then(|v| v.as_i64()).unwrap_or(1);
                let value = self.decrement(key, delta).await;
                ToolResult::success(json!({"key": key, "value": value}))
            }
            "get" => {
                let key = match call.arguments.get("key").and_then(|v| v.as_str()) {
                    Some(k) => k,
                    None => return ToolResult::error_text("Missing required parameter: key"),
                };
                match self.get(key).await {
                    Some(value) => ToolResult::success(json!({"key": key, "value": value})),
                    None => ToolResult::error_text(format!("Counter '{}' not found", key)),
                }
            }
            "list_counters" => {
                let counters = self.list().await;
                ToolResult::success(json!({"counters": counters}))
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
    async fn test_counter_tools() {
        let server = CounterServer::new("counter-1");
        let tools = server.tools();
        assert_eq!(tools.len(), 4);
    }

    #[tokio::test]
    async fn test_increment() {
        let server = CounterServer::new("counter-1");
        let call = ToolCall::new("increment", json!({"key": "hits"}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["value"], 1);
    }

    #[tokio::test]
    async fn test_increment_with_delta() {
        let server = CounterServer::new("counter-1");
        let call = ToolCall::new("increment", json!({"key": "hits", "delta": 5}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["value"], 5);
    }

    #[tokio::test]
    async fn test_decrement() {
        let server = CounterServer::new("counter-1");
        // 先增后减
        let call = ToolCall::new("increment", json!({"key": "hits", "delta": 10}));
        server.handle_tool_call(call).await;

        let call = ToolCall::new("decrement", json!({"key": "hits"}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["value"], 9);
    }

    #[tokio::test]
    async fn test_get_counter() {
        let server = CounterServer::new("counter-1");
        let call = ToolCall::new("increment", json!({"key": "views", "delta": 42}));
        server.handle_tool_call(call).await;

        let call = ToolCall::new("get", json!({"key": "views"}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["value"], 42);
    }

    #[tokio::test]
    async fn test_get_nonexistent_counter() {
        let server = CounterServer::new("counter-1");
        let call = ToolCall::new("get", json!({"key": "nonexistent"}));
        let result = server.handle_tool_call(call).await;
        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_list_counters() {
        let server = CounterServer::new("counter-1");
        let call = ToolCall::new("increment", json!({"key": "a", "delta": 1}));
        server.handle_tool_call(call).await;
        let call = ToolCall::new("increment", json!({"key": "b", "delta": 2}));
        server.handle_tool_call(call).await;

        let call = ToolCall::new("list_counters", json!({}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        let counters = result.content["counters"].as_object().unwrap();
        assert_eq!(counters.len(), 2);
    }

    #[tokio::test]
    async fn test_state_consistency_across_calls() {
        let server = CounterServer::new("counter-1");

        // 多次调用，验证状态一致性
        for i in 1..=5 {
            let call = ToolCall::new("increment", json!({"key": "test"}));
            let result = server.handle_tool_call(call).await;
            assert_eq!(result.content["value"], i);
        }

        let call = ToolCall::new("get", json!({"key": "test"}));
        let result = server.handle_tool_call(call).await;
        assert_eq!(result.content["value"], 5);
    }
}
