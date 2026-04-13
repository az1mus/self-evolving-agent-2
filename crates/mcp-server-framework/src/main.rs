use clap::Parser;
use mcp_server_framework::{
    MCPMessage, MCPServer, ServerConfig, ServerRunner, Tool, ToolCall, ToolResult,
};
use session_manager::ServerId;
use uuid::Uuid;

mod cli;

use cli::{Cli, Commands};

/// 示例 Echo Server
struct EchoServer {
    id: ServerId,
}

impl EchoServer {
    fn new(id: ServerId) -> Self {
        Self { id }
    }
}

#[async_trait::async_trait]
impl MCPServer for EchoServer {
    fn id(&self) -> ServerId {
        self.id.clone()
    }

    fn tools(&self) -> Vec<Tool> {
        vec![Tool::new("echo", "Echo the input message")]
    }

    async fn handle_tool_call(&self, call: ToolCall) -> ToolResult {
        ToolResult::text(format!("Echo: {}", call.arguments))
    }

    async fn on_message(&self, _msg: MCPMessage) -> Option<MCPMessage> {
        None
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            config,
            server_id,
            session_id,
        } => {
            let server_config = if let Some(config_path) = config {
                ServerConfig::from_toml(&std::path::PathBuf::from(config_path))?
            } else {
                let mut cfg = ServerConfig::default();
                if let Some(id) = server_id {
                    cfg.server_id = id;
                }
                if let Some(sid) = session_id {
                    cfg.session_id = Uuid::parse_str(&sid)?;
                }
                cfg
            };

            tracing::info!(
                "Starting server {} in session {}",
                server_config.server_id,
                server_config.session_id
            );

            let server = EchoServer::new(server_config.server_id.clone());
            let mut runner = ServerRunner::new(server, server_config);

            runner.start().await?;

            // 等待中断信号
            tokio::signal::ctrl_c().await?;
            tracing::info!("Received shutdown signal");

            runner.stop().await?;
        }
        Commands::Join { session, .. } => {
            tracing::info!("Joining session: {}", session);
            // TODO: 实现加入 Session 逻辑
        }
        Commands::Leave { server_id } => {
            tracing::info!("Leaving: {}", server_id);
            // TODO: 实现离开逻辑
        }
        Commands::Status { server_id } => {
            tracing::info!("Server status: {}", server_id);
            // TODO: 实现状态查询逻辑
        }
    }

    Ok(())
}
