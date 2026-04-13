//! Concrete Servers Implementation
//!
//! 这个 crate 提供了具体的 MCP Server 实现，包括：
//! - 基础示例 Servers (Echo, Calculator, Time)
//! - 状态管理 Servers (Counter, KVStore)
//! - 文本处理 Servers (TextAnalyzer, TextTransformer)
//! - 外部集成 Servers (HttpClient, FileIO, LLMGateway)
//! - 业务逻辑 Servers (CodeReview, TaskOrchestrator, DataPipeline)

pub mod servers;
pub mod factory;
pub mod registry;

pub use factory::ServerFactory;
pub use registry::ServerRegistry;
pub use servers::*;
