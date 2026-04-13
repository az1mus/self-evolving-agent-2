//! Router Core 基础示例
//!
//! 演示如何使用 router-core 进行消息路由

use router_core::{
    CapabilityRouter, Message, MessageContent, RouterBuilder, RuleBasedClassifier,
    SessionManager,
};
use session_manager::{ServerInfo, ServerStatus};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("=== Router Core 基础示例 ===\n");

    // 1. 创建 Session 和 Session Manager
    let temp_dir = tempfile::tempdir()?;
    let manager = SessionManager::new(temp_dir.path());
    let mut session = manager.create_session()?;

    println!("✓ 创建 Session: {}", session.session_id);

    // 2. 注册 Servers
    session.servers.insert(
        "echo-server".to_string(),
        ServerInfo {
            id: "echo-server".to_string(),
            status: ServerStatus::Active,
            tools: vec!["echo".to_string()],
            metadata: HashMap::new(),
            draining_since: None,
        },
    );

    session.servers.insert(
        "code-review-server".to_string(),
        ServerInfo {
            id: "code-review-server".to_string(),
            status: ServerStatus::Active,
            tools: vec!["code_review".to_string()],
            metadata: HashMap::new(),
            draining_since: None,
        },
    );

    // 3. 设置路由表
    session
        .routing_table
        .insert("capability:echo".to_string(), "echo-server".to_string());
    session
        .routing_table
        .insert("capability:code_review".to_string(), "code-review-server".to_string());

    manager.save_session(&session)?;
    println!("✓ 注册 Servers 和路由表\n");

    // 4. 创建 Router Core
    let router_core = RouterBuilder::new()
        .classifier(Box::new(RuleBasedClassifier::new()))
        .router(Box::new(CapabilityRouter::new()))
        .build()?;

    println!("✓ 创建 Router Core\n");

    // 5. 测试消息路由 - 路由指令(无机处理)
    println!("--- 测试 1: 路由指令 (无机处理) ---");
    let msg1 = Message::simple(
        session.session_id,
        MessageContent::routing_command("echo"),
    );

    let servers = router_core.process(msg1, &session).await?;
    println!("消息路由到: {:?}", servers);
    println!("预期: [\"echo-server\"]\n");

    // 6. 测试消息路由 - 结构化内容(无机处理)
    println!("--- 测试 2: 结构化内容 (无机处理) ---");
    let msg2 = Message::simple(
        session.session_id,
        MessageContent::structured(serde_json::json!({
            "action": "code_review",
            "code": "fn main() { println!(\"hello\"); }"
        })),
    );

    let servers = router_core.process(msg2, &session).await?;
    println!("消息路由到: {:?}", servers);
    println!("预期: [\"code-review-server\"]\n");

    // 7. 测试消息路由 - 非结构化内容(有机处理)
    println!("--- 测试 3: 非结构化内容 (有机处理) ---");
    let msg3 = Message::simple(
        session.session_id,
        MessageContent::unstructured("请帮我审查这段代码"),
    );

    match router_core.process(msg3, &session).await {
        Ok(_) => println!("路由成功"),
        Err(e) => println!("路由失败 (预期): {}", e),
    }
    println!("预期: 失败 - No capability found\n");

    println!("=== 示例完成 ===");
    Ok(())
}