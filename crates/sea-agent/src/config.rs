//! SEA Agent 配置管理

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// SEA Agent 主配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeaConfig {
    /// Session 存储路径
    pub session_store_path: PathBuf,

    /// Router 配置
    pub router: RouterConfig,

    /// Server 默认配置
    pub server_defaults: ServerDefaults,
}

/// Router 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// 最大跳数
    pub max_hops: u32,

    /// Drain 超时（秒）
    pub drain_timeout: u64,

    /// 分类器类型
    pub classifier_type: ClassifierType,
}

/// 分类器类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClassifierType {
    /// 规则基础分类器
    RuleBased,
    /// Mock 分类器（用于测试）
    Mock { default_organic: bool },
}

/// Server 默认配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerDefaults {
    /// Gossip 心跳间隔（秒）
    pub heartbeat_interval_secs: u64,

    /// Gossip 心跳超时（秒）
    pub heartbeat_timeout_secs: u64,

    /// 失效检测阈值
    pub failure_suspect_threshold: u32,
}

impl Default for SeaConfig {
    fn default() -> Self {
        Self {
            session_store_path: PathBuf::from("./sessions"),
            router: RouterConfig {
                max_hops: 10,
                drain_timeout: 300,
                classifier_type: ClassifierType::RuleBased,
            },
            server_defaults: ServerDefaults {
                heartbeat_interval_secs: 5,
                heartbeat_timeout_secs: 30,
                failure_suspect_threshold: 3,
            },
        }
    }
}

impl SeaConfig {
    /// 从 TOML 文件加载配置
    pub fn from_toml(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: SeaConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// 保存到 TOML 文件
    pub fn save_toml(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 获取默认配置路径
    pub fn default_config_path() -> PathBuf {
        directories::ProjectDirs::from("com", "sea", "sea-agent")
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("./config.toml"))
    }
}
