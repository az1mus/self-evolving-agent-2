//! CLI 输出格式化引擎
//!
//! 提供统一的输出格式化、颜色输出、表格渲染、进度指示器

use chrono::{DateTime, Utc};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use prettytable::{format, Cell, Row, Table};
use session_manager::SessionSummary;

use crate::runtime::ServerInfo;

use super::theme::{Theme, ThemeType};

/// 输出格式化器
#[derive(Clone)]
pub struct OutputFormatter {
    theme: Theme,
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self::new(ThemeType::Default)
    }
}

impl OutputFormatter {
    pub fn new(theme_type: ThemeType) -> Self {
        Self {
            theme: theme_type.to_theme(),
        }
    }

    // ==================== 横幅和欢迎信息 ====================

    /// 打印欢迎横幅
    pub fn print_banner(&self) {
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

    /// 打印 REPL 提示符
    pub fn print_repl_prompt(&self) {
        print!("sea> ");
    }

    // ==================== 表格格式化 ====================

    /// 格式化 Session 列表为表格
    pub fn format_session_table(&self, sessions: &[SessionSummary]) -> String {
        if sessions.is_empty() {
            return format!("{} No sessions found.\n", self.theme.system_icon);
        }

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table.set_titles(Row::new(vec![
            Cell::new("ID").style_spec("Fc"),
            Cell::new("Name").style_spec("Fc"),
            Cell::new("State").style_spec("Fc"),
            Cell::new("Servers").style_spec("Fc"),
            Cell::new("Messages").style_spec("Fc"),
            Cell::new("Created").style_spec("Fc"),
        ]));

        for session in sessions {
            table.add_row(Row::new(vec![
                Cell::new(&format!("{:.8}", session.session_id)).style_spec("Fd"),
                Cell::new(&session.name).style_spec("Fb"),
                Cell::new(&format!("{:?}", session.state)).style_spec("Fc"),
                Cell::new(&session.server_count.to_string()).style_spec("Fg"),
                Cell::new(&session.message_count.to_string()).style_spec("Fb"),
                Cell::new(&self.format_datetime(session.created_at)),
            ]));
        }

        format!("\n{}\n", table)
    }

    /// 格式化 Server 列表为表格
    pub fn format_server_table(&self, servers: &[ServerInfo]) -> String {
        if servers.is_empty() {
            return format!("{} No servers found.\n", self.theme.system_icon);
        }

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_CLEAN);

        table.set_titles(Row::new(vec![
            Cell::new("ID").style_spec("Fb"),
            Cell::new("Name").style_spec("Fc"),
            Cell::new("Type").style_spec("Fc"),
            Cell::new("Session").style_spec("Fd"),
            Cell::new("Status").style_spec("Fc"),
        ]));

        for server in servers {
            let status = if server.running {
                format!("{} Running", self.theme.server_running).green()
            } else {
                format!("{} Stopped", self.theme.server_stopped).yellow()
            };

            table.add_row(Row::new(vec![
                Cell::new(&server.id).style_spec("Fb"),
                Cell::new(&server.name).style_spec("Fc"),
                Cell::new(&server.server_type.to_string()),
                Cell::new(&format!("{:.8}", server.session_id)).style_spec("Fd"),
                Cell::new(&status.to_string()),
            ]));
        }

        format!("\n{}\n", table)
    }

    // ==================== 消息格式化 ====================

    /// 格式化用户消息
    pub fn format_user_message(&self, content: &str, timestamp: &DateTime<Utc>) -> String {
        format!(
            "{} {} {}",
            self.theme.user_icon.blue(),
            timestamp.format("%H:%M:%S").to_string().dimmed(),
            content.bold().blue()
        )
    }

    /// 格式化助手响应
    pub fn format_assistant_message(&self, content: &str, timestamp: &DateTime<Utc>) -> String {
        format!(
            "{} {} {}",
            self.theme.assistant_icon.cyan(),
            timestamp.format("%H:%M:%S").to_string().dimmed(),
            content.cyan()
        )
    }

    /// 格式化系统消息
    pub fn format_system_message(&self, level: &str, message: &str) -> String {
        let icon = match level {
            "error" => self.theme.error_icon,
            "warning" => self.theme.warning_icon,
            "success" => self.theme.success_icon,
            _ => self.theme.system_icon,
        };

        let color = match level {
            "error" => Color::Red,
            "warning" => Color::Yellow,
            "success" => Color::Green,
            _ => Color::Blue,
        };

        format!("{} {}", icon, message.color(color))
    }

    // ==================== 进度指示器 ====================

    /// 创建思考动画
    pub fn start_thinking(&self) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Invalid template")
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        pb.set_message("Thinking...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        pb
    }

    /// 创建 Server 启动进度
    pub fn start_server_startup(&self, server_id: &str) -> ProgressBar {
        let pb = ProgressBar::new(3);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "{{spinner:.green}} [{{elapsed_precise}}] [{{bar:40.cyan/blue}}] {{msg}}"
                ))
                .expect("Invalid template")
                .progress_chars("█▓▒░ "),
        );
        pb.set_message(format!("Starting server {}...", server_id));
        pb
    }

    /// 创建批量操作进度
    pub fn start_batch_operation(&self, total: u64, operation: &str) -> ProgressBar {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "{{spinner:.green}} {} [{{bar:40.cyan/blue}}] {{percent}}% ETA: {{eta}}",
                    operation
                ))
                .expect("Invalid template")
                .progress_chars("#>-"),
        );
        pb
    }

    // ==================== 错误格式化 ====================

    /// 格式化错误消息
    pub fn format_error(&self, error_type: &str, message: &str, suggestion: Option<&str>) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "\n{} {}\n\n",
            self.theme.error_icon,
            "Error Occurred".red().bold()
        ));

        output.push_str(&format!("  Type: {}\n", error_type.yellow()));
        output.push_str(&format!("  Message: {}\n", message));

        if let Some(suggestion) = suggestion {
            output.push_str(&format!("\n  💡 {}\n", suggestion.yellow()));
        }

        output.push_str(&format!(
            "\n  {} For more help, run: {}\n\n",
            self.theme.system_icon,
            "sea help".cyan()
        ));

        output
    }

    // ==================== 工具方法 ====================

    /// 格式化日期时间
    fn format_datetime(&self, dt: DateTime<Utc>) -> String {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// 打印成功消息
    pub fn print_success(&self, message: &str) {
        println!("{} {}", self.theme.success_icon, message.green());
    }

    /// 打印错误消息
    pub fn print_error(&self, message: &str) {
        eprintln!("{} {}", self.theme.error_icon, message.red());
    }

    /// 打印警告消息
    pub fn print_warning(&self, message: &str) {
        println!("{} {}", self.theme.warning_icon, message.yellow());
    }

    /// 打印信息消息
    pub fn print_info(&self, message: &str) {
        println!("{} {}", self.theme.system_icon, message.blue());
    }
}
