//! File I/O Server
//!
//! 文件读写 Server（受限路径）

use async_trait::async_trait;
use mcp_server_framework::{MCPServer, Tool, ToolCall, ToolResult};
use serde_json::json;
use std::path::PathBuf;

/// File I/O Server
///
/// 安全限制：只允许访问指定目录
pub struct FileIOServer {
    id: String,
    allowed_dirs: Vec<PathBuf>,
}

impl FileIOServer {
    /// 创建新的 File I/O Server
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            allowed_dirs: vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))],
        }
    }

    /// 创建带受限目录的 File I/O Server
    pub fn with_allowed_dirs(id: impl Into<String>, allowed_dirs: Vec<PathBuf>) -> Self {
        Self {
            id: id.into(),
            allowed_dirs,
        }
    }

    /// 检查路径是否在允许范围内
    fn is_path_allowed(&self, path: &PathBuf) -> bool {
        // 如果没有设置允许目录，则拒绝所有
        if self.allowed_dirs.is_empty() {
            return false;
        }

        // 检查路径是否在任一允许目录下
        self.allowed_dirs.iter().any(|dir| {
            path.starts_with(dir) || path.canonicalize().map(|p| p.starts_with(dir)).unwrap_or(false)
        })
    }

    /// 规范化路径
    fn normalize_path(&self, path: &str) -> Result<PathBuf, String> {
        let path = PathBuf::from(path);

        // 防止路径遍历攻击
        let canonical = path
            .canonicalize()
            .map_err(|e| format!("Invalid path: {}", e))?;

        if !self.is_path_allowed(&canonical) {
            return Err("Path is outside allowed directories".to_string());
        }

        Ok(canonical)
    }

    /// 读取文件
    async fn read_file(&self, path: &str) -> ToolResult {
        match self.normalize_path(path) {
            Ok(canonical) => match tokio::fs::read_to_string(&canonical).await {
                Ok(content) => ToolResult::success(json!({
                    "path": path,
                    "content": content
                })),
                Err(e) => ToolResult::error_text(format!("Failed to read file: {}", e)),
            },
            Err(e) => ToolResult::error_text(e),
        }
    }

    /// 写入文件
    async fn write_file(&self, path: &str, content: &str) -> ToolResult {
        let path_buf = PathBuf::from(path);

        // 检查路径是否在允许目录下
        // 对于新文件，路径可能不存在，所以我们检查父目录
        let check_path = if path_buf.exists() {
            path_buf.clone()
        } else {
            match path_buf.parent() {
                Some(parent) => parent.to_path_buf(),
                None => return ToolResult::error_text("Invalid path"),
            }
        };

        // 如果检查路径存在，则验证它是否在允许范围内
        if check_path.exists() {
            if let Ok(canonical) = check_path.canonicalize() {
                if !self.is_path_allowed(&canonical) {
                    return ToolResult::error_text("Path is outside allowed directories");
                }
            }
        }

        match tokio::fs::write(&path_buf, content).await {
            Ok(_) => ToolResult::success(json!({
                "path": path,
                "written": true
            })),
            Err(e) => ToolResult::error_text(format!("Failed to write file: {}", e)),
        }
    }

    /// 列出目录内容
    async fn list_dir(&self, path: &str) -> ToolResult {
        match self.normalize_path(path) {
            Ok(canonical) => {
                let mut entries = Vec::new();
                let mut read_dir = match tokio::fs::read_dir(&canonical).await {
                    Ok(rd) => rd,
                    Err(e) => return ToolResult::error_text(format!("Failed to read directory: {}", e)),
                };

                while let Ok(Some(entry)) = read_dir.next_entry().await {
                    if let Ok(name) = entry.file_name().into_string() {
                        entries.push(name);
                    }
                }

                ToolResult::success(json!({
                    "path": path,
                    "entries": entries
                }))
            }
            Err(e) => ToolResult::error_text(e),
        }
    }
}

#[async_trait]
impl MCPServer for FileIOServer {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn tools(&self) -> Vec<Tool> {
        vec![
            Tool::with_schema(
                "read_file",
                "Read file content",
                json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "File path (must be within allowed directories)"}
                    },
                    "required": ["path"]
                }),
            ),
            Tool::with_schema(
                "write_file",
                "Write content to file",
                json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "File path (must be within allowed directories)"},
                        "content": {"type": "string", "description": "Content to write"}
                    },
                    "required": ["path", "content"]
                }),
            ),
            Tool::with_schema(
                "list_dir",
                "List directory contents",
                json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Directory path (must be within allowed directories)"}
                    },
                    "required": ["path"]
                }),
            ),
        ]
    }

    async fn handle_tool_call(&self, call: ToolCall) -> ToolResult {
        match call.tool_name.as_str() {
            "read_file" => {
                let path = match call.arguments.get("path").and_then(|v| v.as_str()) {
                    Some(p) => p,
                    None => return ToolResult::error_text("Missing required parameter: path"),
                };
                self.read_file(path).await
            }
            "write_file" => {
                let path = match call.arguments.get("path").and_then(|v| v.as_str()) {
                    Some(p) => p,
                    None => return ToolResult::error_text("Missing required parameter: path"),
                };
                let content = match call.arguments.get("content").and_then(|v| v.as_str()) {
                    Some(c) => c,
                    None => return ToolResult::error_text("Missing required parameter: content"),
                };
                self.write_file(path, content).await
            }
            "list_dir" => {
                let path = match call.arguments.get("path").and_then(|v| v.as_str()) {
                    Some(p) => p,
                    None => return ToolResult::error_text("Missing required parameter: path"),
                };
                self.list_dir(path).await
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
    fn test_file_io_tools() {
        let server = FileIOServer::new("fileio-1");
        let tools = server.tools();
        assert_eq!(tools.len(), 3);
    }

    #[tokio::test]
    async fn test_path_traversal_protection() {
        let temp_dir = tempfile::tempdir().unwrap();
        let server = FileIOServer::with_allowed_dirs(
            "fileio-1",
            vec![temp_dir.path().to_path_buf()],
        );

        // 尝试读取允许目录之外的文件
        let result = server.read_file("/etc/passwd").await;
        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn test_read_write_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.path().canonicalize().unwrap();
        let file_path = temp_path.join("test.txt");

        let server = FileIOServer::with_allowed_dirs(
            "fileio-1",
            vec![temp_path.clone()],
        );

        // 写入文件
        let result = server
            .write_file(file_path.to_str().unwrap(), "hello world")
            .await;
        assert!(result.is_error.is_none(), "Write failed: {:?}", result.content);

        // 读取文件
        let result = server.read_file(file_path.to_str().unwrap()).await;
        assert!(result.is_error.is_none(), "Read failed: {:?}", result.content);
        assert_eq!(result.content["content"], "hello world");
    }
}
