//! Calculator Server
//!
//! 基础数学运算 Server，用于测试工具调用

use async_trait::async_trait;
use mcp_server_framework::{MCPServer, Tool, ToolCall, ToolResult};
use serde_json::json;

/// Calculator Server
pub struct CalculatorServer {
    id: String,
}

impl CalculatorServer {
    /// 创建新的 Calculator Server
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    fn add(&self, a: f64, b: f64) -> f64 {
        a + b
    }

    fn subtract(&self, a: f64, b: f64) -> f64 {
        a - b
    }

    fn multiply(&self, a: f64, b: f64) -> f64 {
        a * b
    }

    fn divide(&self, a: f64, b: f64) -> Result<f64, String> {
        if b == 0.0 {
            Err("Division by zero".to_string())
        } else {
            Ok(a / b)
        }
    }

    /// 从参数中提取两个数字
    fn extract_two_numbers(args: &serde_json::Value) -> Result<(f64, f64), String> {
        let a = args
            .get("a")
            .and_then(|v| v.as_f64())
            .ok_or("Missing or invalid parameter: a")?;
        let b = args
            .get("b")
            .and_then(|v| v.as_f64())
            .ok_or("Missing or invalid parameter: b")?;
        Ok((a, b))
    }
}

#[async_trait]
impl MCPServer for CalculatorServer {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn tools(&self) -> Vec<Tool> {
        let number_schema = json!({
            "type": "object",
            "properties": {
                "a": {"type": "number", "description": "First number"},
                "b": {"type": "number", "description": "Second number"}
            },
            "required": ["a", "b"]
        });

        vec![
            Tool::with_schema("add", "Add two numbers", number_schema.clone()),
            Tool::with_schema("subtract", "Subtract b from a", number_schema.clone()),
            Tool::with_schema("multiply", "Multiply two numbers", number_schema.clone()),
            Tool::with_schema(
                "divide",
                "Divide a by b",
                number_schema,
            ),
        ]
    }

    async fn handle_tool_call(&self, call: ToolCall) -> ToolResult {
        match call.tool_name.as_str() {
            "add" => {
                match Self::extract_two_numbers(&call.arguments) {
                    Ok((a, b)) => ToolResult::success(json!({"result": self.add(a, b)})),
                    Err(e) => ToolResult::error_text(e),
                }
            }
            "subtract" => {
                match Self::extract_two_numbers(&call.arguments) {
                    Ok((a, b)) => ToolResult::success(json!({"result": self.subtract(a, b)})),
                    Err(e) => ToolResult::error_text(e),
                }
            }
            "multiply" => {
                match Self::extract_two_numbers(&call.arguments) {
                    Ok((a, b)) => ToolResult::success(json!({"result": self.multiply(a, b)})),
                    Err(e) => ToolResult::error_text(e),
                }
            }
            "divide" => {
                match Self::extract_two_numbers(&call.arguments) {
                    Ok((a, b)) => match self.divide(a, b) {
                        Ok(result) => ToolResult::success(json!({"result": result})),
                        Err(e) => ToolResult::error_text(e),
                    },
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
    async fn test_calculator_tools() {
        let server = CalculatorServer::new("calc-1");
        let tools = server.tools();
        assert_eq!(tools.len(), 4);
        assert_eq!(tools[0].name, "add");
        assert_eq!(tools[1].name, "subtract");
        assert_eq!(tools[2].name, "multiply");
        assert_eq!(tools[3].name, "divide");
    }

    #[tokio::test]
    async fn test_add() {
        let server = CalculatorServer::new("calc-1");
        let call = ToolCall::new("add", json!({"a": 2, "b": 3}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["result"], 5.0);
    }

    #[tokio::test]
    async fn test_subtract() {
        let server = CalculatorServer::new("calc-1");
        let call = ToolCall::new("subtract", json!({"a": 10, "b": 4}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["result"], 6.0);
    }

    #[tokio::test]
    async fn test_multiply() {
        let server = CalculatorServer::new("calc-1");
        let call = ToolCall::new("multiply", json!({"a": 3, "b": 7}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["result"], 21.0);
    }

    #[tokio::test]
    async fn test_divide() {
        let server = CalculatorServer::new("calc-1");
        let call = ToolCall::new("divide", json!({"a": 15, "b": 3}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["result"], 5.0);
    }

    #[tokio::test]
    async fn test_divide_by_zero() {
        let server = CalculatorServer::new("calc-1");
        let call = ToolCall::new("divide", json!({"a": 10, "b": 0}));
        let result = server.handle_tool_call(call).await;
        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_missing_params() {
        let server = CalculatorServer::new("calc-1");
        let call = ToolCall::new("add", json!({"a": 1}));
        let result = server.handle_tool_call(call).await;
        assert_eq!(result.is_error, Some(true));
    }
}
