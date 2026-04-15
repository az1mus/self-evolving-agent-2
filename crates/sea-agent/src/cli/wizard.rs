//! 首次运行向导
//!
//! 引导新用户完成初始配置

use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::path::PathBuf;

use crate::config::SeaConfig;
use crate::error::Result;
use crate::runtime::SeaAgent;

use super::output::OutputFormatter;
use super::theme::icons;

/// 首次运行向导
pub struct FirstRunWizard {
    formatter: OutputFormatter,
}

impl FirstRunWizard {
    pub fn new(formatter: OutputFormatter) -> Self {
        Self { formatter }
    }

    /// 检查是否需要首次运行向导
    pub fn should_run() -> bool {
        // 检查是否存在配置文件或 Session 文件
        let config_path = SeaConfig::default_config_path();
        !config_path.exists()
    }

    /// 运行首次运行向导
    pub async fn run(&self) -> Result<SeaConfig> {
        self.print_welcome_banner();

        println!("🚀 Welcome to SEA Agent! Let's set up your environment.\n");

        // 步骤 1: 存储路径配置
        let config = self.configure_storage()?;

        // 步骤 2: 询问是否创建默认 Session
        let create_default = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Create a default session with basic servers?")
            .default(true)
            .interact()
            .map_err(|e| crate::error::SeaError::Config(e.to_string()))?;

        if create_default {
            self.create_default_session(&config).await?;
        }

        // 保存配置
        self.save_config(&config)?;

        self.formatter.print_success("Setup complete!");
        println!("\n{} Run 'sea repl' to start chatting or 'sea guide' for help.\n", icons::ROCKET);

        Ok(config)
    }

    /// 打印欢迎横幅
    fn print_welcome_banner(&self) {
        println!();
        println!("╔════════════════════════════════════════════════════╗");
        println!("║                                                    ║");
        println!("║   ███████╗███████╗ ██████╗██╗  ██╗    ██╗██╗███╗   ██╗");
        println!("║   ██╔════╝██╔════╝██╔════╝██║ ██╔╝    ██║██║████╗  ██║");
        println!("║   ███████╗█████╗  ██║     █████╔╝     ██║██║██╔██╗ ██║");
        println!("║   ╚════██║██╔══╝  ██║     ██╔═██╗     ██║██║██║╚██╗██║");
        println!("║   ███████║███████╗╚██████╗██║  ██╗██╗ ██║██║██║ ╚████║");
        println!("║   ╚══════╝╚══════╝ ╚═════╝╚═╝  ╚═╝╚═╝ ╚═╝╚═╝╚═╝  ╚═══╝");
        println!("║                                                    ║");
        println!("║        Self-Evolving Agent v0.2.0                 ║");
        println!("║        MCP-Based Intelligent Agent System          ║");
        println!("║                                                    ║");
        println!("╚══════════════════════════════════════════════════╝");
        println!();
    }

    /// 配置存储路径
    fn configure_storage(&self) -> Result<SeaConfig> {
        println!("Step 1: Storage Configuration");
        println!("─────────────────────────────\n");

        let session_path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Session storage path")
            .default("./sessions".to_string())
            .interact()
            .map_err(|e| crate::error::SeaError::Config(e.to_string()))?;

        println!();

        Ok(SeaConfig {
            session_store_path: PathBuf::from(session_path),
            ..Default::default()
        })
    }

    /// 创建默认 Session
    async fn create_default_session(&self, config: &SeaConfig) -> Result<()> {
        println!("Step 2: Creating Default Session");
        println!("─────────────────────────────────\n");

        let mut agent = SeaAgent::new(config.clone()).await?;

        // 创建 Session
        let session_id = agent.create_session().await?;
        self.formatter
            .print_success(&format!("Created session: {}", session_id));

        // 注册默认 Servers，包括 LLM Gateway 用于自然语言处理
        let default_servers = vec!["llm_gateway", "echo", "calculator"];

        for server_type in default_servers {
            let st = Self::parse_server_type(server_type)?;
            let server_id = agent.register_server(session_id, st, None).await?;
            agent.start_server(&server_id).await?;

            let server_desc = match server_type {
                "llm_gateway" => "Natural Language Processing",
                "echo" => "Echo Server",
                "calculator" => "Calculator Server",
                _ => server_type,
            };
            println!("  {} Started server: {} ({})", icons::SERVER_RUNNING, server_id, server_desc);
        }

        println!();
        self.formatter
            .print_info(&format!("Default session {} is ready!", session_id));
        self.formatter.print_info("You can now chat naturally or use JSON commands.");

        Ok(())
    }

    /// 保存配置
    fn save_config(&self, config: &SeaConfig) -> Result<()> {
        let config_path = SeaConfig::default_config_path();

        // 确保目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| crate::error::SeaError::Io(e))?;
        }

        config
            .save_toml(&config_path)
            .map_err(|e| crate::error::SeaError::Config(e.to_string()))?;

        println!(
            "\n{} Configuration saved to: {}",
            icons::SUCCESS,
            config_path.display()
        );

        Ok(())
    }

    /// 解析 Server 类型
    fn parse_server_type(s: &str) -> Result<concrete_servers::factory::ServerType> {
        use std::str::FromStr;
        concrete_servers::factory::ServerType::from_str(s).map_err(|_| {
            crate::error::SeaError::InvalidOperation(format!("Unknown server type: {}", s))
        })
    }
}
