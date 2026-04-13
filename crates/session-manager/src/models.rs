use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Session 唯一标识
pub type SessionId = Uuid;

/// Server 唯一标识
pub type ServerId = String;

/// Session 状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Active,
    Paused,
    Terminated,
}

/// Server 状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerStatus {
    Pending,
    Active,
    Draining,
    Removed,
}

/// Server 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub id: ServerId,
    pub status: ServerStatus,
    pub tools: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draining_since: Option<DateTime<Utc>>,
}

/// 消息角色
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Server,
}

/// 消息记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRecord {
    pub id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Cache 存储
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStore {
    pub input_cache: HashMap<String, serde_json::Value>,
    pub inference_cache: HashMap<String, serde_json::Value>,
}

/// Session 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    #[serde(default = "default_max_hops")]
    pub max_hops: u32,
    #[serde(default = "default_drain_timeout")]
    pub drain_timeout: u64,
}

fn default_max_hops() -> u32 {
    10
}

fn default_drain_timeout() -> u64 {
    300
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_hops: default_max_hops(),
            drain_timeout: default_drain_timeout(),
        }
    }
}

/// Session 聚合根
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: SessionId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub state: SessionState,
    pub servers: HashMap<ServerId, ServerInfo>,
    pub routing_table: HashMap<String, ServerId>,
    pub message_history: Vec<MessageRecord>,
    pub cache: CacheStore,
    pub config: SessionConfig,
}

impl Session {
    /// 创建新 Session
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            session_id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            state: SessionState::Active,
            servers: HashMap::new(),
            routing_table: HashMap::new(),
            message_history: Vec::new(),
            cache: CacheStore::default(),
            config: SessionConfig::default(),
        }
    }

    /// 标记已更新
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Session 摘要（列表展示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: SessionId,
    pub created_at: DateTime<Utc>,
    pub state: SessionState,
    pub server_count: usize,
    pub message_count: usize,
}

impl From<&Session> for SessionSummary {
    fn from(session: &Session) -> Self {
        Self {
            session_id: session.session_id,
            created_at: session.created_at,
            state: session.state.clone(),
            server_count: session.servers.len(),
            message_count: session.message_history.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new();
        assert_eq!(session.state, SessionState::Active);
        assert!(session.servers.is_empty());
        assert!(session.message_history.is_empty());
        assert!(session.routing_table.is_empty());
    }

    #[test]
    fn test_session_default() {
        let session = Session::default();
        assert_eq!(session.state, SessionState::Active);
    }

    #[test]
    fn test_session_config_defaults() {
        let config = SessionConfig::default();
        assert_eq!(config.max_hops, 10);
        assert_eq!(config.drain_timeout, 300);
    }

    #[test]
    fn test_session_serialization_roundtrip() {
        let mut session = Session::new();
        session.servers.insert(
            "server-a".to_string(),
            ServerInfo {
                id: "server-a".to_string(),
                status: ServerStatus::Active,
                tools: vec!["tool1".to_string(), "tool2".to_string()],
                metadata: HashMap::new(),
                draining_since: None,
            },
        );
        session.message_history.push(MessageRecord {
            id: Uuid::new_v4(),
            role: MessageRole::User,
            content: "hello".to_string(),
            timestamp: Utc::now(),
            metadata: None,
        });

        let json = serde_json::to_string_pretty(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(session.session_id, deserialized.session_id);
        assert_eq!(session.state, deserialized.state);
        assert_eq!(session.servers.len(), deserialized.servers.len());
        assert_eq!(
            session.message_history.len(),
            deserialized.message_history.len()
        );
    }

    #[test]
    fn test_server_status_serialization() {
        let status = ServerStatus::Draining;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("draining"));

        let deserialized: ServerStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_session_summary_from_session() {
        let mut session = Session::new();
        session.servers.insert(
            "s1".to_string(),
            ServerInfo {
                id: "s1".to_string(),
                status: ServerStatus::Active,
                tools: vec![],
                metadata: HashMap::new(),
                draining_since: None,
            },
        );

        let summary = SessionSummary::from(&session);
        assert_eq!(summary.server_count, 1);
        assert_eq!(summary.message_count, 0);
    }
}
