use crate::manager::SessionManager;
use crate::ServerLifecycle;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "session-manager")]
#[command(about = "Session Manager CLI for Self-Evolving Agent")]
#[command(version)]
pub struct Cli {
    /// Base directory for session files
    #[arg(long, env = "SESSION_DIR", default_value = ".sessions")]
    pub session_dir: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new session
    Create,

    /// List all sessions
    List,

    /// Show session details
    Show {
        /// Session ID
        id: String,
    },

    /// Delete a session
    Delete {
        /// Session ID
        id: String,
    },

    /// Server management commands
    Server {
        #[command(subcommand)]
        command: ServerCommands,
    },
}

#[derive(Subcommand)]
pub enum ServerCommands {
    /// Register a server to a session
    Register {
        /// Session ID
        session_id: String,
        /// Server ID
        server_id: String,
        /// Tools provided by the server (comma-separated)
        #[arg(long, default_value = "")]
        tools: String,
    },

    /// List servers in a session
    List {
        /// Session ID
        session_id: String,
    },

    /// Activate a server (Pending → Active)
    Activate {
        /// Session ID
        session_id: String,
        /// Server ID
        server_id: String,
    },

    /// Drain a server (Active → Draining)
    Drain {
        /// Session ID
        session_id: String,
        /// Server ID
        server_id: String,
    },
}

pub fn run(cli: Cli) -> Result<()> {
    let manager = SessionManager::new(&cli.session_dir);

    match cli.command {
        Commands::Create => {
            let session = manager.create_session()?;
            println!("Session created: {}", session.session_id);
            println!("State: {:?}", session.state);
            println!("Created at: {}", session.created_at);
        }

        Commands::List => {
            let sessions = manager.list_sessions()?;
            if sessions.is_empty() {
                println!("No sessions found.");
                return Ok(());
            }

            println!(
                "{:<40} {:<12} {:<8} {:<8}",
                "ID", "State", "Servers", "Messages"
            );
            println!("{}", "-".repeat(72));
            for s in sessions {
                println!(
                    "{:<40} {:<12} {:<8} {:<8}",
                    s.session_id,
                    format!("{:?}", s.state).to_lowercase(),
                    s.server_count,
                    s.message_count
                );
            }
        }

        Commands::Show { id } => {
            let session_id: Uuid = id.parse().map_err(|_| anyhow::anyhow!("Invalid UUID"))?;
            let session = manager.load_session(session_id)?;

            println!("Session ID: {}", session.session_id);
            println!("State: {:?}", session.state);
            println!("Created: {}", session.created_at);
            println!("Updated: {}", session.updated_at);
            println!("Max hops: {}", session.config.max_hops);
            println!("Drain timeout: {}s", session.config.drain_timeout);

            if !session.servers.is_empty() {
                println!("\nServers:");
                for (id, info) in &session.servers {
                    println!(
                        "  {} [{:?}] tools: {}",
                        id,
                        info.status,
                        if info.tools.is_empty() {
                            "none".to_string()
                        } else {
                            info.tools.join(", ")
                        }
                    );
                }
            }

            if !session.routing_table.is_empty() {
                println!("\nRouting Table:");
                for (cap, sid) in &session.routing_table {
                    println!("  {} → {}", cap, sid);
                }
            }

            println!("\nMessages: {}", session.message_history.len());
            println!(
                "Cache entries: {} input, {} inference",
                session.cache.input_cache.len(),
                session.cache.inference_cache.len()
            );
        }

        Commands::Delete { id } => {
            let session_id: Uuid = id.parse().map_err(|_| anyhow::anyhow!("Invalid UUID"))?;
            manager.delete_session(session_id)?;
            println!("Session {} deleted.", session_id);
        }

        Commands::Server { command } => {
            handle_server_command(&manager, command)?;
        }
    }

    Ok(())
}

fn handle_server_command(manager: &SessionManager, command: ServerCommands) -> Result<()> {
    let lifecycle = ServerLifecycle::new(manager);

    match command {
        ServerCommands::Register {
            session_id,
            server_id,
            tools,
        } => {
            let sid: Uuid = session_id
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid UUID"))?;
            let tools: Vec<String> = if tools.is_empty() {
                vec![]
            } else {
                tools.split(',').map(|s| s.trim().to_string()).collect()
            };

            lifecycle.register_server(sid, server_id.clone(), tools, HashMap::new())?;
            println!(
                "Server '{}' registered to session {} (Pending)",
                server_id, sid
            );
        }

        ServerCommands::List { session_id } => {
            let sid: Uuid = session_id
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid UUID"))?;
            let session = manager.load_session(sid)?;

            if session.servers.is_empty() {
                println!("No servers in this session.");
                return Ok(());
            }

            println!("{:<20} {:<12} Tools", "Server ID", "Status");
            println!("{}", "-".repeat(60));
            for (id, info) in &session.servers {
                println!(
                    "{:<20} {:<12} {}",
                    id,
                    format!("{:?}", info.status).to_lowercase(),
                    if info.tools.is_empty() {
                        "none".to_string()
                    } else {
                        info.tools.join(", ")
                    }
                );
            }
        }

        ServerCommands::Activate {
            session_id,
            server_id,
        } => {
            let sid: Uuid = session_id
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid UUID"))?;
            lifecycle.activate_server(sid, &server_id)?;
            println!("Server '{}' activated.", server_id);
        }

        ServerCommands::Drain {
            session_id,
            server_id,
        } => {
            let sid: Uuid = session_id
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid UUID"))?;
            lifecycle.drain_server(sid, &server_id)?;
            println!("Server '{}' draining.", server_id);
        }
    }

    Ok(())
}
