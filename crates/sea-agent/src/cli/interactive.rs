//! REPL 交互式会话
//!
//! 提供完整的 REPL（Read-Eval-Print Loop）交互体验

use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use session_manager::SessionId;

use crate::error::Result;
use crate::runtime::SeaAgent;

use super::output::OutputFormatter;
use super::theme::icons;

/// REPL 会话
pub struct ReplSession {
    agent: SeaAgent,
    current_session: Option<SessionId>,
    formatter: OutputFormatter,
    history: Vec<String>,
    history_index: usize,
}

impl ReplSession {
    /// 创建新的 REPL 会话
    pub fn new(agent: SeaAgent, formatter: OutputFormatter) -> Self {
        Self {
            agent,
            current_session: None,
            formatter,
            history: Vec::new(),
            history_index: 0,
        }
    }

    /// 使用指定的 Session 创建 REPL 会话
    pub fn with_session(
        agent: SeaAgent,
        session_id: SessionId,
        formatter: OutputFormatter,
    ) -> Self {
        Self {
            agent,
            current_session: Some(session_id),
            formatter,
            history: Vec::new(),
            history_index: 0,
        }
    }

    /// 运行 REPL 主循环
    pub async fn run(&mut self) -> Result<()> {
        self.formatter.print_banner();

        // 如果没有指定 Session，提示用户选择或创建
        if self.current_session.is_none() {
            self.select_or_create_session().await?;
        }

        println!();
        self.formatter.print_info("Welcome to SEA Agent REPL!");
        println!();
        self.print_quick_help();
        println!();

        // 主循环
        loop {
            // 显示提示符
            self.formatter.print_repl_prompt();

            // 读取用户输入
            let input = match self.read_input() {
                Some(input) => input,
                None => continue, // 输入为空，继续循环
            };

            // 处理输入
            match self.process_input(&input).await {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    self.formatter.print_error(&format!("{}", e));
                }
            }

            // 保存到历史
            self.history.push(input);
            self.history_index = self.history.len();
        }

        self.formatter.print_success("Goodbye!");
        Ok(())
    }

    /// 选择或创建 Session
    async fn select_or_create_session(&mut self) -> Result<()> {
        let sessions = self.agent.list_sessions().await?;

        if sessions.is_empty() {
            // 没有现有 Session，创建新的
            self.formatter.print_info("No existing sessions found. Creating a new one...");

            // 询问 Session 名称
            let session_name: String = Input::new()
                .with_prompt("Enter session name (press Enter for auto-generated)")
                .allow_empty(true)
                .interact_text()
                .map_err(|e| crate::error::SeaError::Config(e.to_string()))?;

            let name = if session_name.trim().is_empty() {
                None
            } else {
                Some(session_name.trim().to_string())
            };

            let session_id = self.agent.create_session_with_name(name).await?;
            self.current_session = Some(session_id);
            self.formatter
                .print_success(&format!("Created session: {}", session_id));
        } else {
            // 有现有 Session，让用户选择
            let mut options: Vec<String> = sessions
                .iter()
                .map(|s| {
                    format!(
                        "{} - {} ({:?}, {} servers)",
                        s.name, s.session_id, s.state, s.server_count
                    )
                })
                .collect();
            options.push("Create new session".to_string());

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a session")
                .items(&options)
                .default(0)
                .interact()
                .map_err(|e| crate::error::SeaError::Config(e.to_string()))?;

            if selection == options.len() - 1 {
                // 创建新 Session
                // 询问 Session 名称
                let session_name: String = Input::new()
                    .with_prompt("Enter session name (press Enter for auto-generated)")
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| crate::error::SeaError::Config(e.to_string()))?;

                let name = if session_name.trim().is_empty() {
                    None
                } else {
                    Some(session_name.trim().to_string())
                };

                let session_id = self.agent.create_session_with_name(name).await?;
                self.current_session = Some(session_id);
                self.formatter
                    .print_success(&format!("Created session: {}", session_id));
            } else {
                // 使用现有 Session
                let session_id = sessions[selection].session_id;
                self.current_session = Some(session_id);
                self.formatter
                    .print_info(&format!("Using session: {}", session_id));
            }
        }

        Ok(())
    }

    /// 读取用户输入
    fn read_input(&self) -> Option<String> {
        // 使用 dialoguer 读取输入
        let input: String = Input::new()
            .with_prompt("")
            .interact_text()
            .ok()?;

        let trimmed = input.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    /// 处理用户输入
    async fn process_input(&mut self, input: &str) -> Result<bool> {
        // 检查是否是斜杠命令
        if input.starts_with('/') {
            self.handle_slash_command(input).await
        } else {
            // 普通消息
            self.handle_message(input).await
        }
    }

    /// 处理斜杠命令
    async fn handle_slash_command(&mut self, cmd: &str) -> Result<bool> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let command = parts[0];

        match command {
            "/help" | "/h" | "/?" => {
                self.print_help();
                Ok(true)
            }
            "/quit" | "/exit" | "/q" => {
                if self.confirm_exit()? {
                    return Ok(false);
                }
                Ok(true)
            }
            "/clear" | "/cls" => {
                self.clear_screen();
                Ok(true)
            }
            "/session" => {
                self.handle_session_command(&parts[1..]).await?;
                Ok(true)
            }
            "/server" => {
                self.handle_server_command(&parts[1..]).await?;
                Ok(true)
            }
            "/status" => {
                self.show_status().await?;
                Ok(true)
            }
            "/history" => {
                self.show_history().await?;
                Ok(true)
            }
            "/switch" => {
                if parts.len() < 2 {
                    self.formatter.print_error("Usage: /switch <session-id>");
                } else {
                    self.switch_session(parts[1]).await?;
                }
                Ok(true)
            }
            "/save" => {
                self.save_session().await?;
                Ok(true)
            }
            _ => {
                self.formatter
                    .print_error(&format!("Unknown command: {}. Type /help for help.", command));
                Ok(true)
            }
        }
    }

    /// 处理普通消息
    async fn handle_message(&mut self, message: &str) -> Result<bool> {
        let session_id = match self.current_session {
            Some(id) => id,
            None => {
                self.formatter
                    .print_error("No active session. Use /session create to create one.");
                return Ok(true);
            }
        };

        // 显示用户消息
        let timestamp = chrono::Utc::now();
        println!(
            "{}",
            self.formatter
                .format_user_message(message, &timestamp)
        );

        // 显示思考动画
        let pb = self.formatter.start_thinking();

        // 智能判断消息类型并构造合适的消息内容
        let content = if message.trim().starts_with('{') && message.trim().ends_with('}') {
            // JSON 格式消息，直接使用
            message.to_string()
        } else {
            // 自然语言消息，发送纯文本（让分类器判定为有机处理，路由到 LLM Gateway）
            message.to_string()
        };

        // 发送消息
        let result = match self.agent.send_message(session_id, &content).await {
            Ok(result) => result,
            Err(e) => {
                pb.finish_and_clear();

                // 如果路由失败，提供友好提示
                let error_msg = format!("{}", e);
                if error_msg.contains("All routers failed") || error_msg.contains("No capability found") {
                    self.formatter.print_error("⚠️  No server available to handle this message.");
                    println!();
                    println!("💡 To handle natural language messages, you need an LLM Gateway server:");
                    println!();
                    println!("   1. Register an LLM Gateway:");
                    println!("      {}", "/server register llm_gateway".cyan());
                    println!();
                    println!("   2. Start the server:");
                    println!("      {}", "/server start <server-id>".cyan());
                    println!();
                    println!("   Or use structured JSON with specific capabilities:");
                    let example1 = r#"{"action": "echo", "text": "Hello"}"#;
                    let example2 = r#"{"capability": "add", "a": 5, "b": 3}"#;
                    println!("      Example: {}", example1.cyan());
                    println!("      Example: {}", example2.cyan());
                    println!();
                    return Ok(true);
                } else {
                    return Err(e);
                }
            }
        };

        // 停止思考动画
        pb.finish_and_clear();

        // 显示响应
        let response_timestamp = chrono::Utc::now();
        println!(
            "{}",
            self.formatter.format_assistant_message(&result.response, &response_timestamp)
        );

        // 显示路由信息
        if !result.routed_servers.is_empty() {
            println!(
                "  {} Routed to: {}",
                icons::TOOLS,
                result.routed_servers.join(", ")
            );
        }

        println!();
        Ok(true)
    }

    // ==================== 内置命令处理 ====================

    /// 打印帮助信息
    fn print_help(&self) {
        println!();
        println!("{} REPL Commands", icons::TOOLS);
        println!("{}", "═".repeat(50));
        println!();
        println!("  {:<20} {}", "/help, /h, /?", "Show this help message");
        println!("  {:<20} {}", "/quit, /exit, /q", "Exit REPL mode");
        println!("  {:<20} {}", "/clear, /cls", "Clear screen");
        println!("  {:<20} {}", "/status", "Show system status");
        println!("  {:<20} {}", "/history", "View message history");
        println!("  {:<20} {}", "/save", "Save current session state");
        println!("  {:<20} {}", "/switch <id>", "Switch to another session");
        println!();
        println!("  Session management:");
        println!("  {:<20} {}", "/session list", "List all sessions");
        println!("  {:<20} {}", "/session create", "Create a new session");
        println!("  {:<20} {}", "/session show", "Show current session details");
        println!("  {:<20} {}", "/session delete <id>", "Delete a session");
        println!();
        println!("  Server management:");
        println!("  {:<20} {}", "/server list", "List all servers");
        println!("  {:<20} {}", "/server types", "Show available server types");
        println!("  {:<20} {}", "/server register <type>", "Register a new server");
        println!("  {:<20} {}", "/server start <id>", "Start a server");
        println!("  {:<20} {}", "/server stop <id>", "Stop a server");
        println!();
        println!("  Chat:");
        println!("  {:<20} {}", "<message>", "Send a message to the agent");
        println!("  {:<20} {}", "↑/↓", "Navigate message history");
        println!();
    }

    /// 打印快速帮助
    fn print_quick_help(&self) {
        println!("{} Quick Start:", icons::ROCKET);
        println!("  Type a message to chat with the agent");
        println!("  Type /help to see all commands");
        println!("  Type /quit to exit");
        println!();
    }

    /// 清屏
    fn clear_screen(&self) {
        // ANSI escape code to clear screen
        print!("\x1B[2J\x1B[1;1H");
    }

    /// 确认退出
    fn confirm_exit(&self) -> Result<bool> {
        let confirm = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Exit REPL?")
            .default(true)
            .interact()
            .map_err(|e| crate::error::SeaError::Config(e.to_string()))?;

        Ok(confirm)
    }

    /// 处理 session 命令
    async fn handle_session_command(&mut self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            self.formatter.print_error("Usage: /session <list|create|show|delete>");
            return Ok(());
        }

        match args[0] {
            "list" => {
                let sessions = self.agent.list_sessions().await?;
                print!("{}", self.formatter.format_session_table(&sessions));
            }
            "create" => {
                // 询问 Session 名称
                let session_name: String = Input::new()
                    .with_prompt("Enter session name (press Enter for auto-generated)")
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| crate::error::SeaError::Config(e.to_string()))?;

                let name = if session_name.trim().is_empty() {
                    None
                } else {
                    Some(session_name.trim().to_string())
                };

                let session_id = self.agent.create_session_with_name(name).await?;
                self.formatter
                    .print_success(&format!("Created session: {}", session_id));
            }
            "show" => {
                if let Some(session_id) = self.current_session {
                    let session = self.agent.show_session(session_id).await?;
                    println!("Session: {}", session.session_id);
                    println!("  State: {:?}", session.state);
                    println!("  Created: {}", session.created_at);
                    println!("  Servers: {}", session.servers.len());
                    println!("  Messages: {}", session.message_history.len());
                    println!("  Routing entries: {}", session.routing_table.len());
                } else {
                    self.formatter.print_info("No active session");
                }
            }
            "delete" => {
                if args.len() < 2 {
                    self.formatter.print_error("Usage: /session delete <session-id>");
                    return Ok(());
                }

                let session_id = self.parse_session_id(args[1])?;
                self.agent.delete_session(session_id).await?;
                self.formatter
                    .print_success(&format!("Deleted session: {}", session_id));

                // 如果删除的是当前 Session，清空当前 Session
                if self.current_session == Some(session_id) {
                    self.current_session = None;
                    self.formatter
                        .print_info("Current session was deleted. Use /session create or /switch to select a new session.");
                }
            }
            _ => {
                self.formatter
                    .print_error(&format!("Unknown session command: {}", args[0]));
            }
        }

        Ok(())
    }

    /// 处理 server 命令
    async fn handle_server_command(&mut self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            self.formatter.print_error("Usage: /server <list|types|register|start|stop>");
            return Ok(());
        }

        match args[0] {
            "list" => {
                let servers = self.agent.list_servers();
                print!("{}", self.formatter.format_server_table(&servers));
            }
            "types" => {
                println!("Available server types:");
                println!("  llm_gateway - LLM Gateway server (Natural language processing)");
                println!("  echo - Echo server");
                println!("  calculator - Calculator server");
                println!("  time - Time server");
                println!("  counter - Counter server");
                println!("  kvstore - Key-Value store server");
            }
            "register" => {
                if args.len() < 2 {
                    self.formatter.print_error("Usage: /server register <type>");
                    return Ok(());
                }

                let session_id = match self.current_session {
                    Some(id) => id,
                    None => {
                        self.formatter.print_error("No active session");
                        return Ok(());
                    }
                };

                let server_type = self.parse_server_type(args[1])?;

                // 询问 Server 名称
                let server_name: String = Input::new()
                    .with_prompt("Enter server name (press Enter for auto-generated)")
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|e| crate::error::SeaError::Config(e.to_string()))?;

                let name = if server_name.trim().is_empty() {
                    None
                } else {
                    Some(server_name.trim().to_string())
                };

                let server_id = self.agent.register_server_with_name(session_id, server_type, None, name).await?;
                self.formatter
                    .print_success(&format!("Registered server: {}", server_id));
            }
            "start" => {
                if args.len() < 2 {
                    self.formatter.print_error("Usage: /server start <server-id>");
                    return Ok(());
                }

                self.agent.start_server(args[1]).await?;
                self.formatter
                    .print_success(&format!("Started server: {}", args[1]));
            }
            "stop" => {
                if args.len() < 2 {
                    self.formatter.print_error("Usage: /server stop <server-id>");
                    return Ok(());
                }

                self.agent.stop_server(args[1]).await?;
                self.formatter
                    .print_success(&format!("Stopped server: {}", args[1]));
            }
            _ => {
                self.formatter
                    .print_error(&format!("Unknown server command: {}", args[0]));
            }
        }

        Ok(())
    }

    /// 显示状态
    async fn show_status(&self) -> Result<()> {
        println!();
        println!("{} System Status", icons::SYSTEM);
        println!("{}", "═".repeat(50));

        if let Some(session_id) = self.current_session {
            let session = self.agent.show_session(session_id).await?;

            println!("  Current Session: {}", session_id);
            println!("  State: {:?}", session.state);
            println!("  Active Servers: {}", session.servers.len());
            println!("  Total Messages: {}", session.message_history.len());
            println!("  Routing Entries: {}", session.routing_table.len());

            if !session.servers.is_empty() {
                println!();
                println!("  Servers:");
                for (id, info) in &session.servers {
                    let status = match info.status {
                        session_manager::ServerStatus::Pending => "⏳ Pending",
                        session_manager::ServerStatus::Active => "🟢 Active",
                        session_manager::ServerStatus::Draining => "🔄 Draining",
                        session_manager::ServerStatus::Removed => "❌ Removed",
                    };
                    println!("    {} {} - {:?}", status, id, info.tools);
                }
            }
        } else {
            println!("  No active session");
        }

        println!();
        Ok(())
    }

    /// 显示历史消息
    async fn show_history(&self) -> Result<()> {
        let session_id = match self.current_session {
            Some(id) => id,
            None => {
                self.formatter.print_info("No active session");
                return Ok(());
            }
        };

        let messages = self.agent.get_message_history(session_id, Some(20), None).await?;

        if messages.is_empty() {
            self.formatter.print_info("No messages in history");
        } else {
            println!();
            println!("{} Message History (last 20)", icons::CHAT);
            println!("{}", "═".repeat(50));
            for msg in messages {
                let formatted = match msg.role {
                    session_manager::MessageRole::User => self
                        .formatter
                        .format_user_message(&msg.content, &msg.timestamp),
                    session_manager::MessageRole::Assistant => self
                        .formatter
                        .format_assistant_message(&msg.content, &msg.timestamp),
                    session_manager::MessageRole::System => self
                        .formatter
                        .format_system_message("info", &msg.content),
                    session_manager::MessageRole::Server => self
                        .formatter
                        .format_system_message("info", &msg.content),
                };
                println!("{}", formatted);
            }
            println!();
        }

        Ok(())
    }

    /// 切换 Session
    async fn switch_session(&mut self, session_id_str: &str) -> Result<()> {
        let session_id = self.parse_session_id(session_id_str)?;

        // 验证 Session 是否存在
        let _ = self.agent.show_session(session_id).await?;

        self.current_session = Some(session_id);
        self.formatter
            .print_success(&format!("Switched to session: {}", session_id));

        Ok(())
    }

    /// 保存 Session
    async fn save_session(&self) -> Result<()> {
        if let Some(session_id) = self.current_session {
            // Session 自动保存，这里只是提示用户
            self.formatter
                .print_success(&format!("Session {} saved", session_id));
        } else {
            self.formatter.print_info("No active session to save");
        }
        Ok(())
    }

    // ==================== 工具方法 ====================

    /// 解析 Session ID
    fn parse_session_id(&self, s: &str) -> Result<SessionId> {
        uuid::Uuid::parse_str(s).map_err(|_| {
            crate::error::SeaError::InvalidOperation(format!("Invalid session ID: {}", s))
        })
    }

    /// 解析 Server 类型
    fn parse_server_type(&self, s: &str) -> Result<concrete_servers::factory::ServerType> {
        use std::str::FromStr;
        concrete_servers::factory::ServerType::from_str(s).map_err(|_| {
            crate::error::SeaError::InvalidOperation(format!(
                "Unknown server type: '{}'. Available: echo, calculator, time, counter, kvstore",
                s
            ))
        })
    }
}
