use crate::message::ProcessingType;
use session_manager::ServerId;
use thiserror::Error;

/// Router Core 错误类型
#[derive(Debug, Error)]
pub enum RouterError {
    #[error("Classification failed: {0}")]
    ClassificationFailed(String),

    #[error("Preprocessing failed: {0}")]
    PreprocessingFailed(String),

    #[error("No capable server found for capability: {0}")]
    NoCapableServer(String),

    #[error("Routing failed: {0}")]
    RoutingFailed(String),

    #[error("Cycle detected: message already visited server {0}")]
    CycleDetected(ServerId),

    #[error("Max hops exceeded: hop_count={0}, max_hops={1}")]
    MaxHopsExceeded(u32, u32),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Session error: {0}")]
    SessionError(#[from] session_manager::ManagerError),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Processing failed for type {0:?}: {1}")]
    ProcessingFailed(ProcessingType, String),
}
