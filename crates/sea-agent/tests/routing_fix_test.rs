use sea_agent::runtime::SeaAgent;
use sea_agent::config::SeaConfig;
use concrete_servers::factory::ServerType;

#[tokio::test]
async fn test_natural_language_routing() {
    // 创建测试配置
    let config = SeaConfig::default();

    // 创建 SEA Agent
    let mut agent = SeaAgent::new(config).await.unwrap();

    // 创建新 session
    let session_id = agent.create_session().await.unwrap();

    // 注册 LLM Gateway
    let llm_id = agent
        .register_server(session_id, ServerType::LLMGateway, Some("test-llm".to_string()))
        .await
        .unwrap();

    // 启动 LLM Gateway
    agent.start_server(&llm_id).await.unwrap();

    // 测试 1: 自然语言消息应该被分类为有机处理并路由到 LLM Gateway
    let result = agent.send_message(session_id, "hi").await.unwrap();
    assert_eq!(result.processing_type, router_core::ProcessingType::Organic);
    assert_eq!(result.routed_servers, vec!["test-llm"]);

    println!("✅ Test passed: Natural language routed to LLM Gateway");
}

#[tokio::test]
async fn test_natural_language_without_llm_gateway() {
    // 创建测试配置
    let config = SeaConfig::default();

    // 创建 SEA Agent
    let mut agent = SeaAgent::new(config).await.unwrap();

    // 创建新 session
    let session_id = agent.create_session().await.unwrap();

    // 注册一个 Echo server（不注册 LLM Gateway）
    let echo_id = agent
        .register_server(session_id, ServerType::Echo, Some("test-echo".to_string()))
        .await
        .unwrap();

    agent.start_server(&echo_id).await.unwrap();

    // 测试 2: 自然语言消息没有 LLM Gateway 时应该失败并返回友好错误
    let result = agent.send_message(session_id, "hi").await;
    assert!(result.is_err());

    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("All routers failed") || error_msg.contains("No capability"));

    println!("✅ Test passed: Natural language without LLM Gateway returns error");
}

#[tokio::test]
async fn test_structured_message_routing() {
    // 创建测试配置
    let config = SeaConfig::default();

    // 创建 SEA Agent
    let mut agent = SeaAgent::new(config).await.unwrap();

    // 创建新 session
    let session_id = agent.create_session().await.unwrap();

    // 注册 Echo server
    let echo_id = agent
        .register_server(session_id, ServerType::Echo, Some("test-echo".to_string()))
        .await
        .unwrap();

    agent.start_server(&echo_id).await.unwrap();

    // 测试 3: 结构化消息应该被分类为无机处理并路由到对应的 server
    let result = agent
        .send_message(session_id, r#"{"action": "echo", "text": "Hello"}"#)
        .await
        .unwrap();

    // 由于有 action 字段，应该被分类为无机处理
    assert_eq!(result.processing_type, router_core::ProcessingType::Inorganic);
    assert_eq!(result.routed_servers, vec!["test-echo"]);

    println!("✅ Test passed: Structured message routed to Echo server");
}
