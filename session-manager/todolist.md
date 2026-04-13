# Module 1: Session Manager — Todolist

## 设计路线

```
session.json (文件) → SessionStore (内存+文件) → SessionManager (API) → Server 生命周期管理
```

核心思路：先定义数据结构，再实现文件持久化，再封装管理 API，最后加入 Server 生命周期。

---

## Phase 1: 数据结构与文件持久化

- [ ] **1.1 定义 Session 数据结构**
  - `SessionId`: UUID v4
  - `SessionState`: enum { Active, Paused, Terminated }
  - `ServerInfo`: { id, status, tools, metadata }
  - `ServerStatus`: enum { Pending, Active, Draining, Removed }
  - `MessageRecord`: { role, content, timestamp, metadata }
  - `CacheStore`: { input_cache, inference_cache }
  - `SessionConfig`: { max_hops, drain_timeout, ... }
  - `Session`: 聚合根，包含以上所有字段
  - 文件: `src/models.rs`

- [ ] **1.2 实现 Serialize/Deserialize**
  - 为所有数据结构实现 `serde::Serialize`, `serde::Deserialize`
  - 自定义 `ServerStatus` 的序列化（字符串形式）
  - 文件: `src/models.rs`

- [ ] **1.3 实现 Session 文件读写**
  - `SessionStore` trait: `load(path)`, `save(path, session)`, `delete(path)`
  - `JsonSessionStore` 实现：基于 serde_json
  - 文件锁（简单文件锁，防并发写入）
  - 文件: `src/store.rs`

- [ ] **1.4 单元测试**
  - Session 创建与序列化
  - 文件读写往返测试
  - 边界情况：空 Session、大消息历史

---

## Phase 2: Session Manager API

- [ ] **2.1 实现 SessionManager**
  - `create_session() -> Session`
  - `load_session(id) -> Option<Session>`
  - `save_session(session) -> Result`
  - `list_sessions() -> Vec<SessionSummary>`
  - `delete_session(id) -> Result`
  - `terminate_session(id) -> Result` (标记 Terminated)
  - 文件: `src/manager.rs`

- [ ] **2.2 实现消息历史管理**
  - `add_message(session_id, role, content)`
  - `get_messages(session_id, limit, offset)`
  - `clear_messages(session_id)`
  - 文件: `src/manager.rs`

- [ ] **2.3 实现 Cache 管理**
  - `get_input_cache(session_id, hash) -> Option`
  - `set_input_cache(session_id, hash, value)`
  - `get_inference_cache(session_id, key) -> Option`
  - `set_inference_cache(session_id, key, value)`
  - `invalidate_cache(session_id, key?)`
  - 文件: `src/cache.rs`

- [ ] **2.4 单元测试**
  - Session CRUD 全流程
  - 消息添加与查询
  - Cache 存取与失效

---

## Phase 3: Server 生命周期管理

- [ ] **3.1 Server 注册与注销**
  - `register_server(session_id, server_info) -> Result`
  - `deregister_server(session_id, server_id) -> Result`
  - `update_server_status(session_id, server_id, new_status)`
  - 自动更新 `routing_table`
  - 文件: `src/lifecycle.rs`

- [ ] **3.2 Server 状态转换**
  - Pending → Active: `activate_server()`
  - Active → Draining: `drain_server()` (开始排空)
  - Draining → Removed: `remove_server()` (超时或手动)
  - 状态校验：非法转换返回错误
  - 文件: `src/lifecycle.rs`

- [ ] **3.3 Drain 超时机制**
  - 记录进入 Draining 的时间
  - 后台任务检查超时的 Draining Server
  - 超时后强制标记为 Removed
  - 可配置 `drain_timeout`
  - 文件: `src/lifecycle.rs`

- [ ] **3.4 路由表管理**
  - `add_route(session_id, capability, server_id)`
  - `remove_route(session_id, capability)`
  - `lookup_route(session_id, capability) -> Option<ServerId>`
  - Server 注销时自动清理路由
  - 文件: `src/routing_table.rs`

- [ ] **3.5 单元测试**
  - Server 注册/注销流程
  - 状态转换正确性
  - Drain 超时触发
  - 路由表 CRUD 与级联删除

---

## Phase 4: 独立运行入口

- [ ] **4.1 CLI 子命令**
  - `session create` — 创建新 Session
  - `session list` — 列出所有 Session
  - `session show <id>` — 显示 Session 详情
  - `session delete <id>` — 删除 Session
  - `session server register <id> <server-info>` — 注册 Server
  - `session server list <id>` — 列出 Session 内的 Server
  - 文件: `src/cli.rs`, `src/main.rs`

- [ ] **4.2 端到端测试**
  - CLI 创建 → 注册 Server → 查看 → 删除
  - 文件持久化验证

---

## 依赖

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
tokio = { version = "1", features = ["full"] }
clap = { version = "4", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
sha2 = "0.10"  # for cache hash
```

## 目录结构

```
session-manager/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── lib.rs
│   ├── models.rs
│   ├── store.rs
│   ├── manager.rs
│   ├── cache.rs
│   ├── lifecycle.rs
│   └── routing_table.rs
└── tests/
    ├── integration_test.rs
    └── fixtures/
        └── sample_session.json
```

## 验收标准

1. `cargo run -- session create` 能创建 session.json
2. `cargo run -- session show <id>` 能显示详情
3. Server 生命周期状态转换正确
4. Drain 超时后 Server 被强制移除
5. 所有单元测试通过
