//! 帮助系统
//!
//! 提供分层帮助信息，包括主帮助、命令帮助和上下文帮助

use colored::*;

use super::theme::icons;

/// 帮助格式化器
pub struct HelpFormatter;

impl HelpFormatter {
    /// 打印主帮助信息
    pub fn print_main_help() {
        println!();
        println!("{} SEA Agent - Command Reference", icons::ROCKET);
        println!("{}\n", "═".repeat(50));

        println!("Usage: sea [OPTIONS] <COMMAND>\n");

        println!("Commands:");
        println!("  {} {:<15} {}", icons::ROCKET, "run", "Start the complete system with default configuration");
        println!("  {} {:<15} {}", icons::CHAT, "repl", "Start interactive REPL mode");
        println!("  {} {:<15} {}", icons::CHAT, "chat", "Quick chat mode (simplified workflow)");
        println!("  {} {:<15} {}", icons::FOLDER, "session", "Manage sessions");
        println!("  {} {:<15} {}", icons::TOOLS, "server", "Manage servers");
        println!("  {} {:<15} {}", icons::CHAT, "message", "Send messages and view history");
        println!("  {} {:<15} {}", icons::CONFIG, "config", "Generate default configuration file");
        println!("  {} {:<15} {}", icons::SYSTEM, "status", "Show system status (use --watch for live monitoring)");

        println!("\nOptions:");
        println!("  {:<25} {}", "--config <PATH>", "Configuration file path");
        println!("  {:<25} {}", "--session-path <PATH>", "Session storage path");
        println!("  {:<25} {}", "--log-level <LEVEL>", "Log level (error|warn|info|debug|trace)");
        println!("  {:<25} {}", "--theme <THEME>", "Theme (default|dark|monochrome)");

        println!("\nExamples:");
        println!("  {} Start with default configuration", "sea run".cyan());
        println!("  {} Start interactive REPL", "sea repl".cyan());
        println!("  {} Create a new session", "sea session create".cyan());
        println!("  {} Send a message", "sea message send --session <ID> \"Hello\"".cyan());
        println!("  {} Monitor servers in real-time", "sea status --watch".cyan());

        println!("\nDocumentation:");
        println!("  {} GitHub: {}", icons::SYSTEM, "https://github.com/az1mus/sea-agent".blue().underline());
        println!("  {} Wiki:   {}", icons::SYSTEM, "https://github.com/az1mus/sea-agent/wiki".blue().underline());
        println!();
    }

    /// 打印 Session 帮助
    pub fn print_session_help() {
        println!("\n{} Session Management", icons::FOLDER);
        println!("{}\n", "═".repeat(50));

        println!("Commands:");
        println!("  {:<10} {}", "create", "Create a new session");
        println!("  {:<10} {}", "list", "List all sessions");
        println!("  {:<10} {}", "show <ID>", "Show session details");
        println!("  {:<10} {}", "delete <ID>", "Delete a session");

        println!("\nOptions:");
        println!("  No additional options for session commands");

        println!("\nExamples:");
        println!("  sea session create");
        println!("  sea session list");
        println!("  sea session show 550e8400-e29b-41d4-a716-446655440000");
        println!("  sea session delete 550e8400-e29b-41d4-a716-446655440000");

        println!("\nSession States:");
        println!("  {:<12} {}", "Active", "Session is active and ready to use");
        println!("  {:<12} {}", "Inactive", "Session is inactive but still saved");
        println!("  {:<12} {}", "Removed", "Session has been marked for deletion");
        println!();
    }

    /// 打印 Server 帮助
    pub fn print_server_help() {
        println!("\n{} Server Management", icons::TOOLS);
        println!("{}\n", "═".repeat(50));

        println!("Commands:");
        println!("  {:<20} {}", "register", "Register a new server to a session");
        println!("  {:<20} {}", "list", "List all servers or servers in a session");
        println!("  {:<20} {}", "start <ID>", "Start a server");
        println!("  {:<20} {}", "stop <ID>", "Stop a server");
        println!("  {:<20} {}", "types", "Show available server types");

        println!("\nRegister Options:");
        println!("  {:<25} {}", "--session <ID>", "Session ID to register the server to");
        println!("  {:<25} {}", "--id <CUSTOM_ID>", "Custom server ID (optional)");

        println!("\nList Options:");
        println!("  {:<25} {}", "--session <ID>", "Filter servers by session ID");

        println!("\nExamples:");
        println!("  sea server register --session <id> echo");
        println!("  sea server register --session <id> calculator --id calc-001");
        println!("  sea server list");
        println!("  sea server list --session <id>");
        println!("  sea server start echo-abc123");
        println!("  sea server stop echo-abc123");
        println!("  sea server types");

        println!("\nServer Types:");
        println!("  {:<15} {}", "llm_gateway", "LLM Gateway server - Natural language processing");
        println!("  {:<15} {}", "echo", "Echo server - echoes messages back");
        println!("  {:<15} {}", "calculator", "Calculator server - performs math operations");
        println!("  {:<15} {}", "time", "Time server - provides time-related functions");
        println!("  {:<15} {}", "counter", "Counter server - named counter management");
        println!("  {:<15} {}", "kvstore", "Key-Value store server - persistent key-value storage");
        println!();
    }

    /// 打印 Message 帮助
    pub fn print_message_help() {
        println!("\n{} Message Operations", icons::CHAT);
        println!("{}\n", "═".repeat(50));

        println!("Commands:");
        println!("  {:<10} {}", "send", "Send a message to a session");
        println!("  {:<10} {}", "history", "View message history of a session");

        println!("\nSend Options:");
        println!("  {:<25} {}", "--session <ID>", "Session ID (required)");
        println!("  {:<25} {}", "<content>", "Message content (JSON or text)");

        println!("\nHistory Options:");
        println!("  {:<25} {}", "--session <ID>", "Session ID (required)");
        println!("  {:<25} {}", "--limit <N>", "Limit number of messages (optional)");
        println!("  {:<25} {}", "--offset <N>", "Skip first N messages (optional)");

        println!("\nExamples:");
        println!("  # Send JSON message");
        println!("  sea message send --session <id> '{{\"action\": \"add\", \"a\": 10, \"b\": 20}}'");
        println!();
        println!("  # Send plain text");
        println!("  sea message send --session <id> \"Hello, SEA!\"");
        println!();
        println!("  # View recent messages");
        println!("  sea message history --session <id>");
        println!();
        println!("  # View with pagination");
        println!("  sea message history --session <id> --limit 20 --offset 10");

        println!("\nMessage Format:");
        println!("  JSON messages should contain one of these fields:");
        println!("  - 'action': The action to perform");
        println!("  - 'capability': The required capability");
        println!("  - 'operation': The operation name");
        println!("  - 'target': The target identifier");
        println!();
    }

    /// 打印 REPL 帮助
    pub fn print_repl_help() {
        println!();
        println!("{} Interactive REPL Mode", icons::CHAT);
        println!("{}\n", "═".repeat(50));

        println!("REPL Commands:");
        println!("  {:<20} {}", "/help", "Show this help message");
        println!("  {:<20} {}", "/quit, /exit", "Exit REPL mode");
        println!("  {:<20} {}", "/clear", "Clear screen");
        println!("  {:<20} {}", "/status", "Show system status");
        println!("  {:<20} {}", "/history", "View message history");
        println!("  {:<20} {}", "/save", "Save current session state");
        println!("  {:<20} {}", "/switch <ID>", "Switch to another session");

        println!("\nSession Commands:");
        println!("  {:<20} {}", "/session list", "List all sessions");
        println!("  {:<20} {}", "/session create", "Create a new session");
        println!("  {:<20} {}", "/session show", "Show current session details");
        println!("  {:<20} {}", "/session delete <ID>", "Delete a session");

        println!("\nServer Commands:");
        println!("  {:<20} {}", "/server list", "List all servers");
        println!("  {:<20} {}", "/server types", "Show available server types");
        println!("  {:<20} {}", "/server register <TYPE>", "Register a new server");
        println!("  {:<20} {}", "/server start <ID>", "Start a server");
        println!("  {:<20} {}", "/server stop <ID>", "Stop a server");

        println!("\nChat:");
        println!("  Simply type your message and press Enter to chat with the agent.");

        println!("\nExamples:");
        println!("  sea> Hello, how are you?");
        println!("  sea> /session create");
        println!("  sea> /server register calculator");
        println!("  sea> /server start calculator-abc123");
        println!("  sea> Calculate 123 + 456");
        println!();
    }

    /// 打印快速入门指南
    pub fn print_quick_start() {
        println!();
        println!("{} Quick Start Guide", icons::ROCKET);
        println!("{}\n", "═".repeat(50));

        println!("Step 1: Create a Session");
        println!("  $ sea session create");
        println!("  ✅ Session created: <session-id>\n");

        println!("Step 2: Register Servers");
        println!("  $ sea server register --session <session-id> echo");
        println!("  $ sea server register --session <session-id> calculator");
        println!("  ✅ Servers registered\n");

        println!("Step 3: Start Servers");
        println!("  $ sea server start echo-abc123");
        println!("  $ sea server start calculator-def456");
        println!("  ✅ Servers started\n");

        println!("Step 4: Send Messages");
        println!("  $ sea message send --session <id> '{{\"action\": \"echo\", \"text\": \"Hello\"}}'");
        println!("  $ sea message send --session <id> '{{\"action\": \"add\", \"a\": 10, \"b\": 20}}'");
        println!("  ✅ Messages sent\n");

        println!("Alternative: Use REPL Mode");
        println!("  $ sea repl");
        println!("  sea> /session create");
        println!("  sea> /server register echo");
        println!("  sea> /server start echo-abc123");
        println!("  sea> Hello, SEA!");
        println!();
    }

    /// 打印配置帮助
    pub fn print_config_help() {
        println!("\n{} Configuration", icons::CONFIG);
        println!("{}\n", "═".repeat(50));

        println!("Generate Default Config:");
        println!("  $ sea config --output config.toml");
        println!("  ✅ Default configuration saved to: config.toml\n");

        println!("Config File Location:");
        println!("  Default paths (searched in order):");
        println!("  1. ./config.toml (current directory)");
        println!("  2. ~/.config/sea-agent/config.toml (Linux/macOS)");
        println!("  3. %APPDATA%\\sea\\sea-agent\\config.toml (Windows)\n");

        println!("Config Options:");
        println!("  session_store_path       - Path to store session files");
        println!("  router.max_hops          - Maximum routing hops");
        println!("  router.drain_timeout     - Drain timeout in seconds");
        println!("  router.classifier_type   - Message classifier type");
        println!("  server_defaults.*        - Default server settings\n");

        println!("Example config.toml:");
        println!("  session_store_path = \"./sessions\"");
        println!();
        println!("  [router]");
        println!("  max_hops = 10");
        println!("  drain_timeout = 300");
        println!("  classifier_type = \"RuleBased\"");
        println!();
        println!("  [server_defaults]");
        println!("  heartbeat_interval_secs = 5");
        println!("  heartbeat_timeout_secs = 30");
        println!();
    }

    /// 打印故障排除指南
    pub fn print_troubleshooting() {
        println!("\n{} Troubleshooting", icons::TOOLS);
        println!("{}\n", "═".repeat(50));

        println!("Common Issues:\n");

        println!("1. \"Session not found\"");
        println!("   Cause: Invalid session ID or session was deleted");
        println!("   Solution: Use 'sea session list' to see available sessions\n");

        println!("2. \"No capability found in message\"");
        println!("   Cause: Message doesn't contain capability identifier");
        println!("   Solution: Use JSON format with 'action' field");
        println!("   Example: {{\"action\": \"echo\", \"text\": \"Hello\"}}\n");

        println!("3. \"No server available for capability\"");
        println!("   Cause: No server provides the required capability");
        println!("   Solution: Register and start a server that provides it");
        println!("   Use 'sea server types' to see available servers\n");

        println!("4. \"Server already stopped\"");
        println!("   Cause: Trying to stop a server that's not running");
        println!("   Note: This is expected behavior for CLI tools");
        println!("   Servers are stopped when the process exits\n");

        println!("5. \"Permission denied\"");
        println!("   Cause: Insufficient permissions for file operations");
        println!("   Solution: Check file permissions and ownership\n");

        println!("Get More Help:");
        println!("  - Run 'sea <command> --help' for command-specific help");
        println!("  - Visit: https://github.com/az1mus/sea-agent/wiki");
        println!("  - Report issues: https://github.com/az1mus/sea-agent/issues");
        println!();
    }
}
