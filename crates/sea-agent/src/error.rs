//! SEA Agent 错误类型

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SeaError {
    #[error("Session error: {0}")]
    Session(#[from] session_manager::ManagerError),

    #[error("Router error: {0}")]
    Router(#[from] router_core::RouterError),

    #[error("Server error: {0}")]
    Server(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type Result<T> = std::result::Result<T, SeaError>;
