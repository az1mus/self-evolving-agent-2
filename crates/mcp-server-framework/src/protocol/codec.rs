use super::message::MCPMessage;
use thiserror::Error;

/// MCP 编解码错误
#[derive(Debug, Error)]
pub enum MCPCodecError {
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid message format")]
    InvalidFormat,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// MCP 编解码器
///
/// 提供消息的序列化和反序列化功能
pub struct MCPCodec;

impl MCPCodec {
    /// 编码消息为 JSON 字节
    pub fn encode(message: &MCPMessage) -> Result<Vec<u8>, MCPCodecError> {
        let json_str = serde_json::to_string(message)?;
        Ok(json_str.into_bytes())
    }

    /// 从 JSON 字节解码消息
    pub fn decode(data: &[u8]) -> Result<MCPMessage, MCPCodecError> {
        let json_str = std::str::from_utf8(data).map_err(|_| MCPCodecError::InvalidFormat)?;
        let message: MCPMessage = serde_json::from_str(json_str)?;
        Ok(message)
    }

    /// 编码消息为 JSON 字符串
    pub fn encode_to_string(message: &MCPMessage) -> Result<String, MCPCodecError> {
        Ok(serde_json::to_string(message)?)
    }

    /// 从 JSON 字符串解码消息
    pub fn decode_from_str(json_str: &str) -> Result<MCPMessage, MCPCodecError> {
        Ok(serde_json::from_str(json_str)?)
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
    fn test_encode_decode_request() {
        let msg = MCPMessage::request("test_method", Some(json!({"key": "value"})));

        // 编码
        let encoded = MCPCodec::encode(&msg).unwrap();
        assert!(!encoded.is_empty());

        // 解码
        let decoded = MCPCodec::decode(&encoded).unwrap();
        assert_eq!(decoded.message_type, msg.message_type);
        assert_eq!(decoded.method(), msg.method());
    }

    #[test]
    fn test_encode_decode_response() {
        let id = uuid::Uuid::new_v4().to_string();
        let msg = MCPMessage::response(id.clone(), json!({"result": "ok"}));

        // 编码
        let encoded = MCPCodec::encode(&msg).unwrap();

        // 解码
        let decoded = MCPCodec::decode(&encoded).unwrap();
        assert_eq!(decoded.id(), Some(&id));
    }

    #[test]
    fn test_encode_decode_notification() {
        let msg = MCPMessage::notification("update", Some(json!({"status": "changed"})));

        // 编码
        let encoded = MCPCodec::encode(&msg).unwrap();

        // 解码
        let decoded = MCPCodec::decode(&encoded).unwrap();
        assert_eq!(decoded.method(), Some("update"));
    }

    #[test]
    fn test_encode_decode_string() {
        let msg = MCPMessage::request("test", None);

        // 编码为字符串
        let json_str = MCPCodec::encode_to_string(&msg).unwrap();
        assert!(json_str.contains("\"method\":\"test\""));

        // 从字符串解码
        let decoded = MCPCodec::decode_from_str(&json_str).unwrap();
        assert_eq!(decoded.method(), Some("test"));
    }

    #[test]
    fn test_roundtrip_preserves_message() {
        let original = MCPMessage::request(
            "complex_method",
            Some(json!({
                "nested": {
                    "data": [1, 2, 3],
                    "text": "hello"
                }
            })),
        );

        let encoded = MCPCodec::encode(&original).unwrap();
        let decoded = MCPCodec::decode(&encoded).unwrap();

        // 验证关键字段
        assert_eq!(decoded.message_type, original.message_type);
        assert_eq!(decoded.method(), original.method());
        assert_eq!(decoded.id(), original.id());
    }

    #[test]
    fn test_invalid_decode() {
        let invalid_data = b"not valid json";
        let result = MCPCodec::decode(invalid_data);
        assert!(result.is_err());
    }
}
