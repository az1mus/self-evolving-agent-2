use serde::{Deserialize, Serialize};
use session_manager::ServerId;
use uuid::Uuid;

/// Server 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server ID
    pub server_id: ServerId,
    /// Session ID
    pub session_id: session_manager::SessionId,
    /// Gossip 配置
    pub gossip: GossipConfig,
    /// 失效检测配置
    pub failure_detection: FailureDetectionConfig,
}

/// Gossip 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipConfig {
    /// 心跳间隔 (秒)
    pub heartbeat_interval_secs: u64,
    /// 心跳超时 (秒)
    pub heartbeat_timeout_secs: u64,
    /// 拓扑同步间隔 (秒)
    pub topology_sync_interval_secs: u64,
}

impl Default for GossipConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_secs: 5,
            heartbeat_timeout_secs: 15,
            topology_sync_interval_secs: 30,
        }
    }
}

/// 失效检测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureDetectionConfig {
    /// 可疑阈值 (心跳超时次数)
    pub suspect_threshold: u32,
    /// 确认失效阈值
    pub confirm_threshold: u32,
}

impl Default for FailureDetectionConfig {
    fn default() -> Self {
        Self {
            suspect_threshold: 2,
            confirm_threshold: 4,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server_id: format!("server-{}", Uuid::new_v4()),
            session_id: session_manager::SessionId::new_v4(),
            gossip: GossipConfig::default(),
            failure_detection: FailureDetectionConfig::default(),
        }
    }
}

impl ServerConfig {
    /// 从 TOML 文件加载配置
    pub fn from_toml(path: &std::path::Path) -> Result<Self, ConfigParseError> {
        let content = std::fs::read_to_string(path)?;
        let config: ServerConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到 TOML 文件
    pub fn save_toml(&self, path: &std::path::Path) -> Result<(), ConfigParseError> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// 配置错误
#[derive(Debug, thiserror::Error)]
pub enum ConfigParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    Serialize(#[from] toml::ser::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.gossip.heartbeat_interval_secs, 5);
        assert_eq!(config.failure_detection.suspect_threshold, 2);
    }

    #[test]
    fn test_config_serialization() {
        let config = ServerConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();

        let decoded: ServerConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(
            decoded.gossip.heartbeat_interval_secs,
            config.gossip.heartbeat_interval_secs
        );
    }
}
