//! Text Transformer Server
//!
//! 文本转换 Server

use async_trait::async_trait;
use mcp_server_framework::{MCPServer, Tool, ToolCall, ToolResult};
use serde_json::json;

/// Text Transformer Server
pub struct TextTransformerServer {
    id: String,
}

impl TextTransformerServer {
    /// 创建新的 Text Transformer Server
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    /// 转大写
    fn to_uppercase(text: &str) -> String {
        text.to_uppercase()
    }

    /// 转小写
    fn to_lowercase(text: &str) -> String {
        text.to_lowercase()
    }

    /// 反转字符串
    fn reverse(text: &str) -> String {
        text.chars().rev().collect()
    }

    /// Base64 编码
    #[cfg(feature = "text-tools")]
    fn base64_encode(text: &str) -> String {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        STANDARD.encode(text)
    }

    /// Base64 解码
    #[cfg(feature = "text-tools")]
    fn base64_decode(text: &str) -> Result<String, String> {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        STANDARD
            .decode(text)
            .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
            .map_err(|e| format!("Base64 decode error: {}", e))
    }

    /// 去除空白字符
    fn trim(text: &str) -> String {
        text.trim().to_string()
    }

    /// 替换文本
    fn replace(text: &str, from: &str, to: &str) -> String {
        text.replace(from, to)
    }
}

#[async_trait]
impl MCPServer for TextTransformerServer {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn tools(&self) -> Vec<Tool> {
        let mut tools = vec![
            Tool::with_schema(
                "to_uppercase",
                "Convert text to uppercase",
                json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to transform"}
                    },
                    "required": ["text"]
                }),
            ),
            Tool::with_schema(
                "to_lowercase",
                "Convert text to lowercase",
                json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to transform"}
                    },
                    "required": ["text"]
                }),
            ),
            Tool::with_schema(
                "reverse",
                "Reverse text characters",
                json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to reverse"}
                    },
                    "required": ["text"]
                }),
            ),
            Tool::with_schema(
                "trim",
                "Remove leading and trailing whitespace",
                json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to trim"}
                    },
                    "required": ["text"]
                }),
            ),
            Tool::with_schema(
                "replace",
                "Replace text occurrences",
                json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Original text"},
                        "from": {"type": "string", "description": "Text to replace"},
                        "to": {"type": "string", "description": "Replacement text"}
                    },
                    "required": ["text", "from", "to"]
                }),
            ),
        ];

        #[cfg(feature = "text-tools")]
        {
            tools.push(Tool::with_schema(
                "base64_encode",
                "Encode text to Base64",
                json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to encode"}
                    },
                    "required": ["text"]
                }),
            ));
            tools.push(Tool::with_schema(
                "base64_decode",
                "Decode Base64 text",
                json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Base64 text to decode"}
                    },
                    "required": ["text"]
                }),
            ));
        }

        tools
    }

    async fn handle_tool_call(&self, call: ToolCall) -> ToolResult {
        match call.tool_name.as_str() {
            "to_uppercase" => {
                let text = match call.arguments.get("text").and_then(|v| v.as_str()) {
                    Some(t) => t,
                    None => return ToolResult::error_text("Missing required parameter: text"),
                };
                ToolResult::success(json!({"result": Self::to_uppercase(text)}))
            }
            "to_lowercase" => {
                let text = match call.arguments.get("text").and_then(|v| v.as_str()) {
                    Some(t) => t,
                    None => return ToolResult::error_text("Missing required parameter: text"),
                };
                ToolResult::success(json!({"result": Self::to_lowercase(text)}))
            }
            "reverse" => {
                let text = match call.arguments.get("text").and_then(|v| v.as_str()) {
                    Some(t) => t,
                    None => return ToolResult::error_text("Missing required parameter: text"),
                };
                ToolResult::success(json!({"result": Self::reverse(text)}))
            }
            "trim" => {
                let text = match call.arguments.get("text").and_then(|v| v.as_str()) {
                    Some(t) => t,
                    None => return ToolResult::error_text("Missing required parameter: text"),
                };
                ToolResult::success(json!({"result": Self::trim(text)}))
            }
            "replace" => {
                let text = match call.arguments.get("text").and_then(|v| v.as_str()) {
                    Some(t) => t,
                    None => return ToolResult::error_text("Missing required parameter: text"),
                };
                let from = match call.arguments.get("from").and_then(|v| v.as_str()) {
                    Some(f) => f,
                    None => return ToolResult::error_text("Missing required parameter: from"),
                };
                let to = match call.arguments.get("to").and_then(|v| v.as_str()) {
                    Some(t) => t,
                    None => return ToolResult::error_text("Missing required parameter: to"),
                };
                ToolResult::success(json!({"result": Self::replace(text, from, to)}))
            }
            #[cfg(feature = "text-tools")]
            "base64_encode" => {
                let text = match call.arguments.get("text").and_then(|v| v.as_str()) {
                    Some(t) => t,
                    None => return ToolResult::error_text("Missing required parameter: text"),
                };
                ToolResult::success(json!({"result": Self::base64_encode(text)}))
            }
            #[cfg(feature = "text-tools")]
            "base64_decode" => {
                let text = match call.arguments.get("text").and_then(|v| v.as_str()) {
                    Some(t) => t,
                    None => return ToolResult::error_text("Missing required parameter: text"),
                };
                match Self::base64_decode(text) {
                    Ok(result) => ToolResult::success(json!({"result": result})),
                    Err(e) => ToolResult::error_text(e),
                }
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
    async fn test_text_transformer_tools() {
        let server = TextTransformerServer::new("text-transformer-1");
        let tools = server.tools();
        assert!(tools.len() >= 5); // 至少 5 个基本工具
    }

    #[tokio::test]
    async fn test_to_uppercase() {
        let server = TextTransformerServer::new("text-transformer-1");
        let call = ToolCall::new("to_uppercase", json!({"text": "hello"}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["result"], "HELLO");
    }

    #[tokio::test]
    async fn test_to_lowercase() {
        let server = TextTransformerServer::new("text-transformer-1");
        let call = ToolCall::new("to_lowercase", json!({"text": "HELLO"}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["result"], "hello");
    }

    #[tokio::test]
    async fn test_reverse() {
        let server = TextTransformerServer::new("text-transformer-1");
        let call = ToolCall::new("reverse", json!({"text": "hello"}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["result"], "olleh");
    }

    #[tokio::test]
    async fn test_trim() {
        let server = TextTransformerServer::new("text-transformer-1");
        let call = ToolCall::new("trim", json!({"text": "  hello  "}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["result"], "hello");
    }

    #[tokio::test]
    async fn test_replace() {
        let server = TextTransformerServer::new("text-transformer-1");
        let call = ToolCall::new("replace", json!({"text": "hello world", "from": "world", "to": "rust"}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["result"], "hello rust");
    }

    #[cfg(feature = "text-tools")]
    #[tokio::test]
    async fn test_base64_encode() {
        let server = TextTransformerServer::new("text-transformer-1");
        let call = ToolCall::new("base64_encode", json!({"text": "hello"}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["result"], "aGVsbG8=");
    }

    #[cfg(feature = "text-tools")]
    #[tokio::test]
    async fn test_base64_decode() {
        let server = TextTransformerServer::new("text-transformer-1");
        let call = ToolCall::new("base64_decode", json!({"text": "aGVsbG8="}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["result"], "hello");
    }
}
