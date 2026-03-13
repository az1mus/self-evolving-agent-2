//! Self-Evolving Agent
//!
//! 一个能够自我进化的智能代理系统，支持工具进化与提示词进化。
//!
//! 架构：
//! - AgentDefinition: Agent 定义/模板，独立存储，可复用
//! - Session: 运行时容器，包含消息历史
//! - AgentInstance: Agent 实例，在 Session 中运行
//!
//! 多层提示词结构：
//! - Session概要: 默认为空，对话过程中由 Session 调用总结器总结（以后实现）
//! - Agent设定: 在 Agent Definition 中设置
//! - 对话示例: 在 Agent Definition 中设置
//! - Session Summary: 自动总结（以后实现）
//! - 上下文: 涉及到此 Agent 的真实对话上下文，是否包含在 Agent 中定义
//! - 指令: 传统意义上的系统提示词，在 Agent Definition 中设置
//! - 全局记忆: 由 Gateway 维护的全局信息

pub mod agent;
pub mod cli;
pub mod config;
pub mod gateway;
pub mod llm;
pub mod logger;
pub mod prompt;
pub mod session;

pub use agent::{
    AgentDefinition, AgentDefinitionId, AgentDefinitionInfo,
    AgentInstance, AgentInstanceId, AgentInstanceInfo,
    AgentInput, AgentOutput,
};
pub use cli::CliApp;
pub use config::ConfigManager;
pub use gateway::{Gateway, GatewayEvent};
pub use llm::LlmClient;
pub use logger::{LogLevel, Logger};
pub use prompt::{
    PromptBuilder, PromptContext, PromptTemplate, StructuredPrompt,
};
pub use session::{Session, SessionId, SessionInfo, SessionManager};
