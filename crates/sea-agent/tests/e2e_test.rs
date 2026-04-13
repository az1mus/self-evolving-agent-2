//! SEA Agent 集成测试

use sea_agent::{SeaAgent, SeaConfig};
use tempfile::TempDir;

#[tokio::test]
async fn test_sea_agent_lifecycle() {
    // 创建临时目录
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // 创建配置
    let mut config = SeaConfig::default();
    config.session_store_path = temp_dir.path().to_path_buf();

    // 创建 SEA Agent
    let agent = SeaAgent::new(config).await.expect("Failed to create agent");

    // 创建 Session
    let session_id = agent
        .create_session()
        .await
        .expect("Failed to create session");

    assert!(!session_id.to_string().is_empty());

    // 列出 Sessions
    let sessions = agent.list_sessions().await.expect("Failed to list sessions");
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_id, session_id);

    // 删除 Session
    agent
        .delete_session(session_id)
        .await
        .expect("Failed to delete session");

    let sessions = agent.list_sessions().await.expect("Failed to list sessions");
    assert!(sessions.is_empty());
}

#[tokio::test]
async fn test_sea_agent_server_management() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = SeaConfig::default();
    config.session_store_path = temp_dir.path().to_path_buf();

    let mut agent = SeaAgent::new(config).await.expect("Failed to create agent");
    let session_id = agent.create_session().await.expect("Failed to create session");

    // 注册 Server
    let server_id = agent
        .register_server(
            session_id,
            concrete_servers::factory::ServerType::Echo,
            None,
        )
        .await
        .expect("Failed to register server");

    assert!(server_id.starts_with("echo-"));

    // 列出 Servers
    let servers = agent.list_servers();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].server_type, concrete_servers::factory::ServerType::Echo);
    assert!(!servers[0].running);

    // 启动 Server
    agent
        .start_server(&server_id)
        .await
        .expect("Failed to start server");

    let servers = agent.list_servers();
    assert!(servers[0].running);

    // 停止 Server
    agent
        .stop_server(&server_id)
        .await
        .expect("Failed to stop server");

    let servers = agent.list_servers();
    assert!(!servers[0].running);
}

#[tokio::test]
async fn test_sea_agent_messaging() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = SeaConfig::default();
    config.session_store_path = temp_dir.path().to_path_buf();

    let mut agent = SeaAgent::new(config).await.expect("Failed to create agent");
    let session_id = agent.create_session().await.expect("Failed to create session");

    // 注册并启动 Echo Server
    let server_id = agent
        .register_server(
            session_id,
            concrete_servers::factory::ServerType::Echo,
            None,
        )
        .await
        .expect("Failed to register server");

    agent
        .start_server(&server_id)
        .await
        .expect("Failed to start server");

    // 发送带路由信息的消息（路由指令格式）
    let result = agent
        .send_message(session_id, r#"{"action": "echo", "text": "Hello, world!"}"#)
        .await
        .expect("Failed to send message");

    // 验证消息历史中有用户消息
    let history = agent
        .get_message_history(session_id, None, None)
        .await
        .expect("Failed to get message history");

    assert!(!history.is_empty());
    assert_eq!(history[0].role, session_manager::MessageRole::User);
}

#[tokio::test]
async fn test_sea_agent_init() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = SeaConfig::default();
    config.session_store_path = temp_dir.path().to_path_buf();

    let mut agent = SeaAgent::new(config).await.expect("Failed to create agent");

    // 初始化系统
    let session_id = agent.init().await.expect("Failed to init agent");

    // 验证默认 Servers 已启动
    let servers = agent.list_session_servers(session_id);
    assert!(!servers.is_empty());

    // 至少应该有 Echo, Calculator, Time 三个 Server
    assert!(servers.len() >= 3);

    // 验证所有 Server 都在运行
    for server in &servers {
        assert!(server.running);
    }

    // 关闭
    agent.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
async fn test_available_server_types() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = SeaConfig::default();
    config.session_store_path = temp_dir.path().to_path_buf();
    let agent = SeaAgent::new(config).await.expect("Failed to create agent");

    let types = agent.available_server_types();
    assert!(!types.is_empty());
    assert!(types.contains(&"echo".to_string()));
    assert!(types.contains(&"calculator".to_string()));
    assert!(types.contains(&"time".to_string()));
}
