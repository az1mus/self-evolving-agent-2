use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// MCP 消息类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MCPMessageType {
    Request,
    Response,
    Notification,
}

/// MCP 消息 ID
pub type MessageId = String;

/// MCP 消息统一结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPMessage {
    /// 消息类型
    #[serde(rename = "type")]
    pub message_type: MCPMessageType,
    /// 消息负载
    #[serde(flatten)]
    pub payload: MCPMessagePayload,
    /// 元数据 (可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// MCP 消息负载 (Request / Response / Notification)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MCPMessagePayload {
    Request(MCPRequest),
    Response(MCPResponse),
    Notification(MCPNotification),
}

/// MCP 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPRequest {
    /// 请求 ID
    pub id: MessageId,
    /// 方法名
    pub method: String,
    /// 参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// MCP 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResponse {
    /// 对应的请求 ID
    pub id: MessageId,
    /// 结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// 错误
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<MCPError>,
}

/// MCP 错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPError {
    /// 错误码
    pub code: i32,
    /// 错误消息
    pub message: String,
    /// 额外数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// MCP 通知 (单向消息)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPNotification {
    /// 方法名
    pub method: String,
    /// 参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

// ============================================================================
// 实现辅助方法
// ============================================================================

impl MCPMessage {
    /// 创建请求消息
    pub fn request(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            message_type: MCPMessageType::Request,
            payload: MCPMessagePayload::Request(MCPRequest {
                id,
                method: method.into(),
                params,
            }),
            metadata: None,
        }
    }

    /// 创建响应消息 (成功)
    pub fn response(id: MessageId, result: serde_json::Value) -> Self {
        Self {
            message_type: MCPMessageType::Response,
            payload: MCPMessagePayload::Response(MCPResponse {
                id,
                result: Some(result),
                error: None,
            }),
            metadata: None,
        }
    }

    /// 创建响应消息 (错误)
    pub fn error_response(id: MessageId, code: i32, message: impl Into<String>) -> Self {
        Self {
            message_type: MCPMessageType::Response,
            payload: MCPMessagePayload::Response(MCPResponse {
                id,
                result: None,
                error: Some(MCPError {
                    code,
                    message: message.into(),
                    data: None,
                }),
            }),
            metadata: None,
        }
    }

    /// 创建通知消息
    pub fn notification(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            message_type: MCPMessageType::Notification,
            payload: MCPMessagePayload::Notification(MCPNotification {
                method: method.into(),
                params,
            }),
            metadata: None,
        }
    }

    /// 获取请求 ID (如果是请求或响应)
    pub fn id(&self) -> Option<&MessageId> {
        match &self.payload {
            MCPMessagePayload::Request(req) => Some(&req.id),
            MCPMessagePayload::Response(resp) => Some(&resp.id),
            MCPMessagePayload::Notification(_) => None,
        }
    }

    /// 获取方法名
    pub fn method(&self) -> Option<&str> {
        match &self.payload {
            MCPMessagePayload::Request(req) => Some(&req.method),
            MCPMessagePayload::Notification(notif) => Some(&notif.method),
            MCPMessagePayload::Response(_) => None,
        }
    }
}

impl MCPError {
    /// 创建错误
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// 创建错误 (带额外数据)
    pub fn with_data(code: i32, message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
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
    fn test_request_message() {
        let msg = MCPMessage::request("test_method", Some(json!({"key": "value"})));

        assert_eq!(msg.message_type, MCPMessageType::Request);
        assert!(msg.id().is_some());
        assert_eq!(msg.method(), Some("test_method"));
    }

    #[test]
    fn test_response_message_success() {
        let id = Uuid::new_v4().to_string();
        let msg = MCPMessage::response(id.clone(), json!({"result": "ok"}));

        assert_eq!(msg.message_type, MCPMessageType::Response);
        assert_eq!(msg.id(), Some(&id));
    }

    #[test]
    fn test_response_message_error() {
        let id = Uuid::new_v4().to_string();
        let msg = MCPMessage::error_response(id.clone(), -32600, "Invalid Request");

        assert_eq!(msg.message_type, MCPMessageType::Response);
        assert_eq!(msg.id(), Some(&id));

        if let MCPMessagePayload::Response(resp) = &msg.payload {
            assert!(resp.error.is_some());
            let error = resp.error.as_ref().unwrap();
            assert_eq!(error.code, -32600);
            assert_eq!(error.message, "Invalid Request");
        } else {
            panic!("Expected Response payload");
        }
    }

    #[test]
    fn test_notification_message() {
        let msg = MCPMessage::notification("update", Some(json!({"status": "changed"})));

        assert_eq!(msg.message_type, MCPMessageType::Notification);
        assert!(msg.id().is_none());
        assert_eq!(msg.method(), Some("update"));
    }

    #[test]
    fn test_message_serialization() {
        let msg = MCPMessage::request("test", Some(json!({"param": 123})));
        let json_str = serde_json::to_string(&msg).unwrap();

        // 应该包含所有字段
        assert!(json_str.contains("\"type\":\"request\""));
        assert!(json_str.contains("\"method\":\"test\""));

        // 反序列化
        let decoded: MCPMessage = serde_json::from_str(&json_str).unwrap();
        assert_eq!(decoded.message_type, MCPMessageType::Request);
        assert_eq!(decoded.method(), Some("test"));
    }
}
