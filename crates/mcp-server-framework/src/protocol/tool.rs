use serde::{Deserialize, Serialize};

/// MCP 工具定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tool {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 输入 Schema (JSON Schema)
    pub input_schema: serde_json::Value,
}

/// 工具调用请求
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolCall {
    /// 工具名称
    pub tool_name: String,
    /// 调用参数
    pub arguments: serde_json::Value,
}

/// 工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolResult {
    /// 结果内容
    pub content: serde_json::Value,
    /// 是否为错误
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

// ============================================================================
// 实现辅助方法
// ============================================================================

impl Tool {
    /// 创建工具定义
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema: serde_json::json!({"type": "object"}),
        }
    }

    /// 创建工具定义 (带 Schema)
    pub fn with_schema(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

impl ToolCall {
    /// 创建工具调用
    pub fn new(tool_name: impl Into<String>, arguments: serde_json::Value) -> Self {
        Self {
            tool_name: tool_name.into(),
            arguments,
        }
    }
}

impl ToolResult {
    /// 创建成功结果
    pub fn success(content: serde_json::Value) -> Self {
        Self {
            content,
            is_error: None,
        }
    }

    /// 创建错误结果
    pub fn error(content: serde_json::Value) -> Self {
        Self {
            content,
            is_error: Some(true),
        }
    }

    /// 创建文本结果
    pub fn text(text: impl Into<String>) -> Self {
        Self::success(serde_json::json!({
            "type": "text",
            "text": text.into()
        }))
    }

    /// 创建错误文本结果
    pub fn error_text(text: impl Into<String>) -> Self {
        Self::error(serde_json::json!({
            "type": "text",
            "text": text.into()
        }))
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_creation() {
        let tool = Tool::new("echo", "Echo the input");
        assert_eq!(tool.name, "echo");
        assert_eq!(tool.description, "Echo the input");
    }

    #[test]
    fn test_tool_with_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "message": {"type": "string"}
            },
            "required": ["message"]
        });

        let tool = Tool::with_schema("echo", "Echo the input", schema.clone());
        assert_eq!(tool.name, "echo");
        assert_eq!(tool.input_schema, schema);
    }

    #[test]
    fn test_tool_call() {
        let call = ToolCall::new("echo", json!({"message": "hello"}));
        assert_eq!(call.tool_name, "echo");
        assert_eq!(call.arguments, json!({"message": "hello"}));
    }

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success(json!({"output": "hello"}));
        assert!(result.is_error.is_none());
        assert_eq!(result.content, json!({"output": "hello"}));
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error(json!({"error": "failed"}));
        assert_eq!(result.is_error, Some(true));
        assert_eq!(result.content, json!({"error": "failed"}));
    }

    #[test]
    fn test_tool_result_text() {
        let result = ToolResult::text("hello world");
        assert_eq!(
            result.content,
            json!({
                "type": "text",
                "text": "hello world"
            })
        );
        assert!(result.is_error.is_none());
    }

    #[test]
    fn test_tool_serialization() {
        let tool = Tool::with_schema(
            "calculate",
            "Calculate expression",
            json!({"type": "object"}),
        );
        let json_str = serde_json::to_string(&tool).unwrap();

        let decoded: Tool = serde_json::from_str(&json_str).unwrap();
        assert_eq!(decoded.name, "calculate");
        assert_eq!(decoded.description, "Calculate expression");
    }
}
