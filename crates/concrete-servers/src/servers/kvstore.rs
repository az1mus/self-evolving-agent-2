//! Key-Value Store Server
//!
//! 键值存储 Server，支持 Session Cache 集成

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mcp_server_framework::{MCPServer, Tool, ToolCall, ToolResult};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 存储的值（带 TTL）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredValue {
    value: serde_json::Value,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

impl StoredValue {
    fn new(value: serde_json::Value, ttl_seconds: Option<i64>) -> Self {
        let now = Utc::now();
        Self {
            value,
            created_at: now,
            expires_at: ttl_seconds.map(|ttl| now + chrono::Duration::seconds(ttl)),
        }
    }

    fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(exp) => Utc::now() > exp,
            None => false,
        }
    }
}

/// KV Store Server
pub struct KVStoreServer {
    id: String,
    store: Arc<RwLock<HashMap<String, StoredValue>>>,
}

impl KVStoreServer {
    /// 创建新的 KV Store Server
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 设置键值
    async fn set(&self, key: String, value: serde_json::Value, ttl: Option<i64>) -> ToolResult {
        let stored = StoredValue::new(value.clone(), ttl);
        let mut store = self.store.write().await;
        store.insert(key.clone(), stored);
        ToolResult::success(json!({"key": key, "value": value, "stored": true}))
    }

    /// 获取键值
    async fn get(&self, key: &str) -> ToolResult {
        let mut store = self.store.write().await;
        match store.get(key) {
            Some(stored) if !stored.is_expired() => {
                ToolResult::success(json!({
                    "key": key,
                    "value": stored.value,
                    "created_at": stored.created_at.to_rfc3339()
                }))
            }
            Some(_) => {
                // 已过期，删除
                store.remove(key);
                ToolResult::error_text(format!("Key '{}' has expired", key))
            }
            None => ToolResult::error_text(format!("Key '{}' not found", key)),
        }
    }

    /// 删除键
    async fn delete(&self, key: &str) -> ToolResult {
        let mut store = self.store.write().await;
        match store.remove(key) {
            Some(_) => ToolResult::success(json!({"key": key, "deleted": true})),
            None => ToolResult::error_text(format!("Key '{}' not found", key)),
        }
    }

    /// 列出所有键
    async fn list_keys(&self) -> ToolResult {
        let store = self.store.read().await;
        let keys: Vec<String> = store
            .iter()
            .filter(|(_, v)| !v.is_expired())
            .map(|(k, _)| k.clone())
            .collect();
        ToolResult::success(json!({"keys": keys, "count": keys.len()}))
    }
}

#[async_trait]
impl MCPServer for KVStoreServer {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn tools(&self) -> Vec<Tool> {
        vec![
            Tool::with_schema(
                "set",
                "Set a key-value pair with optional TTL (in seconds)",
                json!({
                    "type": "object",
                    "properties": {
                        "key": {"type": "string", "description": "Key name"},
                        "value": {"description": "Value to store (any JSON value)"},
                        "ttl": {"type": "integer", "description": "Time-to-live in seconds (optional)"}
                    },
                    "required": ["key", "value"]
                }),
            ),
            Tool::with_schema(
                "get",
                "Get the value for a key",
                json!({
                    "type": "object",
                    "properties": {
                        "key": {"type": "string", "description": "Key name"}
                    },
                    "required": ["key"]
                }),
            ),
            Tool::with_schema(
                "delete",
                "Delete a key",
                json!({
                    "type": "object",
                    "properties": {
                        "key": {"type": "string", "description": "Key name"}
                    },
                    "required": ["key"]
                }),
            ),
            Tool::with_schema(
                "list_keys",
                "List all keys",
                json!({"type": "object", "properties": {}}),
            ),
        ]
    }

    async fn handle_tool_call(&self, call: ToolCall) -> ToolResult {
        match call.tool_name.as_str() {
            "set" => {
                let key = match call.arguments.get("key").and_then(|v| v.as_str()) {
                    Some(k) => k.to_string(),
                    None => return ToolResult::error_text("Missing required parameter: key"),
                };
                let value = match call.arguments.get("value") {
                    Some(v) => v.clone(),
                    None => return ToolResult::error_text("Missing required parameter: value"),
                };
                let ttl = call.arguments.get("ttl").and_then(|v| v.as_i64());
                self.set(key, value, ttl).await
            }
            "get" => {
                let key = match call.arguments.get("key").and_then(|v| v.as_str()) {
                    Some(k) => k,
                    None => return ToolResult::error_text("Missing required parameter: key"),
                };
                self.get(key).await
            }
            "delete" => {
                let key = match call.arguments.get("key").and_then(|v| v.as_str()) {
                    Some(k) => k,
                    None => return ToolResult::error_text("Missing required parameter: key"),
                };
                self.delete(key).await
            }
            "list_keys" => self.list_keys().await,
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
    async fn test_kvstore_tools() {
        let server = KVStoreServer::new("kvstore-1");
        let tools = server.tools();
        assert_eq!(tools.len(), 4);
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let server = KVStoreServer::new("kvstore-1");
        let call = ToolCall::new("set", json!({"key": "name", "value": "Alice"}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());

        let call = ToolCall::new("get", json!({"key": "name"}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["value"], "Alice");
    }

    #[tokio::test]
    async fn test_set_with_ttl() {
        let server = KVStoreServer::new("kvstore-1");
        let call = ToolCall::new("set", json!({"key": "temp", "value": "data", "ttl": 1}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());

        // 立即获取应该成功
        let call = ToolCall::new("get", json!({"key": "temp"}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());

        // 等待 2 秒后应该过期
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let call = ToolCall::new("get", json!({"key": "temp"}));
        let result = server.handle_tool_call(call).await;
        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_get_nonexistent_key() {
        let server = KVStoreServer::new("kvstore-1");
        let call = ToolCall::new("get", json!({"key": "nonexistent"}));
        let result = server.handle_tool_call(call).await;
        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_delete() {
        let server = KVStoreServer::new("kvstore-1");
        let call = ToolCall::new("set", json!({"key": "to_delete", "value": "data"}));
        server.handle_tool_call(call).await;

        let call = ToolCall::new("delete", json!({"key": "to_delete"}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["deleted"], true);

        // 删除后应该找不到
        let call = ToolCall::new("get", json!({"key": "to_delete"}));
        let result = server.handle_tool_call(call).await;
        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_list_keys() {
        let server = KVStoreServer::new("kvstore-1");
        let call = ToolCall::new("set", json!({"key": "k1", "value": 1}));
        server.handle_tool_call(call).await;
        let call = ToolCall::new("set", json!({"key": "k2", "value": 2}));
        server.handle_tool_call(call).await;

        let call = ToolCall::new("list_keys", json!({}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        let keys = result.content["keys"].as_array().unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[tokio::test]
    async fn test_complex_value() {
        let server = KVStoreServer::new("kvstore-1");
        let complex_value = json!({
            "name": "Alice",
            "age": 30,
            "skills": ["rust", "python"],
            "active": true
        });
        let call = ToolCall::new("set", json!({"key": "user", "value": complex_value}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());

        let call = ToolCall::new("get", json!({"key": "user"}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        let value = &result.content["value"];
        assert_eq!(value["name"], "Alice");
        assert_eq!(value["skills"][0], "rust");
    }
}
