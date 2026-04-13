//! Code Review Server
//!
//! 代码审查 Server，调用 LLM Gateway Server（有机处理）

use async_trait::async_trait;
use mcp_server_framework::{MCPServer, Tool, ToolCall, ToolResult};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// 代码审查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    /// 审查摘要
    pub summary: String,
    /// 问题列表
    pub issues: Vec<ReviewIssue>,
    /// 评分 (0-10)
    pub score: u8,
    /// 是否通过
    pub passed: bool,
}

/// 审查问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    /// 严重程度
    pub severity: IssueSeverity,
    /// 问题描述
    pub description: String,
    /// 建议修复
    pub suggestion: Option<String>,
    /// 行号（可选）
    pub line: Option<u32>,
}

/// 问题严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// 改进建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub category: String,
    pub description: String,
    pub priority: u8,
}

/// Code Review Server
pub struct CodeReviewServer {
    id: String,
}

impl CodeReviewServer {
    /// 创建新的 Code Review Server
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    /// 审查代码（本地规则实现，不依赖 LLM）
    fn review_code(&self, code: &str, language: &str) -> ReviewResult {
        let mut issues = Vec::new();
        let mut score: u8 = 10;

        // 基本规则检查
        let lines: Vec<&str> = code.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let line_num = (i + 1) as u32;
            let trimmed = line.trim();

            // 检查过长行
            if line.len() > 120 {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Warning,
                    description: format!("Line too long ({} chars)", line.len()),
                    suggestion: Some("Consider breaking this line".to_string()),
                    line: Some(line_num),
                });
                score = score.saturating_sub(1);
            }

            // 检查 TODO/FIXME/HACK
            if trimmed.contains("TODO") || trimmed.contains("FIXME") || trimmed.contains("HACK") {
                issues.push(ReviewIssue {
                    severity: IssueSeverity::Info,
                    description: format!("Found marker: {}", if trimmed.contains("TODO") { "TODO" } else if trimmed.contains("FIXME") { "FIXME" } else { "HACK" }),
                    suggestion: Some("Consider resolving this before merging".to_string()),
                    line: Some(line_num),
                });
            }

            // 检查空 catch 块
            if trimmed.contains("catch") && !trimmed.contains('{') {
                // 下一行可能是空块
                if i + 1 < lines.len() && lines[i + 1].trim() == "}" {
                    issues.push(ReviewIssue {
                        severity: IssueSeverity::Warning,
                        description: "Empty catch block detected".to_string(),
                        suggestion: Some("Handle the exception or log it".to_string()),
                        line: Some(line_num),
                    });
                    score = score.saturating_sub(1);
                }
            }

            // 检查 console.log/print 调试语句
            if trimmed.contains("console.log") || trimmed.contains("println!") || trimmed.contains("print!(") {
                if !trimmed.starts_with("//") {
                    issues.push(ReviewIssue {
                        severity: IssueSeverity::Warning,
                        description: "Debug print statement detected".to_string(),
                        suggestion: Some("Remove debug statements before merging".to_string()),
                        line: Some(line_num),
                    });
                    score = score.saturating_sub(1);
                }
            }
        }

        // 通用检查
        if code.is_empty() {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Error,
                description: "Empty code".to_string(),
                suggestion: None,
                line: None,
            });
            score = 0;
        }

        if code.len() > 10000 {
            issues.push(ReviewIssue {
                severity: IssueSeverity::Warning,
                description: "Code is very long, consider splitting into modules".to_string(),
                suggestion: Some("Break into smaller functions/modules".to_string()),
                line: None,
            });
            score = score.saturating_sub(1);
        }

        let passed = score >= 6;
        let summary = format!(
            "Code review for {}: Found {} issue(s). Score: {}/10. {}",
            language,
            issues.len(),
            score,
            if passed { "PASSED" } else { "NEEDS IMPROVEMENT" }
        );

        ReviewResult {
            summary,
            issues,
            score,
            passed,
        }
    }

    /// 生成改进建议
    fn suggest_improvements(&self, code: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        let line_count = code.lines().count();
        if line_count > 50 {
            suggestions.push(Suggestion {
                category: "modularity".to_string(),
                description: "Consider breaking this code into smaller functions".to_string(),
                priority: 7,
            });
        }

        if !code.contains("test") && !code.contains("Test") {
            suggestions.push(Suggestion {
                category: "testing".to_string(),
                description: "No tests detected. Consider adding unit tests".to_string(),
                priority: 8,
            });
        }

        if !code.contains("doc") && !code.contains("///") && !code.contains("//!") {
            suggestions.push(Suggestion {
                category: "documentation".to_string(),
                description: "Consider adding documentation comments".to_string(),
                priority: 5,
            });
        }

        if code.matches("unwrap()").count() > 3 {
            suggestions.push(Suggestion {
                category: "error_handling".to_string(),
                description: "Multiple unwrap() calls found. Consider proper error handling".to_string(),
                priority: 9,
            });
        }

        suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));
        suggestions
    }
}

#[async_trait]
impl MCPServer for CodeReviewServer {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn tools(&self) -> Vec<Tool> {
        vec![
            Tool::with_schema(
                "review_code",
                "Review code and provide feedback",
                json!({
                    "type": "object",
                    "properties": {
                        "code": {"type": "string", "description": "Code to review"},
                        "language": {"type": "string", "description": "Programming language"}
                    },
                    "required": ["code", "language"]
                }),
            ),
            Tool::with_schema(
                "suggest_improvements",
                "Suggest improvements for code",
                json!({
                    "type": "object",
                    "properties": {
                        "code": {"type": "string", "description": "Code to analyze"}
                    },
                    "required": ["code"]
                }),
            ),
        ]
    }

    async fn handle_tool_call(&self, call: ToolCall) -> ToolResult {
        match call.tool_name.as_str() {
            "review_code" => {
                let code = match call.arguments.get("code").and_then(|v| v.as_str()) {
                    Some(c) => c,
                    None => return ToolResult::error_text("Missing required parameter: code"),
                };
                let language = call
                    .arguments
                    .get("language")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                let result = self.review_code(code, language);
                match serde_json::to_value(result) {
                    Ok(value) => ToolResult::success(value),
                    Err(e) => ToolResult::error_text(format!("Serialization error: {}", e)),
                }
            }
            "suggest_improvements" => {
                let code = match call.arguments.get("code").and_then(|v| v.as_str()) {
                    Some(c) => c,
                    None => return ToolResult::error_text("Missing required parameter: code"),
                };

                let suggestions = self.suggest_improvements(code);
                ToolResult::success(json!({"suggestions": suggestions}))
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
    async fn test_code_review_tools() {
        let server = CodeReviewServer::new("code-review-1");
        let tools = server.tools();
        assert_eq!(tools.len(), 2);
    }

    #[tokio::test]
    async fn test_review_clean_code() {
        let server = CodeReviewServer::new("code-review-1");
        let call = ToolCall::new(
            "review_code",
            json!({"code": "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}", "language": "rust"}),
        );
        let result = server.handle_tool_call(call).await;

        assert!(result.is_error.is_none());
        assert!(result.content["passed"].as_bool().unwrap());
        assert!(result.content["score"].as_u64().unwrap() >= 8);
    }

    #[tokio::test]
    async fn test_review_code_with_issues() {
        let server = CodeReviewServer::new("code-review-1");
        let long_line = "x".repeat(150);
        let code = format!("fn main() {{\n    println!(\"hello\");\n    let long = \"{}\";\n}}\n// TODO: fix this", long_line);
        let call = ToolCall::new("review_code", json!({"code": code, "language": "rust"}));
        let result = server.handle_tool_call(call).await;

        assert!(result.is_error.is_none());
        let issues = result.content["issues"].as_array().unwrap();
        assert!(!issues.is_empty());
    }

    #[tokio::test]
    async fn test_review_empty_code() {
        let server = CodeReviewServer::new("code-review-1");
        let call = ToolCall::new("review_code", json!({"code": "", "language": "rust"}));
        let result = server.handle_tool_call(call).await;

        assert!(result.is_error.is_none());
        assert!(!result.content["passed"].as_bool().unwrap());
        assert_eq!(result.content["score"], 0);
    }

    #[tokio::test]
    async fn test_suggest_improvements() {
        let server = CodeReviewServer::new("code-review-1");
        let code = "fn main() { let x = something.unwrap(); let y = other.unwrap(); let z = third.unwrap(); let w = fourth.unwrap(); }";
        let call = ToolCall::new("suggest_improvements", json!({"code": code}));
        let result = server.handle_tool_call(call).await;

        assert!(result.is_error.is_none());
        let suggestions = result.content["suggestions"].as_array().unwrap();
        assert!(!suggestions.is_empty());
    }
}