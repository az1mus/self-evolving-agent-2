//! Server Registry
//!
//! 维护所有可用的 Server 类型，支持动态注册

use crate::factory::ServerConfig;
use anyhow::{anyhow, Result};
use mcp_server_framework::MCPServer;
use session_manager::SessionId;
use std::collections::HashMap;
use std::sync::Arc;

/// Server 创建函数类型
type ServerCreator = Box<dyn Fn(ServerConfig, SessionId) -> Result<Arc<dyn MCPServer>> + Send + Sync>;

/// Server 注册表
pub struct ServerRegistry {
    /// 已注册的 Server 类型和创建函数
    creators: HashMap<String, ServerCreator>,
    /// 已注册类型的描述
    descriptions: HashMap<String, String>,
}

impl ServerRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        let mut registry = Self {
            creators: HashMap::new(),
            descriptions: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    /// 注册默认的 Server 类型
    fn register_defaults(&mut self) {
        // Echo
        self.register("echo", "Echo Server - returns input text", |_config, _session_id| {
            Ok(Arc::new(crate::servers::EchoServer::new(
                _config.id.unwrap_or_else(|| format!("echo-{}", uuid::Uuid::new_v4())),
            )))
        }).ok();

        // Calculator
        self.register("calculator", "Calculator Server - basic math operations", |_config, _session_id| {
            Ok(Arc::new(crate::servers::CalculatorServer::new(
                _config.id.unwrap_or_else(|| format!("calculator-{}", uuid::Uuid::new_v4())),
            )))
        }).ok();

        // Time
        self.register("time", "Time Server - current time and formatting", |_config, _session_id| {
            Ok(Arc::new(crate::servers::TimeServer::new(
                _config.id.unwrap_or_else(|| format!("time-{}", uuid::Uuid::new_v4())),
            )))
        }).ok();

        // Counter
        self.register("counter", "Counter Server - maintains named counters", |_config, _session_id| {
            Ok(Arc::new(crate::servers::CounterServer::new(
                _config.id.unwrap_or_else(|| format!("counter-{}", uuid::Uuid::new_v4())),
            )))
        }).ok();

        // KVStore
        self.register("kvstore", "KVStore Server - key-value store with TTL", |_config, _session_id| {
            Ok(Arc::new(crate::servers::KVStoreServer::new(
                _config.id.unwrap_or_else(|| format!("kvstore-{}", uuid::Uuid::new_v4())),
            )))
        }).ok();
    }

    /// 注册新的 Server 类型
    pub fn register(
        &mut self,
        type_name: &str,
        description: &str,
        creator: impl Fn(ServerConfig, SessionId) -> Result<Arc<dyn MCPServer>> + Send + Sync + 'static,
    ) -> Result<()> {
        if self.creators.contains_key(type_name) {
            return Err(anyhow!("Server type '{}' is already registered", type_name));
        }
        self.creators.insert(type_name.to_string(), Box::new(creator));
        self.descriptions.insert(type_name.to_string(), description.to_string());
        Ok(())
    }

    /// 创建指定类型的 Server
    pub fn create(
        &self,
        type_name: &str,
        config: ServerConfig,
        session_id: SessionId,
    ) -> Result<Arc<dyn MCPServer>> {
        let creator = self
            .creators
            .get(type_name)
            .ok_or_else(|| anyhow!("Unknown server type: '{}'. Available: {}", type_name, self.available_types().join(", ")))?;
        creator(config, session_id)
    }

    /// 获取所有可用的 Server 类型
    pub fn available_types(&self) -> Vec<String> {
        let mut types: Vec<String> = self.creators.keys().cloned().collect();
        types.sort();
        types
    }

    /// 获取 Server 类型描述
    pub fn get_description(&self, type_name: &str) -> Option<&str> {
        self.descriptions.get(type_name).map(|s| s.as_str())
    }

    /// 检查类型是否已注册
    pub fn is_registered(&self, type_name: &str) -> bool {
        self.creators.contains_key(type_name)
    }
}

impl Default for ServerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::ServerType;

    #[test]
    fn test_registry_defaults() {
        let registry = ServerRegistry::new();
        assert!(registry.is_registered("echo"));
        assert!(registry.is_registered("calculator"));
        assert!(registry.is_registered("time"));
    }

    #[test]
    fn test_available_types() {
        let registry = ServerRegistry::new();
        let types = registry.available_types();
        assert!(types.contains(&"echo".to_string()));
        assert!(types.contains(&"calculator".to_string()));
        assert!(types.contains(&"time".to_string()));
    }

    #[test]
    fn test_create_from_registry() {
        let registry = ServerRegistry::new();
        let session_id = SessionId::new_v4();
        let config = ServerConfig::new(ServerType::Echo).with_id("test-echo");

        let server = registry.create("echo", config, session_id).unwrap();
        assert_eq!(server.id(), "test-echo");
    }

    #[test]
    fn test_create_unknown_type() {
        let registry = ServerRegistry::new();
        let session_id = SessionId::new_v4();
        let config = ServerConfig::new(ServerType::Echo);

        let result = registry.create("unknown", config, session_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_custom_server() {
        let mut registry = ServerRegistry::new();

        registry
            .register(
                "custom",
                "Custom test server",
                |_config, _session_id| {
                    Ok(Arc::new(crate::servers::EchoServer::new(
                        _config.id.unwrap_or_else(|| "custom-1".to_string()),
                    )))
                },
            )
            .unwrap();

        assert!(registry.is_registered("custom"));
        assert_eq!(registry.get_description("custom"), Some("Custom test server"));
    }

    #[test]
    fn test_duplicate_registration() {
        let mut registry = ServerRegistry::new();
        let result = registry.register("echo", "duplicate", |_config, _session_id| {
            Ok(Arc::new(crate::servers::EchoServer::new("dup")))
        });
        assert!(result.is_err());
    }
}
