//! SEA Agent 主入口

use clap::Parser;
use sea_agent::SeaCli;

#[tokio::main]
async fn main() {
    let cli = SeaCli::parse();

    if let Err(e) = cli.execute().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}