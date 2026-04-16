//! SEA Agent CLI 模块
//!
//! 提供统一的命令行接口，支持参数驱动和交互式两种模式

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use session_manager::SessionId;

use crate::config::{ClassifierType, SeaConfig};
use crate::error::Result;
use crate::runtime::SeaAgent;

pub mod error_formatter;
pub mod help;
pub mod interactive;
pub mod output;
pub mod theme;
pub mod wizard;

use help::HelpFormatter;
use interactive::ReplSession;
use output::OutputFormatter;
use theme::ThemeType;
use wizard::FirstRunWizard;

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

    /// 主题类型
    #[arg(long, global = true, default_value = "default")]
    theme: String,

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

        /// 启动后进入交互模式
        #[arg(long)]
        interactive: bool,
    },

    /// 启动交互式 REPL
    Repl {
        /// 自动加载的 Session ID
        #[arg(long)]
        session: Option<String>,
    },

    /// 快速对话模式
    Chat {
        /// 使用默认配置快速启动
        #[arg(long)]
        quick: bool,
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

    /// 显示系统状态
    Status {
        /// 持续监控模式
        #[arg(long)]
        watch: bool,
    },

    /// 显示详细帮助信息
    Guide {
        /// 帮助主题
        #[arg(long)]
        topic: Option<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum SessionActions {
    /// 创建新 Session
    Create {
        /// Session 名称
        #[arg(long)]
        name: Option<String>,
    },

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

        /// Server 名称
        #[arg(long)]
        name: Option<String>,
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

        // 初始化输出格式化器
        let theme_type = self.parse_theme_type();
        let formatter = OutputFormatter::new(theme_type);

        // 检查是否需要首次运行向导
        if FirstRunWizard::should_run() && !matches!(self.command, Commands::Config { .. }) {
            let wizard = FirstRunWizard::new(formatter.clone());
            let _ = wizard.run().await?;
        }

        // 加载配置
        let mut config = self.load_config()?;

        // 执行命令
        match self.command {
            Commands::Run { mock_classifier, interactive: _ } => {
                if mock_classifier {
                    config.router.classifier_type = ClassifierType::Mock {
                        default_organic: true,
                    };
                }
                self.run_command(config, &formatter).await
            }
            Commands::Repl { ref session } => self.repl_command(config, session.clone(), &formatter).await,
            Commands::Chat { quick } => self.chat_command(config, quick, &formatter).await,
            Commands::Session { ref action } => {
                self.session_command(config, action.clone(), &formatter).await
            }
            Commands::Server { ref action } => {
                self.server_command(config, action.clone(), &formatter).await
            }
            Commands::Message { ref action } => {
                self.message_command(config, action.clone(), &formatter).await
            }
            Commands::Config { ref output } => self.config_command(output.clone(), &formatter),
            Commands::Status { watch } => self.status_command(config, watch, &formatter).await,
            Commands::Guide { ref topic } => self.help_command(topic.as_ref()),
        }
    }

    /// 解析主题类型
    fn parse_theme_type(&self) -> ThemeType {
        match self.theme.as_str() {
            "dark" => ThemeType::Dark,
            "monochrome" => ThemeType::Monochrome,
            _ => ThemeType::Default,
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
    async fn run_command(&self, config: SeaConfig, formatter: &OutputFormatter) -> Result<()> {
        formatter.print_info("Starting SEA Agent...");
        let mut agent = SeaAgent::new(config).await?;

        let session_id = agent.init().await?;
        formatter.print_success(&format!("SEA Agent is running with session: {}", session_id));

        println!();
        println!("Available servers:");
        for server in agent.list_session_servers(session_id) {
            let status = if server.running {
                "🟢 running"
            } else {
                "⏸️  stopped"
            };
            println!("  - {} ({}) [{}]", server.id, server.server_type, status);
        }
        println!();
        formatter.print_info("Press Ctrl+C to shutdown...");

        // 等待 Ctrl+C
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| crate::error::SeaError::Io(e))?;

        println!("\nShutting down...");
        agent.shutdown().await?;
        formatter.print_success("Goodbye!");
        Ok(())
    }

    /// 执行 repl 命令
    async fn repl_command(
        &self,
        config: SeaConfig,
        session: Option<String>,
        formatter: &OutputFormatter,
    ) -> Result<()> {
        let agent = SeaAgent::new(config).await?;

        let mut repl = if let Some(session_id_str) = session {
            let session_id = parse_session_id(&session_id_str)?;
            ReplSession::with_session(agent, session_id, formatter.clone())
        } else {
            ReplSession::new(agent, formatter.clone())
        };

        repl.run().await
    }

    /// 执行 session 命令
    async fn session_command(
        &self,
        config: SeaConfig,
        action: SessionActions,
        formatter: &OutputFormatter,
    ) -> Result<()> {
        let agent = SeaAgent::new(config).await?;

        match action {
            SessionActions::Create { name } => {
                let session_id = agent.create_session_with_name(name).await?;
                formatter.print_success(&format!("Session created: {}", session_id));
            }
            SessionActions::List => {
                let sessions = agent.list_sessions().await?;
                print!("{}", formatter.format_session_table(&sessions));
            }
            SessionActions::Show { session_id } => {
                let id = parse_session_id(&session_id)?;
                let session = agent.show_session(id).await?;
                println!("Session: {} ({})", session.session_id, session.name);
                println!("  State: {:?}", session.state);
                println!("  Created: {}", session.created_at);
                println!("  Updated: {}", session.updated_at);
                println!("  Servers: {}", session.servers.len());
                println!("  Messages: {}", session.message_history.len());
                println!("  Routing entries: {}", session.routing_table.len());

                if !session.servers.is_empty() {
                    println!("\n  Registered servers:");
                    for (id, info) in &session.servers {
                        println!("    - {} ({}) [{:?}] tools: {:?}", id, info.name, info.status, info.tools);
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
                formatter.print_success(&format!("Session deleted: {}", id));
            }
        }

        Ok(())
    }

    /// 执行 server 命令
    async fn server_command(
        &self,
        config: SeaConfig,
        action: ServerActions,
        formatter: &OutputFormatter,
    ) -> Result<()> {
        let mut agent = SeaAgent::new(config).await?;

        match action {
            ServerActions::Register {
                session,
                server_type,
                id,
                name,
            } => {
                let session_id = parse_session_id(&session)?;
                let st = parse_server_type_enum(&server_type)?;
                let server_id = agent.register_server_with_name(session_id, st, id, name).await?;
                formatter.print_success(&format!("Server registered: {}", server_id));
            }
            ServerActions::List { session } => {
                let servers = if let Some(session) = session {
                    let session_id = parse_session_id(&session)?;
                    agent.list_session_servers(session_id)
                } else {
                    agent.list_servers()
                };

                print!("{}", formatter.format_server_table(&servers));
            }
            ServerActions::Start { server_id } => {
                agent.start_server(&server_id).await?;
                formatter.print_success(&format!("Server started: {}", server_id));
            }
            ServerActions::Stop { server_id } => {
                agent.stop_server(&server_id).await?;
                formatter.print_success(&format!("Server stopped: {}", server_id));
            }
            ServerActions::Types => {
                println!("Available server types:");
                println!("  llm_gateway - LLM Gateway server (Natural language processing)");
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
    async fn message_command(
        &self,
        config: SeaConfig,
        action: MessageActions,
        formatter: &OutputFormatter,
    ) -> Result<()> {
        let agent = SeaAgent::new(config).await?;

        match action {
            MessageActions::Send { session, content } => {
                let session_id = parse_session_id(&session)?;

                // 显示思考动画
                let pb = formatter.start_thinking();

                let result = agent.send_message(session_id, &content).await?;

                pb.finish_and_clear();

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
                    formatter.print_info("No messages found.");
                } else {
                    println!("Message history:");
                    for msg in messages {
                        println!("  [{:?}] {}: {}", msg.role, msg.timestamp, msg.content);
                    }
                }
            }
        }

        Ok(())
    }

    /// 执行 config 命令
    fn config_command(&self, output: PathBuf, formatter: &OutputFormatter) -> Result<()> {
        let config = SeaConfig::default();
        config
            .save_toml(&output)
            .map_err(|e| crate::error::SeaError::Config(e.to_string()))?;
        formatter.print_success(&format!("Default configuration saved to: {}", output.display()));
        Ok(())
    }

    /// 执行 chat 命令 (快速对话模式)
    async fn chat_command(
        &self,
        config: SeaConfig,
        quick: bool,
        formatter: &OutputFormatter,
    ) -> Result<()> {
        formatter.print_banner();

        if quick {
            formatter.print_info("Quick chat mode - Creating session with default servers...");
        } else {
            formatter.print_info("Chat mode - Starting interactive session...");
        }

        let mut agent = SeaAgent::new(config).await?;

        // 创建或获取 Session
        let session_id = if quick {
            // 快速模式：自动创建 Session 和默认 Servers
            let session_id = agent.create_session().await?;
            formatter.print_success(&format!("Created session: {}", session_id));

            // 自动注册并启动常用 Servers
            let server_types = vec!["echo", "calculator"];
            for server_type in server_types {
                let st = parse_server_type_enum(server_type)?;
                let server_id = agent.register_server(session_id, st, None).await?;
                agent.start_server(&server_id).await?;
                formatter.print_info(&format!("Started server: {} ({})", server_id, server_type));
            }

            session_id
        } else {
            // 正常模式：让用户选择
            let sessions = agent.list_sessions().await?;

            if sessions.is_empty() {
                let session_id = agent.create_session().await?;
                formatter.print_success(&format!("Created session: {}", session_id));
                session_id
            } else {
                // 使用最新的 Session
                sessions[0].session_id
            }
        };

        formatter.print_info(&format!("Using session: {}", session_id));
        println!();
        formatter.print_info("Type your message and press Enter. Press Ctrl+C to exit.");
        println!();

        // 简化的对话循环
        loop {
            use dialoguer::Input;

            let input: String = Input::new()
                .with_prompt("You")
                .interact_text()
                .map_err(|e| crate::error::SeaError::Config(e.to_string()))?;

            if input.trim().is_empty() {
                continue;
            }

            // 显示用户消息
            let timestamp = chrono::Utc::now();
            println!("{}", formatter.format_user_message(&input, &timestamp));

            // 发送消息
            let pb = formatter.start_thinking();
            let result = agent.send_message(session_id, &input).await?;
            pb.finish_and_clear();

            // 显示响应
            let response_timestamp = chrono::Utc::now();
            println!("{}", formatter.format_assistant_message(&result.response, &response_timestamp));
            println!();
        }
    }

    /// 执行 status 命令
    async fn status_command(
        &self,
        config: SeaConfig,
        watch: bool,
        formatter: &OutputFormatter,
    ) -> Result<()> {
        let agent = SeaAgent::new(config).await?;

        if watch {
            self.watch_status(agent, formatter).await
        } else {
            self.show_status_once(agent, formatter).await
        }
    }

    /// 显示一次状态
    async fn show_status_once(&self, agent: SeaAgent, formatter: &OutputFormatter) -> Result<()> {
        println!();
        println!("{} System Status", theme::icons::SYSTEM);
        println!("{}\n", "═".repeat(50));

        let sessions = agent.list_sessions().await?;
        let servers = agent.list_servers();

        println!("  Total Sessions: {}", sessions.len());
        println!("  Total Servers: {}", servers.len());
        println!();

        // 显示活跃的 Servers
        let running_servers: Vec<_> = servers.iter().filter(|s| s.running).collect();
        if !running_servers.is_empty() {
            println!("  Running Servers:");
            for server in running_servers {
                println!("    {} {} ({})", theme::icons::SERVER_RUNNING, server.id, server.server_type);
            }
        } else {
            println!("  No servers currently running");
        }

        println!();
        formatter.print_info("Use 'sea status --watch' for live monitoring");
        println!();

        Ok(())
    }

    /// 持续监控状态
    async fn watch_status(&self, agent: SeaAgent, formatter: &OutputFormatter) -> Result<()> {
        formatter.print_info("Starting status monitor... Press Ctrl+C to exit.");
        println!();

        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));

        loop {
            interval.tick().await;

            // 清屏
            print!("\x1B[2J\x1B[1;1H");

            // 显示时间
            let now = chrono::Local::now();
            println!("{} System Status - {}", theme::icons::SYSTEM, now.format("%Y-%m-%d %H:%M:%S"));
            println!("{}\n", "═".repeat(50));

            // 显示状态
            let sessions = agent.list_sessions().await?;
            let servers = agent.list_servers();

            println!("  Sessions: {} | Servers: {} | Running: {}",
                sessions.len(),
                servers.len(),
                servers.iter().filter(|s| s.running).count()
            );
            println!();

            // 显示 Server 表格
            print!("{}", formatter.format_server_table(&servers));

            println!("\n  Press Ctrl+C to exit");
        }
    }

    /// 执行 help 命令
    fn help_command(&self, topic: Option<&String>) -> Result<()> {
        match topic.map(|s| s.as_str()) {
            Some("session") => HelpFormatter::print_session_help(),
            Some("server") => HelpFormatter::print_server_help(),
            Some("message") => HelpFormatter::print_message_help(),
            Some("repl") => HelpFormatter::print_repl_help(),
            Some("config") => HelpFormatter::print_config_help(),
            Some("quickstart") => HelpFormatter::print_quick_start(),
            Some("troubleshoot") => HelpFormatter::print_troubleshooting(),
            _ => HelpFormatter::print_main_help(),
        }
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
