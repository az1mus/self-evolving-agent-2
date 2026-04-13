//! Text Analyzer Server
//!
//! 文本分析 Server

use async_trait::async_trait;
use mcp_server_framework::{MCPServer, Tool, ToolCall, ToolResult};
use serde_json::json;

/// Text Analyzer Server
pub struct TextAnalyzerServer {
    id: String,
}

impl TextAnalyzerServer {
    /// 创建新的 Text Analyzer Server
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    /// 统计单词数
    fn word_count(text: &str) -> usize {
        text.split_whitespace().count()
    }

    /// 统计字符数
    fn char_count(text: &str) -> usize {
        text.chars().count()
    }

    /// 统计行数
    fn line_count(text: &str) -> usize {
        text.lines().count()
    }

    /// 统计句子数（简单实现）
    fn sentence_count(text: &str) -> usize {
        text.matches(|c| c == '.' || c == '!' || c == '?').count()
    }

    /// 提取关键词（简单实现：返回最常见的单词）
    fn extract_keywords(text: &str, limit: usize) -> Vec<String> {
        use std::collections::HashMap;
        let mut word_freq: HashMap<String, usize> = HashMap::new();

        for word in text.split_whitespace() {
            let word = word.to_lowercase();
            let word = word.trim_matches(|c: char| !c.is_alphanumeric());
            if word.len() > 2 {
                *word_freq.entry(word.to_string()).or_insert(0) += 1;
            }
        }

        let mut freq_vec: Vec<_> = word_freq.into_iter().collect();
        freq_vec.sort_by(|a, b| b.1.cmp(&a.1));
        freq_vec.truncate(limit);
        freq_vec.into_iter().map(|(word, _)| word).collect()
    }

    /// 文本统计
    fn analyze(text: &str) -> serde_json::Value {
        json!({
            "char_count": Self::char_count(text),
            "word_count": Self::word_count(text),
            "line_count": Self::line_count(text),
            "sentence_count": Self::sentence_count(text),
        })
    }
}

#[async_trait]
impl MCPServer for TextAnalyzerServer {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn tools(&self) -> Vec<Tool> {
        vec![
            Tool::with_schema(
                "word_count",
                "Count words in text",
                json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to analyze"}
                    },
                    "required": ["text"]
                }),
            ),
            Tool::with_schema(
                "char_count",
                "Count characters in text",
                json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to analyze"}
                    },
                    "required": ["text"]
                }),
            ),
            Tool::with_schema(
                "analyze",
                "Get comprehensive text statistics",
                json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to analyze"}
                    },
                    "required": ["text"]
                }),
            ),
            Tool::with_schema(
                "extract_keywords",
                "Extract top N keywords from text",
                json!({
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Text to analyze"},
                        "limit": {"type": "integer", "description": "Maximum number of keywords (default: 5)"}
                    },
                    "required": ["text"]
                }),
            ),
        ]
    }

    async fn handle_tool_call(&self, call: ToolCall) -> ToolResult {
        match call.tool_name.as_str() {
            "word_count" => {
                let text = match call.arguments.get("text").and_then(|v| v.as_str()) {
                    Some(t) => t,
                    None => return ToolResult::error_text("Missing required parameter: text"),
                };
                ToolResult::success(json!({"word_count": Self::word_count(text)}))
            }
            "char_count" => {
                let text = match call.arguments.get("text").and_then(|v| v.as_str()) {
                    Some(t) => t,
                    None => return ToolResult::error_text("Missing required parameter: text"),
                };
                ToolResult::success(json!({"char_count": Self::char_count(text)}))
            }
            "analyze" => {
                let text = match call.arguments.get("text").and_then(|v| v.as_str()) {
                    Some(t) => t,
                    None => return ToolResult::error_text("Missing required parameter: text"),
                };
                ToolResult::success(Self::analyze(text))
            }
            "extract_keywords" => {
                let text = match call.arguments.get("text").and_then(|v| v.as_str()) {
                    Some(t) => t,
                    None => return ToolResult::error_text("Missing required parameter: text"),
                };
                let limit = call
                    .arguments
                    .get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5) as usize;
                let keywords = Self::extract_keywords(text, limit);
                ToolResult::success(json!({"keywords": keywords}))
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
    async fn test_text_analyzer_tools() {
        let server = TextAnalyzerServer::new("text-analyzer-1");
        let tools = server.tools();
        assert_eq!(tools.len(), 4);
    }

    #[tokio::test]
    async fn test_word_count() {
        let server = TextAnalyzerServer::new("text-analyzer-1");
        let call = ToolCall::new("word_count", json!({"text": "hello world test"}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["word_count"], 3);
    }

    #[tokio::test]
    async fn test_char_count() {
        let server = TextAnalyzerServer::new("text-analyzer-1");
        let call = ToolCall::new("char_count", json!({"text": "hello"}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["char_count"], 5);
    }

    #[tokio::test]
    async fn test_analyze() {
        let server = TextAnalyzerServer::new("text-analyzer-1");
        let call = ToolCall::new(
            "analyze",
            json!({"text": "Hello world. This is a test."}),
        );
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["word_count"], 6);
        assert_eq!(result.content["sentence_count"], 2);
    }

    #[tokio::test]
    async fn test_extract_keywords() {
        let server = TextAnalyzerServer::new("text-analyzer-1");
        let call = ToolCall::new(
            "extract_keywords",
            json!({"text": "rust is great. rust is fast. rust is safe.", "limit": 3}),
        );
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        let keywords = result.content["keywords"].as_array().unwrap();
        assert!(keywords.contains(&json!("rust")));
    }

    #[tokio::test]
    async fn test_empty_text() {
        let server = TextAnalyzerServer::new("text-analyzer-1");
        let call = ToolCall::new("word_count", json!({"text": ""}));
        let result = server.handle_tool_call(call).await;
        assert!(result.is_error.is_none());
        assert_eq!(result.content["word_count"], 0);
    }
}
