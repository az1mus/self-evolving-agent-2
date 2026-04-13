//! Server Factory
//!
//! 用于创建各种 Server 实例

use crate::servers::{CalculatorServer, CounterServer, EchoServer, KVStoreServer, TimeServer};
use anyhow::{anyhow, Result};
use mcp_server_framework::MCPServer;
use session_manager::SessionId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Server 类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerType {
    Echo,
    Calculator,
    Time,
    Counter,
    KVStore,
    // Phase 3+ 会添加更多类型
    // TextAnalyzer,
    // TextTransformer,
    // HttpClient,
    // FileIO,
    // LLMGateway,
    // CodeReview,
    // TaskOrchestrator,
    // DataPipeline,
}

impl std::fmt::Display for ServerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerType::Echo => write!(f, "echo"),
            ServerType::Calculator => write!(f, "calculator"),
            ServerType::Time => write!(f, "time"),
            ServerType::Counter => write!(f, "counter"),
            ServerType::KVStore => write!(f, "kvstore"),
        }
    }
}

impl std::str::FromStr for ServerType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "echo" => Ok(ServerType::Echo),
            "calculator" => Ok(ServerType::Calculator),
            "time" => Ok(ServerType::Time),
            "counter" => Ok(ServerType::Counter),
            "kvstore" => Ok(ServerType::KVStore),
            _ => Err(anyhow!("Unknown server type: {}", s)),
        }
    }
}

/// Server 配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server 类型
    pub server_type: ServerType,
    /// Server ID (可选，不提供则自动生成)
    pub id: Option<String>,
    /// 额外配置参数
    pub params: HashMap<String, serde_json::Value>,
}

impl ServerConfig {
    /// 创建新的 Server 配置
    pub fn new(server_type: ServerType) -> Self {
        Self {
            server_type,
            id: None,
            params: HashMap::new(),
        }
    }

    /// 设置 Server ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 添加配置参数
    pub fn with_param(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.params.insert(key.into(), value);
        self
    }
}

/// Server 工厂
pub struct ServerFactory;

impl ServerFactory {
    /// 创建 Server 实例
    pub fn create(
        config: ServerConfig,
        _session_id: SessionId,
    ) -> Result<Arc<dyn MCPServer>> {
        let id = config.id.unwrap_or_else(|| {
            format!("{}-{}", config.server_type, uuid::Uuid::new_v4())
        });

        let server: Arc<dyn MCPServer> = match config.server_type {
            ServerType::Echo => Arc::new(EchoServer::new(id)),
            ServerType::Calculator => Arc::new(CalculatorServer::new(id)),
            ServerType::Time => Arc::new(TimeServer::new(id)),
            ServerType::Counter => Arc::new(CounterServer::new(id)),
            ServerType::KVStore => Arc::new(KVStoreServer::new(id)),
        };

        Ok(server)
    }

    /// 创建带默认配置的 Server
    pub fn create_default(
        server_type: ServerType,
        session_id: SessionId,
    ) -> Result<Arc<dyn MCPServer>> {
        Self::create(ServerConfig::new(server_type), session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_server_type_from_str() {
        assert!(matches!(ServerType::from_str("echo"), Ok(ServerType::Echo)));
        assert!(matches!(
            ServerType::from_str("calculator"),
            Ok(ServerType::Calculator)
        ));
        assert!(matches!(ServerType::from_str("time"), Ok(ServerType::Time)));
        assert!(ServerType::from_str("unknown").is_err());
    }

    #[test]
    fn test_server_config() {
        let config = ServerConfig::new(ServerType::Echo)
            .with_id("my-echo")
            .with_param("key", serde_json::json!("value"));

        assert_eq!(config.id, Some("my-echo".to_string()));
        assert!(config.params.contains_key("key"));
    }

    #[test]
    fn test_create_echo_server() {
        let session_id = SessionId::new_v4();
        let config = ServerConfig::new(ServerType::Echo).with_id("echo");
        let server = ServerFactory::create(config, session_id).unwrap();
        assert_eq!(server.id(), "echo");
    }

    #[test]
    fn test_create_calculator_server() {
        let session_id = SessionId::new_v4();
        let config = ServerConfig::new(ServerType::Calculator).with_id("calc-1");
        let server = ServerFactory::create(config, session_id).unwrap();
        assert_eq!(server.id(), "calc-1");
    }
}
