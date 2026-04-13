//! SEA Agent CLI
//!
//! 提供完整的命令行接口，整合所有模块功能

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use session_manager::SessionId;

use crate::config::{ClassifierType, SeaConfig};
use crate::error::Result;
use crate::runtime::SeaAgent;

/// SEA (Self-Evolving Agent) - 基于 MCP 的自演化智能代理系统
#[derive(Parser, Debug)]
#[command(name = "sea", version, about = "Self-Evolving Agent CLI")]
pub struct SeaCli {
    /// 配置文件路径
    #[arg(long, global = true, default_value = "")]
    config: String,

    /// Session 存储路径
    #[arg(long, global = true)]
    session_path: Option<PathBuf>,

    /// 日志级别
    #[arg(long, global = true, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 启动完整系统
    Run {
        /// 使用 Mock 分类器（用于测试）
        #[arg(long)]
        mock_classifier: bool,
    },

    /// Session 管理
    Session {
        #[command(subcommand)]
        action: SessionActions,
    },

    /// Server 管理
    Server {
        #[command(subcommand)]
        action: ServerActions,
    },

    /// 发送消息
    Message {
        #[command(subcommand)]
        action: MessageActions,
    },

    /// 生成默认配置文件
    Config {
        /// 输出路径
        #[arg(long, default_value = "config.toml")]
        output: PathBuf,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum SessionActions {
    /// 创建新 Session
    Create,

    /// 列出所有 Session
    List,

    /// 显示 Session 详情
    Show {
        /// Session ID
        session_id: String,
    },

    /// 删除 Session
    Delete {
        /// Session ID
        session_id: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum ServerActions {
    /// 注册 Server 到 Session
    Register {
        /// Session ID
        #[arg(long)]
        session: String,

        /// Server 类型
        server_type: String,

        /// 自定义 Server ID
        #[arg(long)]
        id: Option<String>,
    },

    /// 列出 Server
    List {
        /// 按 Session 过滤
        #[arg(long)]
        session: Option<String>,
    },

    /// 启动 Server
    Start {
        /// Server ID
        server_id: String,
    },

    /// 停止 Server
    Stop {
        /// Server ID
        server_id: String,
    },

    /// 列出可用的 Server 类型
    Types,
}

#[derive(Subcommand, Debug, Clone)]
enum MessageActions {
    /// 发送消息
    Send {
        /// Session ID
        #[arg(long)]
        session: String,

        /// 消息内容
        content: String,
    },

    /// 查看消息历史
    History {
        /// Session ID
        session: String,

        /// 限制数量
        #[arg(long)]
        limit: Option<usize>,

        /// 偏移量
        #[arg(long)]
        offset: Option<usize>,
    },
}

impl SeaCli {
    /// 执行 CLI 命令
    pub async fn execute(self) -> Result<()> {
        // 初始化日志
        self.init_tracing();

        // 加载配置
        let mut config = self.load_config()?;

        // 执行命令
        match self.command {
            Commands::Run { mock_classifier } => {
                if mock_classifier {
                    config.router.classifier_type = ClassifierType::Mock {
                        default_organic: true,
                    };
                }
                self.run_command(config).await
            }
            Commands::Session { ref action } => self.session_command(config, action.clone()).await,
            Commands::Server { ref action } => self.server_command(config, action.clone()).await,
            Commands::Message { ref action } => self.message_command(config, action.clone()).await,
            Commands::Config { ref output } => self.config_command(output.clone()),
        }
    }

    /// 初始化日志系统
    fn init_tracing(&self) {
        use tracing_subscriber::EnvFilter;

        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(&self.log_level));

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .init();
    }

    /// 加载配置
    fn load_config(&self) -> Result<SeaConfig> {
        // 命令行参数优先
        if let Some(path) = &self.session_path {
            let mut config = SeaConfig::default();
            config.session_store_path = path.clone();
            return Ok(config);
        }

        // 尝试从配置文件加载
        if !self.config.is_empty() {
            let path = PathBuf::from(&self.config);
            if path.exists() {
                return SeaConfig::from_toml(&path)
                    .map_err(|e| crate::error::SeaError::Config(e.to_string()));
            }
        }

        // 尝试默认配置路径
        let default_path = SeaConfig::default_config_path();
        if default_path.exists() {
            if let Ok(config) = SeaConfig::from_toml(&default_path) {
                return Ok(config);
            }
        }

        Ok(SeaConfig::default())
    }

    /// 执行 run 命令
    async fn run_command(&self, config: SeaConfig) -> Result<()> {
        println!("Starting SEA Agent...");
        let mut agent = SeaAgent::new(config).await?;

        let session_id = agent.init().await?;
        println!("SEA Agent is running with session: {}", session_id);
        println!();
        println!("Available servers:");
        for server in agent.list_session_servers(session_id) {
            println!("  - {} ({}) [{}]", server.id, server.server_type, if server.running { "running" } else { "stopped" });
        }
        println!();
        println!("Press Ctrl+C to shutdown...");

        // 等待 Ctrl+C
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| crate::error::SeaError::Io(e))?;

        println!("\nShutting down...");
        agent.shutdown().await?;
        println!("Goodbye!");
        Ok(())
    }

    /// 执行 session 命令
    async fn session_command(&self, config: SeaConfig, action: SessionActions) -> Result<()> {
        let agent = SeaAgent::new(config).await?;

        match action {
            SessionActions::Create => {
                let session_id = agent.create_session().await?;
                println!("Session created: {}", session_id);
            }
            SessionActions::List => {
                let sessions = agent.list_sessions().await?;
                if sessions.is_empty() {
                    println!("No sessions found.");
                } else {
                    println!("Sessions:");
                    for s in sessions {
                        println!(
                            "  {} | {:?} | servers: {} | messages: {}",
                            s.session_id, s.state, s.server_count, s.message_count
                        );
                    }
                }
            }
            SessionActions::Show { session_id } => {
                let id = parse_session_id(&session_id)?;
                let session = agent.show_session(id).await?;
                println!("Session: {}", session.session_id);
                println!("  State: {:?}", session.state);
                println!("  Created: {}", session.created_at);
                println!("  Updated: {}", session.updated_at);
                println!("  Servers: {}", session.servers.len());
                println!("  Messages: {}", session.message_history.len());
                println!("  Routing entries: {}", session.routing_table.len());

                if !session.servers.is_empty() {
                    println!("\n  Registered servers:");
                    for (id, info) in &session.servers {
                        println!(
                            "    - {} [{:?}] tools: {:?}",
                            id, info.status, info.tools
                        );
                    }
                }

                if !session.routing_table.is_empty() {
                    println!("\n  Routing table:");
                    for (cap, server_id) in session.routing_table.iter() {
                        println!("    {} -> {}", cap, server_id);
                    }
                }
            }
            SessionActions::Delete { session_id } => {
                let id = parse_session_id(&session_id)?;
                agent.delete_session(id).await?;
                println!("Session deleted: {}", id);
            }
        }

        Ok(())
    }

    /// 执行 server 命令
    async fn server_command(&self, config: SeaConfig, action: ServerActions) -> Result<()> {
        let mut agent = SeaAgent::new(config).await?;

        match action {
            ServerActions::Register {
                session,
                server_type,
                id,
            } => {
                let session_id = parse_session_id(&session)?;
                let st = parse_server_type_enum(&server_type)?;
                let server_id = agent
                    .register_server(session_id, st, id)
                    .await?;
                println!("Server registered: {}", server_id);
            }
            ServerActions::List { session } => {
                let servers = if let Some(session) = session {
                    let session_id = parse_session_id(&session)?;
                    agent.list_session_servers(session_id)
                } else {
                    agent.list_servers()
                };

                if servers.is_empty() {
                    println!("No servers found.");
                } else {
                    println!("Servers:");
                    for s in servers {
                        println!(
                            "  {} | {} | session: {} | [{}]",
                            s.id,
                            s.server_type,
                            s.session_id,
                            if s.running { "running" } else { "stopped" }
                        );
                    }
                }
            }
            ServerActions::Start { server_id } => {
                agent.start_server(&server_id).await?;
                println!("Server started: {}", server_id);
            }
            ServerActions::Stop { server_id } => {
                agent.stop_server(&server_id).await?;
                println!("Server stopped: {}", server_id);
            }
            ServerActions::Types => {
                println!("Available server types:");
                println!("  echo - Echo server");
                println!("  calculator - Calculator server");
                println!("  time - Time server");
                println!("  counter - Counter server");
                println!("  kvstore - Key-Value store server");
            }
        }

        Ok(())
    }

    /// 执行 message 命令
    async fn message_command(&self, config: SeaConfig, action: MessageActions) -> Result<()> {
        let agent = SeaAgent::new(config).await?;

        match action {
            MessageActions::Send { session, content } => {
                let session_id = parse_session_id(&session)?;
                let result = agent.send_message(session_id, &content).await?;
                println!("Processing: {:?}", result.processing_type);
                println!("Routed to: {}", result.routed_servers.join(", "));
                println!("Response: {}", result.response);
            }
            MessageActions::History {
                session,
                limit,
                offset,
            } => {
                let session_id = parse_session_id(&session)?;
                let messages = agent.get_message_history(session_id, limit, offset).await?;

                if messages.is_empty() {
                    println!("No messages found.");
                } else {
                    println!("Message history:");
                    for msg in messages {
                        println!(
                            "  [{:?}] {}: {}",
                            msg.role, msg.timestamp, msg.content
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// 执行 config 命令
    fn config_command(&self, output: PathBuf) -> Result<()> {
        let config = SeaConfig::default();
        config
            .save_toml(&output)
            .map_err(|e| crate::error::SeaError::Config(e.to_string()))?;
        println!("Default configuration saved to: {}", output.display());
        Ok(())
    }
}

/// 解析 Session ID
fn parse_session_id(s: &str) -> Result<SessionId> {
    uuid::Uuid::parse_str(s)
        .map_err(|_| crate::error::SeaError::InvalidOperation(format!("Invalid session ID: {}", s)))
}

/// 解析 Server 类型枚举
fn parse_server_type_enum(s: &str) -> Result<concrete_servers::factory::ServerType> {
    use std::str::FromStr;
    concrete_servers::factory::ServerType::from_str(s).map_err(|_| {
        crate::error::SeaError::InvalidOperation(format!(
            "Unknown server type: '{}'. Available: echo, calculator, time, counter, kvstore",
            s
        ))
    })
}
