//! SEA (Self-Evolving Agent) 主程序
//!
//! 这是系统的集成层，整合了所有模块：
//! - session-manager: Session 容器与生命周期管理
//! - router-core: 消息路由与有机/无机判定
//! - mcp-server-framework: MCP Server 基础框架
//! - concrete-servers: 具体业务 Server 实现

pub mod cli;
pub mod config;
pub mod error;
pub mod runtime;

pub use cli::SeaCli;
pub use config::SeaConfig;
pub use error::SeaError;
pub use runtime::SeaAgent;
