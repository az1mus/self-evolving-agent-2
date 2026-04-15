//! LLM Gateway 自然语言交互测试
//!
//! 测试 LLM Gateway Server 的自然语言处理能力

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
async fn test_llm_gateway_basic_chat() {
    let (_temp_dir, mut agent) = create_test_agent().await;
    let session_id = agent.create_session().await.expect("Failed to create session");

    // 注册并启动 LLM Gateway
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
        .send_message(session_id, "你好，请介绍一下你自己")
        .await
        .expect("Failed to send message");

    // 验证响应
    assert!(!result.response.is_empty());

    // 由于没有真实的 API Key，应该返回 Mock 响应
    let history = agent
        .get_message_history(session_id, None, None)
        .await
        .expect("Failed to get history");

    assert!(!history.is_empty());

    println!("✅ LLM Gateway 基础对话测试通过");
}

#[tokio::test]
async fn test_llm_gateway_code_analysis() {
    let (_temp_dir, mut agent) = create_test_agent().await;
    let session_id = agent.create_session().await.expect("Failed to create session");

    // 注册并启动 LLM Gateway
    let llm_id = agent
        .register_server(session_id, ServerType::LLMGateway, None)
        .await
        .expect("Failed to register LLM Gateway");

    agent.start_server(&llm_id).await.expect("Failed to start LLM Gateway");

    // 发送代码分析请求
    let code_snippet = r#"
        fn calculate_sum(a: i32, b: i32) -> i32 {
            a + b
        }
    "#;

    let message = format!("请分析这段 Rust 代码的质量和潜在问题：\n{}", code_snippet);

    let result = agent
        .send_message(session_id, &message)
        .await
        .expect("Failed to send code analysis request");

    assert!(!result.response.is_empty());

    println!("✅ LLM Gateway 代码分析测试通过");
}

#[tokio::test]
async fn test_llm_gateway_multi_language_support() {
    let (_temp_dir, mut agent) = create_test_agent().await;
    let session_id = agent.create_session().await.expect("Failed to create session");

    // 注册并启动 LLM Gateway
    let llm_id = agent
        .register_server(session_id, ServerType::LLMGateway, None)
        .await
        .expect("Failed to register LLM Gateway");

    agent.start_server(&llm_id).await.expect("Failed to start LLM Gateway");

    // 测试不同语言的输入
    let messages = vec![
        "Hello, how are you?",                                    // English
        "你好，今天天气怎么样？",                                  // Chinese
        "Bonjour, comment allez-vous?",                          // French
        "Hola, ¿cómo estás?",                                    // Spanish
        "Привет, как дела?",                                     // Russian
    ];

    for msg in messages {
        let result = agent
            .send_message(session_id, msg)
            .await
            .expect(&format!("Failed to send message: {}", msg));

        assert!(!result.response.is_empty(), "Response should not be empty for: {}", msg);
    }

    println!("✅ LLM Gateway 多语言支持测试通过");
}

#[tokio::test]
async fn test_llm_gateway_with_context() {
    let (_temp_dir, mut agent) = create_test_agent().await;
    let session_id = agent.create_session().await.expect("Failed to create session");

    // 注册并启动 LLM Gateway
    let llm_id = agent
        .register_server(session_id, ServerType::LLMGateway, None)
        .await
        .expect("Failed to register LLM Gateway");

    agent.start_server(&llm_id).await.expect("Failed to start LLM Gateway");

    // 建立对话上下文
    agent
        .send_message(session_id, "我正在开发一个 Rust 项目")
        .await
        .expect("Failed to send first message");

    agent
        .send_message(session_id, "项目是一个 Web 服务器")
        .await
        .expect("Failed to send second message");

    agent
        .send_message(session_id, "你能给我一些架构建议吗？")
        .await
        .expect("Failed to send third message");

    // 验证对话历史
    let history = agent
        .get_message_history(session_id, None, None)
        .await
        .expect("Failed to get history");

    // 应该有至少 6 条消息（3 user + 3 assistant）
    assert!(history.len() >= 6);

    println!("✅ LLM Gateway 上下文对话测试通过");
}

#[tokio::test]
async fn test_llm_gateway_structured_request() {
    let (_temp_dir, mut agent) = create_test_agent().await;
    let session_id = agent.create_session().await.expect("Failed to create session");

    // 注册并启动 LLM Gateway
    let llm_id = agent
        .register_server(session_id, ServerType::LLMGateway, None)
        .await
        .expect("Failed to register LLM Gateway");

    agent.start_server(&llm_id).await.expect("Failed to start LLM Gateway");

    // 发送结构化 JSON 请求到 LLM Gateway
    let structured_message = r#"{
        "action": "complete",
        "prompt": "请用 Rust 写一个函数来计算斐波那契数列",
        "options": {
            "max_tokens": 1024
        }
    }"#;

    let result = agent
        .send_message(session_id, structured_message)
        .await
        .expect("Failed to send structured request");

    assert!(!result.response.is_empty());

    println!("✅ LLM Gateway 结构化请求测试通过");
}

#[tokio::test]
async fn test_llm_gateway_error_recovery() {
    let (_temp_dir, mut agent) = create_test_agent().await;
    let session_id = agent.create_session().await.expect("Failed to create session");

    // 注册并启动 LLM Gateway
    let llm_id = agent
        .register_server(session_id, ServerType::LLMGateway, None)
        .await
        .expect("Failed to register LLM Gateway");

    agent.start_server(&llm_id).await.expect("Failed to start LLM Gateway");

    // 发送一个正常消息
    agent
        .send_message(session_id, "这是一条正常消息")
        .await
        .expect("Failed to send normal message");

    // 发送一个可能触发错误的消息（空消息或无效格式）
    // 注意：系统应该优雅地处理错误而不是崩溃
    let result = agent.send_message(session_id, "").await;
    // 空消息应该被接受或返回错误，但不应该崩溃

    // 发送另一个正常消息验证系统仍然可用
    agent
        .send_message(session_id, "系统仍然正常工作吗？")
        .await
        .expect("System should still work after potential error");

    println!("✅ LLM Gateway 错误恢复测试通过");
}

#[tokio::test]
async fn test_llm_gateway_with_other_servers() {
    let (_temp_dir, mut agent) = create_test_agent().await;
    let session_id = agent.create_session().await.expect("Failed to create session");

    // 注册多个 Server
    let llm_id = agent
        .register_server(session_id, ServerType::LLMGateway, None)
        .await
        .expect("Failed to register LLM Gateway");

    let echo_id = agent
        .register_server(session_id, ServerType::Echo, None)
        .await
        .expect("Failed to register Echo Server");

    let calc_id = agent
        .register_server(session_id, ServerType::Calculator, None)
        .await
        .expect("Failed to register Calculator Server");

    agent.start_server(&llm_id).await.expect("Failed to start LLM Gateway");
    agent.start_server(&echo_id).await.expect("Failed to start Echo");
    agent.start_server(&calc_id).await.expect("Failed to start Calculator");

    // 发送自然语言消息（LLM Gateway 处理）
    agent
        .send_message(session_id, "请帮我分析这个问题")
        .await
        .expect("Failed to send to LLM");

    // 发送计算请求（Calculator 处理）
    agent
        .send_message(session_id, r#"{"action": "add", "a": 10, "b": 20}"#)
        .await
        .expect("Failed to send to Calculator");

    // 发送 Echo 请求（Echo 处理）
    agent
        .send_message(session_id, r#"{"action": "echo", "text": "Hello"}"#)
        .await
        .expect("Failed to send to Echo");

    // 验证所有消息都被正确处理
    let history = agent
        .get_message_history(session_id, None, None)
        .await
        .expect("Failed to get history");

    assert!(history.len() >= 6, "Should have at least 6 messages");

    println!("✅ LLM Gateway 与其他 Server 协作测试通过");
}

#[tokio::test]
async fn test_llm_gateway_long_conversation() {
    let (_temp_dir, mut agent) = create_test_agent().await;
    let session_id = agent.create_session().await.expect("Failed to create session");

    // 注册并启动 LLM Gateway
    let llm_id = agent
        .register_server(session_id, ServerType::LLMGateway, None)
        .await
        .expect("Failed to register LLM Gateway");

    agent.start_server(&llm_id).await.expect("Failed to start LLM Gateway");

    // 模拟长对话
    for i in 1..=20 {
        let message = format!("第 {} 条消息：请介绍一下 Rust 的特性", i);
        agent
            .send_message(session_id, &message)
            .await
            .expect(&format!("Failed to send message {}", i));
    }

    // 验证消息历史完整性
    let history = agent
        .get_message_history(session_id, None, None)
        .await
        .expect("Failed to get history");

    assert!(history.len() >= 40, "Should have at least 40 messages (20 user + 20 assistant)");

    // 测试分页查询
    let first_10 = agent
        .get_message_history(session_id, Some(10), Some(0))
        .await
        .expect("Failed to get first 10 messages");

    assert!(first_10.len() <= 10);

    let last_10 = agent
        .get_message_history(session_id, Some(10), Some(30))
        .await
        .expect("Failed to get last 10 messages");

    assert!(last_10.len() <= 10);

    println!("✅ LLM Gateway 长对话测试通过");
}

#[tokio::test]
async fn test_llm_gateway_question_answering() {
    let (_temp_dir, mut agent) = create_test_agent().await;
    let session_id = agent.create_session().await.expect("Failed to create session");

    // 注册并启动 LLM Gateway
    let llm_id = agent
        .register_server(session_id, ServerType::LLMGateway, None)
        .await
        .expect("Failed to register LLM Gateway");

    agent.start_server(&llm_id).await.expect("Failed to start LLM Gateway");

    // 测试不同类型的问题
    let questions = vec![
        "什么是内存安全？",
        "Rust 的所有权系统是如何工作的？",
        "解释一下借用和引用的区别？",
        "什么是生命周期注解？",
        "Rust 中的 Option 和 Result 类型有什么作用？",
    ];

    for question in questions {
        let result = agent
            .send_message(session_id, question)
            .await
            .expect(&format!("Failed to send question: {}", question));

        assert!(!result.response.is_empty());
    }

    println!("✅ LLM Gateway 问答测试通过");
}

#[tokio::test]
async fn test_llm_gateway_code_generation() {
    let (_temp_dir, mut agent) = create_test_agent().await;
    let session_id = agent.create_session().await.expect("Failed to create session");

    // 注册并启动 LLM Gateway
    let llm_id = agent
        .register_server(session_id, ServerType::LLMGateway, None)
        .await
        .expect("Failed to register LLM Gateway");

    agent.start_server(&llm_id).await.expect("Failed to start LLM Gateway");

    // 测试代码生成请求
    let requests = vec![
        "请写一个 Rust 函数来反转字符串",
        "实现一个简单的链表数据结构",
        "写一个函数来检查字符串是否是回文",
        "实现一个二分查找算法",
        "写一个函数来计算两个日期之间的天数差",
    ];

    for request in requests {
        let result = agent
            .send_message(session_id, request)
            .await
            .expect(&format!("Failed to send request: {}", request));

        assert!(!result.response.is_empty());
    }

    println!("✅ LLM Gateway 代码生成测试通过");
}
