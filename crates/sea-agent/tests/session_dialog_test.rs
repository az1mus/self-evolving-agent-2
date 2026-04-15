//! Session 自然语言对话能力测试
//!
//! 测试 SEA 创建 session 并与之用自然语言对话的能力

use sea_agent::{SeaAgent, SeaConfig};
use tempfile::TempDir;
use concrete_servers::factory::ServerType;

/// 创建测试用的 SEA Agent
async fn create_test_agent() -> (TempDir, SeaAgent) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = SeaConfig::default();
    config.session_store_path = temp_dir.path().to_path_buf();

    let agent = SeaAgent::new(config).await.expect("Failed to create agent");
    (temp_dir, agent)
}

#[tokio::test]
async fn test_create_session_with_natural_language_interaction() {
    let (_temp_dir, mut agent) = create_test_agent().await;

    // 创建 Session
    let session_id = agent
        .create_session()
        .await
        .expect("Failed to create session");

    // 注册 LLM Gateway Server
    let llm_id = agent
        .register_server(session_id, ServerType::LLMGateway, None)
        .await
        .expect("Failed to register LLM Gateway");

    agent
        .start_server(&llm_id)
        .await
        .expect("Failed to start LLM Gateway");

    // 发送自然语言消息
    let result = agent
        .send_message(session_id, "你好，请帮我分析这段代码的质量")
        .await
        .expect("Failed to send natural language message");

    // 验证消息被处理
    assert!(!result.response.is_empty());

    // 获取消息历史
    let history = agent
        .get_message_history(session_id, None, None)
        .await
        .expect("Failed to get message history");

    // 验证历史中有用户和助手消息
    assert!(!history.is_empty());
    assert_eq!(history[0].role, session_manager::MessageRole::User);
    assert!(history[0].content.contains("你好"));

    println!("✅ Session 自然语言对话测试通过");
}

#[tokio::test]
async fn test_multi_turn_conversation() {
    let (_temp_dir, mut agent) = create_test_agent().await;
    let session_id = agent.create_session().await.expect("Failed to create session");

    // 注册 LLM Gateway
    let llm_id = agent
        .register_server(session_id, ServerType::LLMGateway, None)
        .await
        .expect("Failed to register LLM Gateway");

    agent.start_server(&llm_id).await.expect("Failed to start LLM Gateway");

    // 多轮对话
    let messages = vec![
        "什么是 Rust 语言？",
        "它有什么特点？",
        "能给我举个例子吗？",
    ];

    for msg in messages {
        let result = agent
            .send_message(session_id, msg)
            .await
            .expect("Failed to send message");

        assert!(!result.response.is_empty(), "Response should not be empty for: {}", msg);
    }

    // 验证消息历史中有 3 轮对话
    let history = agent
        .get_message_history(session_id, None, None)
        .await
        .expect("Failed to get message history");

    // 每轮对话有 user + assistant 消息
    assert!(history.len() >= 6, "Should have at least 6 messages (3 user + 3 assistant)");
    assert_eq!(history[0].role, session_manager::MessageRole::User);
    assert_eq!(history[1].role, session_manager::MessageRole::Assistant);

    println!("✅ 多轮对话测试通过");
}

#[tokio::test]
async fn test_structured_message_with_natural_language() {
    let (_temp_dir, mut agent) = create_test_agent().await;
    let session_id = agent.create_session().await.expect("Failed to create session");

    // 注册 Echo Server
    let echo_id = agent
        .register_server(session_id, ServerType::Echo, None)
        .await
        .expect("Failed to register Echo Server");

    agent.start_server(&echo_id).await.expect("Failed to start Echo Server");

    // 发送结构化 JSON 消息
    let json_message = r#"{
        "action": "echo",
        "text": "这是一段自然语言文本"
    }"#;

    let result = agent
        .send_message(session_id, json_message)
        .await
        .expect("Failed to send structured message");

    assert!(!result.response.is_empty());

    // 验证路由信息
    assert!(!result.routed_servers.is_empty(), "Should be routed to at least one server");

    println!("✅ 结构化消息与自然语言混合测试通过");
}

#[tokio::test]
async fn test_session_persistence_with_messages() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut config = SeaConfig::default();
    config.session_store_path = temp_dir.path().to_path_buf();

    // 第一个 Agent 实例
    let mut agent1 = SeaAgent::new(config.clone()).await.expect("Failed to create agent1");
    let session_id = agent1.create_session().await.expect("Failed to create session");

    // 注册 LLM Gateway
    let llm_id = agent1
        .register_server(session_id, ServerType::LLMGateway, None)
        .await
        .expect("Failed to register LLM Gateway");

    agent1.start_server(&llm_id).await.expect("Failed to start LLM Gateway");

    // 发送消息
    agent1
        .send_message(session_id, "这是第一条消息")
        .await
        .expect("Failed to send message");

    agent1
        .send_message(session_id, "这是第二条消息")
        .await
        .expect("Failed to send message");

    // 关闭第一个 Agent
    agent1.shutdown().await.expect("Failed to shutdown agent1");

    // 创建新的 Agent 实例（模拟重启）
    let agent2 = SeaAgent::new(config.clone()).await.expect("Failed to create agent2");

    // 验证消息历史仍然存在
    let history = agent2
        .get_message_history(session_id, None, None)
        .await
        .expect("Failed to get message history");

    assert!(history.len() >= 4, "Should have at least 4 messages (2 user + 2 assistant)");

    println!("✅ Session 持久化与消息历史测试通过");
}

#[tokio::test]
async fn test_mixed_content_types_in_session() {
    let (_temp_dir, mut agent) = create_test_agent().await;
    let session_id = agent.create_session().await.expect("Failed to create session");

    // 注册多种 Server
    let llm_id = agent
        .register_server(session_id, ServerType::LLMGateway, None)
        .await
        .expect("Failed to register LLM Gateway");

    let echo_id = agent
        .register_server(session_id, ServerType::Echo, None)
        .await
        .expect("Failed to register Echo Server");

    agent.start_server(&llm_id).await.expect("Failed to start LLM Gateway");
    agent.start_server(&echo_id).await.expect("Failed to start Echo Server");

    // 发送混合类型的消息
    // 1. 自然语言消息
    agent
        .send_message(session_id, "请帮我分析这个问题")
        .await
        .expect("Failed to send natural language message");

    // 2. 结构化 JSON 消息
    agent
        .send_message(session_id, r#"{"action": "echo", "text": "test"}"#)
        .await
        .expect("Failed to send structured message");

    // 3. 带路由指令的消息
    agent
        .send_message(session_id, r#"{"capability": "echo", "data": "hello"}"#)
        .await
        .expect("Failed to send routing message");

    // 验证所有消息都被正确处理
    let history = agent
        .get_message_history(session_id, None, None)
        .await
        .expect("Failed to get message history");

    assert!(history.len() >= 6, "Should have at least 6 messages");

    println!("✅ 混合内容类型测试通过");
}

#[tokio::test]
async fn test_message_history_pagination() {
    let (_temp_dir, mut agent) = create_test_agent().await;
    let session_id = agent.create_session().await.expect("Failed to create session");

    // 注册 LLM Gateway
    let llm_id = agent
        .register_server(session_id, ServerType::LLMGateway, None)
        .await
        .expect("Failed to register LLM Gateway");

    agent.start_server(&llm_id).await.expect("Failed to start LLM Gateway");

    // 发送多条消息
    for i in 1..=10 {
        agent
            .send_message(session_id, &format!("消息 {}", i))
            .await
            .expect("Failed to send message");
    }

    // 测试分页功能
    // 获取前 5 条
    let first_page = agent
        .get_message_history(session_id, Some(5), Some(0))
        .await
        .expect("Failed to get first page");

    assert!(first_page.len() <= 5);

    // 获取后 5 条
    let second_page = agent
        .get_message_history(session_id, Some(5), Some(5))
        .await
        .expect("Failed to get second page");

    assert!(second_page.len() <= 5);

    // 获取全部
    let all_messages = agent
        .get_message_history(session_id, None, None)
        .await
        .expect("Failed to get all messages");

    assert!(all_messages.len() >= 20, "Should have at least 20 messages (10 user + 10 assistant)");

    println!("✅ 消息历史分页测试通过");
}
