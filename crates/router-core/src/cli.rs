use clap::{Parser, Subcommand};
use router_core::{
    CapabilityRouter, Classifier, Message, MessageContent, MockClassifier, RouterCore,
    RuleBasedClassifier, SessionManager,
};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "router-core")]
#[command(about = "Router Core CLI - 消息路由与有机/无机判定", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 判定消息的处理类型
    Classify {
        /// 消息内容(JSON 格式)
        #[arg(short, long)]
        message: String,
        /// 使用 Mock 判定器(指定 organic 或 inorganic)
        #[arg(long)]
        mock: Option<String>,
    },
    /// 路由消息到目标 Server
    Route {
        /// 消息内容(JSON 格式)
        #[arg(short, long)]
        message: String,
        /// Session 文件路径
        #[arg(short, long)]
        session: PathBuf,
    },
    /// 预处理消息
    Preprocess {
        /// 消息内容(JSON 格式)
        #[arg(short, long)]
        message: String,
    },
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Classify { message, mock } => {
            let content = if message.trim().starts_with('{') {
                // 尝试解析为 JSON
                MessageContent::structured(serde_json::from_str(&message)?)
            } else {
                // 作为纯文本处理
                MessageContent::unstructured(&message)
            };
            let session_id = Uuid::new_v4();
            let msg = Message::simple(session_id, content);

            let classifier: Box<dyn Classifier> = if let Some(mock_type) = mock {
                match mock_type.as_str() {
                    "organic" => Box::new(MockClassifier::organic()),
                    "inorganic" => Box::new(MockClassifier::inorganic()),
                    _ => anyhow::bail!(
                        "Invalid mock type: {}. Use 'organic' or 'inorganic'",
                        mock_type
                    ),
                }
            } else {
                // 默认使用规则判定器
                Box::new(RuleBasedClassifier::new())
            };

            let result = classifier.classify(&msg).await?;

            println!("Message ID: {}", msg.message_id);
            println!("Processing Type: {}", result);
            println!(
                "Content Type: {}",
                if msg.content.is_structured() {
                    "structured"
                } else {
                    "unstructured"
                }
            );
        }
        Commands::Route { message, session } => {
            let content = MessageContent::structured(serde_json::from_str(&message)?);

            // 加载 Session
            let manager = SessionManager::new(session.parent().unwrap_or(&PathBuf::from(".")));
            let session_id = Uuid::parse_str(
                session
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| anyhow::anyhow!("Invalid session file name"))?,
            )?;
            let session = manager.load_session(session_id)?;

            let msg = Message::simple(session_id, content);

            // 创建 Router Core
            let core = RouterCore::new(
                Box::new(RuleBasedClassifier::new()),
                Box::new(CapabilityRouter::new()),
            );

            let servers = core.process(msg, &session).await?;

            println!("Routed to servers:");
            for server in servers {
                println!("  - {}", server);
            }
        }
        Commands::Preprocess { message } => {
            let content = MessageContent::structured(serde_json::from_str(&message)?);
            let session_id = Uuid::new_v4();
            let msg = Message::simple(session_id, content);

            println!("Original message:");
            println!("{}", serde_json::to_string_pretty(&msg)?);

            // 这里可以添加预处理逻辑
            println!("\nPreprocessing would be applied based on processing type.");
        }
    }

    Ok(())
}
