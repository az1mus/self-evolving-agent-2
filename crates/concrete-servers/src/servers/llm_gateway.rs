//! LLM Gateway Server
//!
//! 调用 LLM API 的 Server

use async_trait::async_trait;
use mcp_server_framework::{MCPServer, Tool, ToolCall, ToolResult};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// LLM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// API Key
    pub api_key: Option<String>,
    /// 模型名称
    pub model: String,
    /// API Base URL
    pub base_url: String,
    /// 最大 Token 数
    pub max_tokens: u32,
    /// 温度
    pub temperature: f32,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: "claude-sonnet-4-6".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
        }
    }
}

/// LLM Gateway Server
///
/// 调用 LLM API 进行文本生成和对话
pub struct LLMGatewayServer {
    id: String,
    config: LLMConfig,
}

impl LLMGatewayServer {
    /// 创建新的 LLM Gateway Server
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            config: LLMConfig::default(),
        }
    }

    /// 创建带配置的 LLM Gateway Server
    pub fn with_config(id: impl Into<String>, config: LLMConfig) -> Self {
        Self {
            id: id.into(),
            config,
        }
    }

    /// 完成 (completion) 接口
    async fn complete(&self, prompt: &str, _options: Option<serde_json::Value>) -> ToolResult {
        // 如果没有 API Key，返回模拟响应
        if self.config.api_key.is_none() || self.config.api_key.as_ref().map(|k| k.is_empty()).unwrap_or(true) {
            return ToolResult::success(json!({
                "response": format!("[Mock LLM Response] Prompt: {}", prompt),
                "model": self.config.model,
                "mock": true
            }));
        }

        #[cfg(feature = "llm")]
        {
            let max_tokens = options
                .as_ref()
                .and_then(|o| o.get("max_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(self.config.max_tokens as u64) as u32;

            let client = reqwest::Client::new();
            let response = client
                .post(format!("{}/v1/messages", self.config.base_url))
                .header("x-api-key", self.config.api_key.as_ref().unwrap())
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&json!({
                    "model": self.config.model,
                    "max_tokens": max_tokens,
                    "messages": [{"role": "user", "content": prompt}]
                }))
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    match resp.json::<serde_json::Value>().await {
                        Ok(body) => ToolResult::success(json!({
                            "status": status,
                            "response": body
                        })),
                        Err(e) => ToolResult::error_text(format!("Failed to parse response: {}", e)),
                    }
                }
                Err(e) => ToolResult::error_text(format!("LLM API request failed: {}", e)),
            }
        }

        #[cfg(not(feature = "llm"))]
        {
            ToolResult::success(json!({
                "response": format!("[Mock LLM Response] Prompt: {}", prompt),
                "model": self.config.model,
                "mock": true
            }))
        }
    }

    /// 对话 (chat) 接口
    async fn chat(&self, messages: &serde_json::Value, _options: Option<serde_json::Value>) -> ToolResult {
        let messages_array = match messages.as_array() {
            Some(arr) => arr,
            None => return ToolResult::error_text("messages must be an array"),
        };

        // 如果没有 API Key，返回模拟响应
        if self.config.api_key.is_none() || self.config.api_key.as_ref().map(|k| k.is_empty()).unwrap_or(true) {
            let last_msg = messages_array
                .last()
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .unwrap_or("(empty)");
            return ToolResult::success(json!({
                "response": format!("[Mock LLM Chat Response] Last message: {}", last_msg),
                "model": self.config.model,
                "message_count": messages_array.len(),
                "mock": true
            }));
        }

        #[cfg(feature = "llm")]
        {
            let max_tokens = options
                .as_ref()
                .and_then(|o| o.get("max_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(self.config.max_tokens as u64) as u32;

            let client = reqwest::Client::new();
            let response = client
                .post(format!("{}/v1/messages", self.config.base_url))
                .header("x-api-key", self.config.api_key.as_ref().unwrap())
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&json!({
                    "model": self.config.model,
                    "max_tokens": max_tokens,
                    "messages": messages
                }))
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    match resp.json::<serde_json::Value>().await {
                        Ok(body) => ToolResult::success(json!({
                            "status": status,
                            "response": body
                        })),
                        Err(e) => ToolResult::error_text(format!("Failed to parse response: {}", e)),
                    }
                }
                Err(e) => ToolResult::error_text(format!("LLM API request failed: {}", e)),
            }
        }

        #[cfg(not(feature = "llm"))]
        {
            let last_msg = messages_array
                .last()
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .unwrap_or("(empty)");
            ToolResult::success(json!({
                "response": format!("[Mock LLM Chat Response] Last message: {}", last_msg),
                "model": self.config.model,
                "message_count": messages_array.len(),
                "mock": true
            }))
        }
    }
}

#[async_trait]
impl MCPServer for LLMGatewayServer {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn tools(&self) -> Vec<Tool> {
        vec![
            Tool::with_schema(
                "complete",
                "Generate text completion using LLM",
                json!({
                    "type": "object",
                    "properties": {
                        "prompt": {"type": "string", "description": "Text prompt"},
                        "options": {
                            "type": "object",
                            "description": "Optional parameters (max_tokens, temperature, etc.)"
                        }
                    },
                    "required": ["prompt"]
                }),
            ),
            Tool::with_schema(
                "chat",
                "Have a chat conversation using LLM",
                json!({
                    "type": "object",
                    "properties": {
                        "messages": {
                            "type": "array",
                            "description": "Array of message objects with role and content",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "role": {"type": "string"},
                                    "content": {"type": "string"}
                                }
                            }
                        },
                        "options": {
                            "type": "object",
                            "description": "Optional parameters"
                        }
                    },
                    "required": ["messages"]
                }),
            ),
        ]
    }

    async fn handle_tool_call(&self, call: ToolCall) -> ToolResult {
        match call.tool_name.as_str() {
            "complete" => {
                let prompt = match call.arguments.get("prompt").and_then(|v| v.as_str()) {
                    Some(p) => p,
                    None => return ToolResult::error_text("Missing required parameter: prompt"),
                };
                let options = call.arguments.get("options").cloned();
                self.complete(prompt, options).await
            }
            "chat" => {
                let messages = match call.arguments.get("messages") {
                    Some(m) => m,
                    None => return ToolResult::error_text("Missing required parameter: messages"),
                };
                let options = call.arguments.get("options").cloned();
                self.chat(messages, options).await
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

    #[test]
    fn test_llm_gateway_tools() {
        let server = LLMGatewayServer::new("llm-1");
        let tools = server.tools();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "complete");
        assert_eq!(tools[1].name, "chat");
    }

    #[tokio::test]
    async fn test_mock_complete() {
        let server = LLMGatewayServer::new("llm-1");
        let call = ToolCall::new("complete", json!({"prompt": "Hello"}));
        let result = server.handle_tool_call(call).await;

        assert!(result.is_error.is_none());
        assert_eq!(result.content["mock"], true);
        assert!(result.content["response"].as_str().unwrap().contains("Hello"));
    }

    #[tokio::test]
    async fn test_mock_chat() {
        let server = LLMGatewayServer::new("llm-1");
        let call = ToolCall::new(
            "chat",
            json!({"messages": [{"role": "user", "content": "Hi there"}]}),
        );
        let result = server.handle_tool_call(call).await;

        assert!(result.is_error.is_none());
        assert_eq!(result.content["mock"], true);
    }

    #[test]
    fn test_llm_config_default() {
        let config = LLMConfig::default();
        assert_eq!(config.model, "claude-sonnet-4-6");
        assert_eq!(config.max_tokens, 4096);
        assert!(config.api_key.is_none());
    }
}
