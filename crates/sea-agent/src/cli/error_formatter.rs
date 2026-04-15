//! 错误格式化器
//!
//! 提供友好的错误提示，包括上下文信息和建议

use colored::*;
use crate::error::SeaError;

use super::theme::icons;

/// 错误格式化器
pub struct ErrorFormatter {
    verbose: bool,
}

impl Default for ErrorFormatter {
    fn default() -> Self {
        Self { verbose: false }
    }
}

impl ErrorFormatter {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    /// 格式化错误消息
    pub fn format_error(&self, error: &SeaError) -> String {
        let mut output = String::new();

        // 错误标题
        output.push_str(&format!(
            "\n{} {}\n\n",
            icons::ERROR,
            "Error Occurred".red().bold()
        ));

        // 错误详情
        match error {
            SeaError::Session(e) => {
                output.push_str(&self.format_session_error(e));
            }
            SeaError::Router(e) => {
                output.push_str(&self.format_router_error(e));
            }
            SeaError::Server(msg) => {
                output.push_str(&self.format_server_error(msg));
            }
            SeaError::Config(msg) => {
                output.push_str(&self.format_config_error(msg));
            }
            SeaError::Io(e) => {
                output.push_str(&self.format_io_error(e));
            }
            SeaError::NotFound(what) => {
                output.push_str(&self.format_not_found_error(what));
            }
            SeaError::InvalidOperation(msg) => {
                output.push_str(&self.format_invalid_operation_error(msg));
            }
        }

        // 帮助链接
        output.push_str(&format!(
            "\n  {} For more help, run: {}\n",
            icons::SYSTEM,
            "sea help".cyan()
        ));

        output.push_str(&format!(
            "  {} Documentation: {}\n\n",
            icons::SYSTEM,
            "https://github.com/az1mus/sea-agent/wiki".blue().underline()
        ));

        output
    }

    /// 格式化 Session 错误
    fn format_session_error(&self, error: &session_manager::ManagerError) -> String {
        let mut output = String::new();

        output.push_str(&format!("  Type: {}\n", "Session Error".yellow()));

        use session_manager::ManagerError::*;
        match error {
            NotFound(id) => {
                output.push_str(&format!("  Session not found: {}\n", id.to_string().cyan()));
                output.push_str(&format!(
                    "\n  {} Use '{}' to list available sessions.\n",
                    icons::LIGHT_BULB.yellow(),
                    "sea session list".cyan()
                ));
            }
            InvalidOperation(msg) => {
                output.push_str(&format!("  Invalid operation: {}\n", msg));
            }
            Io(e) => {
                output.push_str(&format!("  I/O error: {}\n", e));
                output.push_str(&format!(
                    "\n  {} Check file permissions and disk space.\n",
                    icons::LIGHT_BULB.yellow()
                ));
            }
            _ => {
                output.push_str(&format!("  {}\n", error));
            }
        }

        if self.verbose {
            output.push_str(&format!("\n  Debug: {:?}\n", error));
        }

        output
    }

    /// 格式化 Router 错误
    fn format_router_error(&self, error: &router_core::RouterError) -> String {
        let mut output = String::new();

        output.push_str(&format!("  Type: {}\n", "Router Error".yellow()));

        use router_core::RouterError::*;
        match error {
            NoCapableServer(capability) => {
                output.push_str(&format!("  No server available for capability: {}\n", capability.cyan()));
                output.push_str(&format!(
                    "\n  {} Register a server that provides this capability.\n",
                    icons::LIGHT_BULB.yellow()
                ));
                output.push_str(&format!(
                    "  Use '{}' to see available server types.\n",
                    "sea server types".cyan()
                ));
            }
            RoutingFailed(msg) => {
                output.push_str(&format!("  Routing failed: {}\n", msg));
            }
            MaxHopsExceeded(current, max) => {
                output.push_str(&format!("  Max routing hops exceeded: {} (max: {})\n", current, max));
                output.push_str(&format!(
                    "\n  {} Possible circular routing detected. Check routing table.\n",
                    icons::LIGHT_BULB.yellow()
                ));
            }
            InvalidMessage(msg) => {
                output.push_str(&format!("  Invalid message: {}\n", msg));
                output.push_str(&format!(
                    "\n  {} Use JSON format with 'action', 'capability', 'operation', or 'target' field.\n",
                    icons::LIGHT_BULB.yellow()
                ));
            }
            CycleDetected(server_id) => {
                output.push_str(&format!("  Cycle detected: message already visited server {}\n", server_id.cyan()));
                output.push_str(&format!(
                    "\n  {} Check routing table for circular dependencies.\n",
                    icons::LIGHT_BULB.yellow()
                ));
            }
            _ => {
                output.push_str(&format!("  {}\n", error));
            }
        }

        output
    }

    /// 格式化 Server 错误
    fn format_server_error(&self, msg: &str) -> String {
        let mut output = String::new();

        output.push_str(&format!("  Type: {}\n", "Server Error".yellow()));
        output.push_str(&format!("  {}\n", msg));

        if msg.contains("start") || msg.contains("stop") {
            output.push_str(&format!(
                "\n  {} Check server status with '{}'.\n",
                icons::LIGHT_BULB.yellow(),
                "sea server list".cyan()
            ));
        }

        output
    }

    /// 格式化配置错误
    fn format_config_error(&self, msg: &str) -> String {
        let mut output = String::new();

        output.push_str(&format!("  Type: {}\n", "Configuration Error".yellow()));
        output.push_str(&format!("  {}\n", msg));

        output.push_str(&format!(
            "\n  {} Generate a default config with '{}'.\n",
            icons::LIGHT_BULB.yellow(),
            "sea config --output config.toml".cyan()
        ));

        output
    }

    /// 格式化 I/O 错误
    fn format_io_error(&self, error: &std::io::Error) -> String {
        let mut output = String::new();

        output.push_str(&format!("  Type: {}\n", "I/O Error".yellow()));
        output.push_str(&format!("  {}\n", error));

        match error.kind() {
            std::io::ErrorKind::NotFound => {
                output.push_str(&format!(
                    "\n  {} File or directory not found.\n",
                    icons::LIGHT_BULB.yellow()
                ));
            }
            std::io::ErrorKind::PermissionDenied => {
                output.push_str(&format!(
                    "\n  {} Permission denied. Check file permissions.\n",
                    icons::LIGHT_BULB.yellow()
                ));
            }
            std::io::ErrorKind::AlreadyExists => {
                output.push_str(&format!(
                    "\n  {} Resource already exists.\n",
                    icons::LIGHT_BULB.yellow()
                ));
            }
            _ => {}
        }

        output
    }

    /// 格式化未找到错误
    fn format_not_found_error(&self, what: &str) -> String {
        let mut output = String::new();

        output.push_str(&format!("  Type: {}\n", "Not Found".yellow()));
        output.push_str(&format!("  {}\n", what.cyan()));

        if what.to_lowercase().contains("session") {
            output.push_str(&format!(
                "\n  {} Use '{}' to list available sessions.\n",
                icons::LIGHT_BULB.yellow(),
                "sea session list".cyan()
            ));
        } else if what.to_lowercase().contains("server") {
            output.push_str(&format!(
                "\n  {} Use '{}' to list available servers.\n",
                icons::LIGHT_BULB.yellow(),
                "sea server list".cyan()
            ));
        }

        output
    }

    /// 格式化无效操作错误
    fn format_invalid_operation_error(&self, msg: &str) -> String {
        let mut output = String::new();

        output.push_str(&format!("  Type: {}\n", "Invalid Operation".yellow()));
        output.push_str(&format!("  {}\n", msg));

        if msg.contains("session id") || msg.contains("Session ID") {
            output.push_str(&format!(
                "\n  {} Session ID should be in UUID format.\n",
                icons::LIGHT_BULB.yellow()
            ));
            output.push_str(&format!(
                "  Example: {}\n",
                "550e8400-e29b-41d4-a716-446655440000".cyan()
            ));
        } else if msg.contains("server type") {
            output.push_str(&format!(
                "\n  {} Use '{}' to see available server types.\n",
                icons::LIGHT_BULB.yellow(),
                "sea server types".cyan()
            ));
        }

        output
    }

    /// 打印警告消息
    pub fn print_warning(&self, message: &str) {
        println!("\n{} {}\n", icons::WARNING, message.yellow());
    }

    /// 打印信息消息
    pub fn print_info(&self, message: &str) {
        println!("\n{} {}\n", icons::SYSTEM, message.blue());
    }

    /// 打印成功消息
    pub fn print_success(&self, message: &str) {
        println!("\n{} {}\n", icons::SUCCESS, message.green());
    }
}
