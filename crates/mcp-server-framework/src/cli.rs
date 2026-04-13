use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "mcp-server")]
#[command(about = "MCP Server Framework", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// 启动 Server
    Start {
        /// 配置文件路径
        #[arg(short, long)]
        config: Option<String>,

        /// Server ID (可选,不指定则自动生成)
        #[arg(short = 'i', long)]
        server_id: Option<String>,

        /// Session ID (可选,不指定则自动生成)
        #[arg(short = 's', long)]
        session_id: Option<String>,
    },

    /// 加入 Session
    Join {
        /// Session ID
        #[arg(short, long)]
        session: String,

        /// Server ID (可选)
        #[arg(short = 'i', long)]
        server_id: Option<String>,
    },

    /// 离开 Session
    Leave {
        /// Server ID
        server_id: String,
    },

    /// 查看 Server 状态
    Status {
        /// Server ID
        server_id: String,
    },
}
