use session_manager::{CacheManager, ServerLifecycle, SessionManager};
use std::collections::HashMap;
use tempfile::TempDir;

/// 完整的端到端集成测试
#[test]
fn test_full_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path());

    // 1. 创建 Session
    let session = manager.create_session().unwrap();
    let session_id = session.session_id;

    // 2. 注册并激活 Server
    let lifecycle = ServerLifecycle::new(&manager);
    lifecycle
        .register_server(
            session_id,
            "code-reviewer".to_string(),
            vec![
                "review_code".to_string(),
                "suggest_improvements".to_string(),
            ],
            HashMap::new(),
        )
        .unwrap();

    lifecycle
        .activate_server(session_id, "code-reviewer")
        .unwrap();

    // 3. 添加路由
    lifecycle
        .add_route(
            session_id,
            "capability:code_review",
            "code-reviewer".to_string(),
        )
        .unwrap();

    // 4. 添加消息历史
    manager
        .add_message(
            session_id,
            session_manager::MessageRole::User,
            "Review this code".to_string(),
        )
        .unwrap();

    manager
        .add_message(
            session_id,
            session_manager::MessageRole::Assistant,
            "I found 3 issues".to_string(),
        )
        .unwrap();

    // 5. 使用缓存
    let cache_mgr = CacheManager::new(&manager);
    let hash = CacheManager::hash_content("some input");
    cache_mgr
        .set_input_cache(session_id, &hash, serde_json::json!({"result": "cached"}))
        .unwrap();

    // 6. 验证 Session 状态
    let loaded = manager.load_session(session_id).unwrap();
    assert_eq!(loaded.servers.len(), 1);
    assert_eq!(loaded.message_history.len(), 2);
    assert_eq!(loaded.routing_table.len(), 1);
    assert_eq!(loaded.cache.input_cache.len(), 1);

    // 7. Drain 并移除 Server
    lifecycle.drain_server(session_id, "code-reviewer").unwrap();
    lifecycle
        .remove_server(session_id, "code-reviewer")
        .unwrap();

    // 8. 清理缓存
    cache_mgr.invalidate_cache(session_id, None).unwrap();

    // 9. 终止 Session
    manager.terminate_session(session_id).unwrap();

    let final_session = manager.load_session(session_id).unwrap();
    assert_eq!(
        final_session.state,
        session_manager::SessionState::Terminated
    );
}

/// 测试多个 Server 协作场景
#[test]
fn test_multiple_servers() {
    let temp_dir = TempDir::new().unwrap();
    let manager = SessionManager::new(temp_dir.path());
    let session = manager.create_session().unwrap();
    let session_id = session.session_id;

    let lifecycle = ServerLifecycle::new(&manager);

    // 注册多个 servers
    for (server_id, tools) in [
        ("analyzer", vec!["analyze".to_string()]),
        ("reviewer", vec!["review".to_string()]),
        ("formatter", vec!["format".to_string()]),
    ] {
        lifecycle
            .register_server(session_id, server_id.to_string(), tools, HashMap::new())
            .unwrap();
        lifecycle.activate_server(session_id, server_id).unwrap();
    }

    // 配置路由
    lifecycle
        .add_route(session_id, "cap:analyze", "analyzer".to_string())
        .unwrap();
    lifecycle
        .add_route(session_id, "cap:review", "reviewer".to_string())
        .unwrap();
    lifecycle
        .add_route(session_id, "cap:format", "formatter".to_string())
        .unwrap();

    // 验证路由查找
    assert_eq!(
        lifecycle.lookup_route(session_id, "cap:analyze").unwrap(),
        Some("analyzer".to_string())
    );
    assert_eq!(
        lifecycle.lookup_route(session_id, "cap:review").unwrap(),
        Some("reviewer".to_string())
    );

    // 移除一个 server，验证路由自动清理
    lifecycle.deregister_server(session_id, "reviewer").unwrap();

    let session = manager.load_session(session_id).unwrap();
    assert_eq!(session.servers.len(), 2); // analyzer & formatter still there
    assert_eq!(session.routing_table.len(), 2); // only 2 routes remain
    assert!(session.routing_table.get("cap:review").is_none());
}
