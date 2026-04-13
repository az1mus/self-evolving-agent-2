# Module 5: Integration & CLI — Todolist

## 设计路线

```
Workspace 配置 → CLI 入口 → Session-Router-Server 整合 → 运行时管理 → 端到端测试
```

核心思路：将前 4 个模块组装成完整的可运行系统，提供统一的 CLI 和配置管理。

---

## Phase 1: Workspace 与配置

- [ ] **1.1 定义 Workspace 配置**
  - `WorkspaceConfig`: { sessions_dir, default_router, servers, logging }
  - 从 `sea-agent.toml` 加载
  - 支持环境变量覆盖
  - 文件: `src/config.rs`

- [ ] **1.2 目录结构管理**
  - `init_workspace(path)`: 初始化工作目录
  - 创建 `sessions/`, `config/`, `logs/` 目录
  - 生成默认配置文件
  - 文件: `src/workspace.rs`

- [ ] **1.3 配置文件示例**
  - `sea-agent.toml`:
    ```toml
    [session]
    dir = "./sessions"
    default_max_hops = 10
    drain_timeout = 300

    [router]
    classifier = "rule-based"
    strategy = "capability"

    [logging]
    level = "info"
    file = "./logs/sea-agent.log"
    ```
  - 文件: `config/default.toml`

- [ ] **1.4 单元测试**
  - 配置加载与解析
  - 工作目录初始化

---

## Phase 2: CLI 入口

- [ ] **2.1 CLI 框架搭建**
  - 使用 `clap` 定义命令结构
  - 主命令: `sea-agent`
  - 子命令:
    - `init` — 初始化工作空间
    - `session` — Session 管理
    - `server` — Server 管理
    - `send` — 发送消息
    - `run` — 启动完整系统
  - 文件: `src/cli.rs`

- [ ] **2.2 Session 子命令**
  - `session create [--config <path>]` — 创建新 Session
  - `session list` — 列出所有 Session
  - `session show <id>` — 显示 Session 详情
  - `session delete <id>` — 删除 Session
  - 文件: `src/cli/session.rs`

- [ ] **2.3 Server 子命令**
  - `server register <session-id> --type <type> [--name <name>]` — 注册 Server
  - `server list <session-id>` — 列出 Session 内 Server
  - `server start <session-id> <server-id>` — 启动 Server
  - `server stop <server-id>` — 停止 Server
  - `server drain <server-id>` — 排空 Server
  - 文件: `src/cli/server.rs`

- [ ] **2.4 消息发送子命令**
  - `send <session-id> --message <json>` — 发送消息
  - `send <session-id> --file <path>` — 从文件读取消息
  - 输出路由结果
  - 文件: `src/cli/send.rs`

- [ ] **2.5 Run 子命令**
  - `run [--config <path>]` — 启动完整系统
  - 交互式 REPL 模式（可选）
  - 文件: `src/cli/run.rs`

---

## Phase 3: 系统整合

- [ ] **3.1 定义 System 结构**
  - `SEA`: 系统主结构
    - `session_manager: SessionManager`
    - `router: Router`
    - `server_runner: ServerRunner`
    - `config: WorkspaceConfig`
  - 文件: `src/system.rs`

- [ ] **3.2 实现 System 初始化**
  - `SEA::new(config)`: 从配置创建系统
  - 加载或创建 Session
  - 注册和启动配置中的 Servers
  - 文件: `src/system.rs`

- [ ] **3.3 实现消息处理流程**
  - `SEA::process_message(session_id, message) -> Result`
  - 流程：
    1. Session 接收消息
    2. Router 判定有机/无机
    3. 预处理（如需要）
    4. 路由到目标 Server
    5. Server 处理并返回
    6. Session 持久化
  - 文件: `src/system.rs`

- [ ] **3.4 实现 Server 管理**
  - `SEA::register_server(session_id, server_type, config)`
  - `SEA::start_server(server_id)`
  - `SEA::stop_server(server_id)`
  - `SEA::drain_server(server_id)`
  - 文件: `src/system.rs`

- [ ] **3.5 集成测试**
  - 完整消息处理流程
  - Server 注册与启动

---

## Phase 4: 运行时管理

- [ ] **4.1 异步运行时**
  - 使用 `tokio` 运行时
  - 每个 Server 作为独立的 tokio 任务
  - Gossip 后台任务
  - 文件: `src/runtime.rs`

- [ ] **4.2 优雅关闭**
  - 信号处理（Ctrl+C）
  - Drain 所有活跃 Server
  - 保存 Session 状态
  - 文件: `src/shutdown.rs`

- [ ] **4.3 健康检查**
  - `health` 命令：检查系统状态
  - 返回各 Server 状态、Session 状态
  - 文件: `src/cli/health.rs`

- [ ] **4.4 日志与监控**
  - 统一日志配置
  - 关键指标输出
  - 文件: `src/logging.rs`

---

## Phase 5: REPL 模式（可选）

- [ ] **5.1 REPL 实现**
  - 使用 `rustyline` 或 `reedline`
  - 支持 Tab 补全
  - 支持历史命令
  - 文件: `src/repl.rs`

- [ ] **5.2 REPL 命令**
  - `:session create / list / show / delete`
  - `:server register / list / start / stop`
  - `:send <message>`
  - `:config show / set`
  - `:quit`
  - 文件: `src/repl/commands.rs`

- [ ] **5.3 REPL 测试**
  - 命令解析与执行

---

## Phase 6: 端到端测试

- [ ] **6.1 完整场景测试**
  - 初始化 → 创建 Session → 注册 Servers → 发送消息 → 查看结果
  - 文件: `tests/e2e_full_test.rs`

- [ ] **6.2 多 Server 协作测试**
  - Code Review 流程（使用 Concrete Servers）
  - Data Pipeline 流程
  - 文件: `tests/e2e_collaboration_test.rs`

- [ ] **6.3 容错测试**
  - Server 崩溃后的恢复
  - Gossip 失效检测
  - 路由降级
  - 文件: `tests/e2e_fault_test.rs`

- [ ] **6.4 性能测试（基准）**
  - 消息处理吞吐量
  - 路由延迟
  - Gossip 同步延迟
  - 文件: `benches/benchmark.rs`

---

## Phase 7: 文档与发布

- [ ] **7.1 README**
  - 安装说明
  - 快速开始
  - 架构说明
  - 文件: `README.md`

- [ ] **7.2 配置参考**
  - 完整配置文件说明
  - 文件: `docs/configuration.md`

- [ ] **7.3 API 文档**
  - rustdoc 生成
  - 文件: 代码注释

- [ ] **7.4 发布准备**
  - Cargo.toml 元数据
  - CI/CD 配置
  - 文件: `.github/workflows/`

---

## 依赖

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.7"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
tokio = { version = "1", features = ["full"] }
clap = { version = "4", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-appender = "0.2"

# 可选：REPL
rustyline = { version = "12", optional = true }

[dependencies.session-manager]
path = "../session-manager"

[dependencies.router-core]
path = "../router-core"

[dependencies.mcp-server-framework]
path = "../mcp-server-framework"

[dependencies.concrete-servers]
path = "../concrete-servers"

[features]
default = []
repl = ["rustyline"]

[[bin]]
name = "sea-agent"
path = "src/main.rs"
```

## 目录结构

```
sea-agent/
├── Cargo.toml
├── config/
│   └── default.toml
├── src/
│   ├── main.rs
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── session.rs
│   │   ├── server.rs
│   │   ├── send.rs
│   │   ├── run.rs
│   │   └── health.rs
│   ├── config.rs
│   ├── workspace.rs
│   ├── system.rs
│   ├── runtime.rs
│   ├── shutdown.rs
│   ├── logging.rs
│   └── repl.rs          # (可选)
├── tests/
│   ├── e2e_full_test.rs
│   ├── e2e_collaboration_test.rs
│   └── e2e_fault_test.rs
├── benches/
│   └── benchmark.rs
└── docs/
    └── configuration.md
```

## 验收标准

1. `sea-agent init` 能初始化工作空间
2. `sea-agent session create` 能创建 Session
3. `sea-agent server register` 能注册 Server
4. `sea-agent send` 能处理消息并返回结果
5. `sea-agent run` 能启动完整系统
6. 所有端到端测试通过
7. Ctrl+C 能优雅关闭并保存状态
