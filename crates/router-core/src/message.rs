use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use session_manager::{ServerId, SessionId};
use uuid::Uuid;

/// 处理类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingType {
    /// 有机处理 - 调用大型 LLM API
    Organic,
    /// 无机处理 - 确定性规则处理
    Inorganic,
}

impl std::fmt::Display for ProcessingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessingType::Organic => write!(f, "organic"),
            ProcessingType::Inorganic => write!(f, "inorganic"),
        }
    }
}

/// 消息内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum MessageContent {
    /// 结构化内容 (JSON)
    Structured {
        payload: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        schema_version: Option<String>,
    },
    /// 非结构化内容 (纯文本)
    Unstructured { text: String },
    /// 路由指令 - 明确的路由请求
    RoutingCommand {
        target_capability: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<serde_json::Value>,
    },
}

impl MessageContent {
    /// 创建结构化内容
    pub fn structured(payload: serde_json::Value) -> Self {
        MessageContent::Structured {
            payload,
            schema_version: None,
        }
    }

    /// 创建非结构化内容
    pub fn unstructured(text: impl Into<String>) -> Self {
        MessageContent::Unstructured { text: text.into() }
    }

    /// 创建路由指令
    pub fn routing_command(target_capability: impl Into<String>) -> Self {
        MessageContent::RoutingCommand {
            target_capability: target_capability.into(),
            payload: None,
        }
    }

    /// 是否为结构化内容
    pub fn is_structured(&self) -> bool {
        matches!(self, MessageContent::Structured { .. })
    }

    /// 是否为路由指令
    pub fn is_routing_command(&self) -> bool {
        matches!(self, MessageContent::RoutingCommand { .. })
    }

    /// 获取文本表示(用于判定和缓存)
    pub fn to_text(&self) -> String {
        match self {
            MessageContent::Structured { payload, .. } => payload.to_string(),
            MessageContent::Unstructured { text } => text.clone(),
            MessageContent::RoutingCommand {
                target_capability,
                payload,
            } => {
                let mut s = format!("route:{}", target_capability);
                if let Some(p) = payload {
                    s.push_str(&format!(" payload:{}", p));
                }
                s
            }
        }
    }

    /// 获取目标能力(仅路由指令)
    pub fn target_capability(&self) -> Option<&str> {
        match self {
            MessageContent::RoutingCommand {
                target_capability, ..
            } => Some(target_capability),
            _ => None,
        }
    }
}

/// 路由元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetadata {
    /// 已访问的 Server 列表
    pub visited_servers: Vec<ServerId>,
    /// 当前跳数
    pub hop_count: u32,
    /// 最大跳数
    pub max_hops: u32,
    /// 处理类型(判定后设置)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_type: Option<ProcessingType>,
}

impl RoutingMetadata {
    /// 创建新的路由元数据
    pub fn new(max_hops: u32) -> Self {
        Self {
            visited_servers: Vec::new(),
            hop_count: 0,
            max_hops,
            processing_type: None,
        }
    }

    /// 标记已访问 Server
    pub fn visit(&mut self, server_id: ServerId) {
        if !self.visited_servers.contains(&server_id) {
            self.visited_servers.push(server_id);
        }
        self.hop_count += 1;
    }

    /// 检查是否已访问过某个 Server
    pub fn has_visited(&self, server_id: &str) -> bool {
        self.visited_servers.iter().any(|s| s == server_id)
    }

    /// 检查是否超过最大跳数
    pub fn is_max_hops_exceeded(&self) -> bool {
        self.hop_count >= self.max_hops
    }
}

/// 路由消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息 ID
    pub message_id: Uuid,
    /// 所属 Session
    pub session_id: SessionId,
    /// 消息内容
    pub content: MessageContent,
    /// 路由元数据
    pub routing: RoutingMetadata,
    /// 消息时间戳
    pub timestamp: DateTime<Utc>,
}

impl Message {
    /// 创建新消息
    pub fn new(session_id: SessionId, content: MessageContent, max_hops: u32) -> Self {
        Self {
            message_id: Uuid::new_v4(),
            session_id,
            content,
            routing: RoutingMetadata::new(max_hops),
            timestamp: Utc::now(),
        }
    }

    /// 创建带默认配置的消息
    pub fn simple(session_id: SessionId, content: MessageContent) -> Self {
        Self::new(session_id, content, 10)
    }

    /// 设置处理类型
    pub fn with_processing_type(mut self, processing_type: ProcessingType) -> Self {
        self.routing.processing_type = Some(processing_type);
        self
    }

    /// 标记已访问 Server
    pub fn visit_server(&mut self, server_id: ServerId) {
        self.routing.visit(server_id);
    }

    /// 计算内容的 SHA-256 哈希(用于 Cache 匹配)
    pub fn content_hash(&self) -> String {
        session_manager::CacheManager::hash_content(&self.content.to_text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processing_type_display() {
        assert_eq!(ProcessingType::Organic.to_string(), "organic");
        assert_eq!(ProcessingType::Inorganic.to_string(), "inorganic");
    }

    #[test]
    fn test_message_content_structured() {
        let content = MessageContent::structured(serde_json::json!({"key": "value"}));
        assert!(content.is_structured());
        assert!(!content.is_routing_command());
    }

    #[test]
    fn test_message_content_routing_command() {
        let content = MessageContent::routing_command("code_review");
        assert!(content.is_routing_command());
        assert_eq!(content.target_capability(), Some("code_review"));
    }

    #[test]
    fn test_message_content_unstructured() {
        let content = MessageContent::unstructured("hello world");
        assert!(!content.is_structured());
        assert!(!content.is_routing_command());
        assert_eq!(content.to_text(), "hello world");
    }

    #[test]
    fn test_routing_metadata() {
        let mut meta = RoutingMetadata::new(10);
        assert!(meta.visited_servers.is_empty());
        assert_eq!(meta.hop_count, 0);
        assert!(!meta.is_max_hops_exceeded());

        meta.visit("server-a".to_string());
        assert!(meta.has_visited("server-a"));
        assert!(!meta.has_visited("server-b"));
        assert_eq!(meta.hop_count, 1);

        // 再次访问不会增加 hop_count，但已经包含所以不会重复
        meta.visit("server-a".to_string());
        assert_eq!(meta.hop_count, 2); // hop_count 总是递增
    }

    #[test]
    fn test_routing_metadata_max_hops() {
        let mut meta = RoutingMetadata::new(3);
        meta.visit("a".to_string());
        meta.visit("b".to_string());
        meta.visit("c".to_string());
        assert!(meta.is_max_hops_exceeded());
    }

    #[test]
    fn test_message_creation() {
        let session_id = Uuid::new_v4();
        let msg = Message::simple(session_id, MessageContent::unstructured("test"));

        assert_eq!(msg.session_id, session_id);
        assert!(msg.routing.visited_servers.is_empty());
        assert!(msg.routing.processing_type.is_none());
    }

    #[test]
    fn test_message_with_processing_type() {
        let session_id = Uuid::new_v4();
        let msg = Message::simple(session_id, MessageContent::unstructured("test"))
            .with_processing_type(ProcessingType::Organic);

        assert_eq!(msg.routing.processing_type, Some(ProcessingType::Organic));
    }

    #[test]
    fn test_message_content_hash() {
        let session_id = Uuid::new_v4();
        let msg1 = Message::simple(session_id, MessageContent::unstructured("hello"));
        let msg2 = Message::simple(session_id, MessageContent::unstructured("hello"));
        let msg3 = Message::simple(session_id, MessageContent::unstructured("world"));

        assert_eq!(msg1.content_hash(), msg2.content_hash());
        assert_ne!(msg1.content_hash(), msg3.content_hash());
    }

    #[test]
    fn test_message_serialization_roundtrip() {
        let session_id = Uuid::new_v4();
        let msg = Message::new(
            session_id,
            MessageContent::structured(serde_json::json!({"action": "review"})),
            5,
        );

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();

        assert_eq!(msg.message_id, deserialized.message_id);
        assert_eq!(msg.session_id, deserialized.session_id);
        assert_eq!(msg.routing.max_hops, deserialized.routing.max_hops);
    }
}
