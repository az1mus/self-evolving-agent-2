//! HTTP Client Server
//!
//! 发送 HTTP 请求的 Server

use async_trait::async_trait;
use mcp_server_framework::{MCPServer, Tool, ToolCall, ToolResult};
use serde_json::json;

/// HTTP Client Server
#[cfg(feature = "http")]
pub struct HttpClientServer {
    id: String,
    client: reqwest::Client,
}

#[cfg(feature = "http")]
impl HttpClientServer {
    /// 创建新的 HTTP Client Server
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            client: reqwest::Client::new(),
        }
    }

    /// HTTP GET 请求
    async fn http_get(&self, url: &str) -> ToolResult {
        match self.client.get(url).send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                match response.text().await {
                    Ok(body) => ToolResult::success(json!({
                        "status": status,
                        "body": body
                    })),
                    Err(e) => ToolResult::error_text(format!("Failed to read response body: {}", e)),
                }
            }
            Err(e) => ToolResult::error_text(format!("HTTP request failed: {}", e)),
        }
    }

    /// HTTP POST 请求
    async fn http_post(
        &self,
        url: &str,
        body: Option<serde_json::Value>,
        headers: Option<std::collections::HashMap<String, String>>,
    ) -> ToolResult {
        let mut request = self.client.post(url);

        if let Some(h) = headers {
            for (key, value) in h {
                request = request.header(&key, &value);
            }
        }

        if let Some(b) = body {
            request = request.json(&b);
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                match response.text().await {
                    Ok(resp_body) => ToolResult::success(json!({
                        "status": status,
                        "body": resp_body
                    })),
                    Err(e) => ToolResult::error_text(format!("Failed to read response body: {}", e)),
                }
            }
            Err(e) => ToolResult::error_text(format!("HTTP request failed: {}", e)),
        }
    }
}

#[cfg(feature = "http")]
#[async_trait]
impl MCPServer for HttpClientServer {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn tools(&self) -> Vec<Tool> {
        vec![
            Tool::with_schema(
                "http_get",
                "Perform HTTP GET request",
                json!({
                    "type": "object",
                    "properties": {
                        "url": {"type": "string", "description": "URL to fetch"}
                    },
                    "required": ["url"]
                }),
            ),
            Tool::with_schema(
                "http_post",
                "Perform HTTP POST request",
                json!({
                    "type": "object",
                    "properties": {
                        "url": {"type": "string", "description": "URL to post to"},
                        "body": {"description": "Request body (JSON)"},
                        "headers": {"type": "object", "description": "HTTP headers"}
                    },
                    "required": ["url"]
                }),
            ),
        ]
    }

    async fn handle_tool_call(&self, call: ToolCall) -> ToolResult {
        match call.tool_name.as_str() {
            "http_get" => {
                let url = match call.arguments.get("url").and_then(|v| v.as_str()) {
                    Some(u) => u,
                    None => return ToolResult::error_text("Missing required parameter: url"),
                };
                self.http_get(url).await
            }
            "http_post" => {
                let url = match call.arguments.get("url").and_then(|v| v.as_str()) {
                    Some(u) => u,
                    None => return ToolResult::error_text("Missing required parameter: url"),
                };
                let body = call.arguments.get("body").cloned();
                let headers = call
                    .arguments
                    .get("headers")
                    .and_then(|v| serde_json::from_value(v.clone()).ok());
                self.http_post(url, body, headers).await
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

// Non-HTTP feature stub
#[cfg(not(feature = "http"))]
pub struct HttpClientServer {
    id: String,
}

#[cfg(not(feature = "http"))]
impl HttpClientServer {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[cfg(not(feature = "http"))]
#[async_trait]
impl MCPServer for HttpClientServer {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn tools(&self) -> Vec<Tool> {
        vec![]
    }

    async fn handle_tool_call(&self, _call: ToolCall) -> ToolResult {
        ToolResult::error_text("HTTP feature not enabled")
    }

    async fn on_message(
        &self,
        _msg: mcp_server_framework::MCPMessage,
    ) -> Option<mcp_server_framework::MCPMessage> {
        None
    }
}

#[cfg(test)]
#[cfg(feature = "http")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_client_tools() {
        let server = HttpClientServer::new("http-1");
        let tools = server.tools();
        assert_eq!(tools.len(), 2);
    }
}
