# Module 3: MCP Server Framework — Todolist

## 设计路线

```
MCP 协议基础 → Server Trait 与生命周期 → Gossip 协议 → 拓扑维护 → 消息传递与路由
```

核心思路：先实现 MCP 协议层，再定义 Server 抽象，再实现去中心化拓扑，最后整合消息路由。

---

## Phase 1: MCP 协议基础

- [ ] **1.1 定义 MCP 消息类型**
  - `MCPMessage`: { type, payload, metadata }
  - `MCPMessageType`: enum { Request, Response, Notification }
  - `MCPRequest`: { id, method, params }
  - `MCPResponse`: { id, result, error? }
  - `MCPNotification`: { method, params }
  - 文件: `src/protocol/message.rs`

- [ ] **1.2 定义 MCP 工具结构**
  - `Tool`: { name, description, input_schema }
  - `ToolCall`: { tool_name, arguments }
  - `ToolResult`: { content, is_error? }
  - 文件: `src/protocol/tool.rs`

- [ ] **1.3 MCP 消息编解码**
  - `MCPCodec`: 实现 `Encoder`/`Decoder` trait
  - JSON 序列化/反序列化
  - 文件: `src/protocol/codec.rs`

- [ ] **1.4 单元测试**
  - 消息序列化/反序列化
  - 编解码往返测试

---

## Phase 2: Server Trait 与生命周期

- [ ] **2.1 定义 Server Trait**
  - `MCPServer` trait:
    - `id() -> ServerId`
    - `tools() -> Vec<Tool>`
    - `handle_tool_call(call) -> ToolResult`
    - `on_message(msg) -> Option<MCPMessage>`
  - 文件: `src/server/trait.rs`

- [ ] **2.2 实现 Server 基础结构**
  - `ServerBase`: 提供 ID、工具列表、状态管理
  - `ServerState`: enum { Pending, Active, Draining, Removed }
  - `ServerMeta`: { known_peers, local_tools, routing_table, session_ref }
  - 文件: `src/server/base.rs`

- [ ] **2.3 Server 生命周期管理**
  - `ServerLifecycle` trait:
    - `start() -> Result`
    - `stop() -> Result`
    - `drain() -> Result`
  - `ServerHandle`: 异步运行 Server
  - 文件: `src/server/lifecycle.rs`

- [ ] **2.4 Server 注册到 Session**
  - `register_to_session(server, session_manager)`
  - 更新 Session 的 Server 列表和路由表
  - 文件: `src/server/registration.rs`

- [ ] **2.5 单元测试**
  - Server 创建与工具注册
  - 生命周期状态转换
  - 注册到 Session 的流程

---

## Phase 3: Gossip 协议实现

- [ ] **3.1 定义 Gossip 消息类型**
  - `GossipMessage`: enum
    - `Heartbeat`: { server_id, timestamp }
    - `Join`: { server_id, tools }
    - `Leave`: { server_id }
    - `ToolAnnounce`: { server_id, tools }
    - `TopologySync`: { server_id, known_peers }
    - `Suspect`: { server_id, reporter }
    - `Welcome`: { server_id, known_peers, routing_table }
  - 文件: `src/gossip/message.rs`

- [ ] **3.2 实现 Gossip 消息处理**
  - `GossipHandler`: 处理各类 Gossip 消息
  - `on_heartbeat()`: 更新存活状态
  - `on_join()`: 添加到 known_peers
  - `on_leave()`: 从 known_peers 移除
  - `on_tool_announce()`: 更新路由表
  - 文件: `src/gossip/handler.rs`

- [ ] **3.3 实现 Heartbeat 机制**
  - 后台任务定期发送 Heartbeat
  - 检测 Heartbeat 超时
  - 配置: `heartbeat_interval`, `heartbeat_timeout`
  - 文件: `src/gossip/heartbeat.rs`

- [ ] **3.4 实现失效检测**
  - `FailureDetector`: 检测超时的 Server
  - 状态机: Live → Suspected → Confirmed Dead
  - 配置: `suspect_threshold`, `confirm_threshold`
  - 广播 `Suspect` 消息
  - 文件: `src/gossip/failure.rs`

- [ ] **3.5 实现拓扑同步**
  - 新 Server 加入时广播 `Join`
  - 周期性广播 `TopologySync`
  - 合并来自其他节点的拓扑信息
  - 文件: `src/gossip/topology.rs`

- [ ] **3.6 单元测试**
  - Gossip 消息处理
  - Heartbeat 超时触发失效检测
  - 拓扑同步正确性

---

## Phase 4: 去中心化拓扑维护

- [ ] **4.1 本地状态维护**
  - `LocalTopologyState`: { known_peers, routing_table, version }
  - 版本号用于冲突解决
  - 文件: `src/topology/state.rs`

- [ ] **4.2 拓扑更新逻辑**
  - `add_peer(server_id)`
  - `remove_peer(server_id)`
  - `update_routing_table(capability, server_id)`
  - 触发 Gossip 消息同步
  - 文件: `src/topology/update.rs`

- [ ] **4.3 拓扑查询**
  - `find_server_by_capability(capability) -> Option<ServerId>`
  - `list_peers() -> Vec<ServerId>`
  - `get_server_info(server_id) -> Option<ServerInfo>`
  - 文件: `src/topology/query.rs`

- [ ] **4.4 拓扑一致性检查**
  - 检测孤岛节点
  - 检测分区（可选）
  - 文件: `src/topology/consistency.rs`

- [ ] **4.5 单元测试**
  - 拓扑更新与查询
  - 多节点拓扑同步模拟

---

## Phase 5: 消息传递与路由

- [ ] **5.1 消息发送接口**
  - `send_message(to_server, message) -> Result`
  - 使用 Channel 或网络传输
  - 文件: `src/messaging/send.rs`

- [ ] **5.2 消息接收与分发**
  - `MessageDispatcher`: 接收消息并分发到对应 Handler
  - 支持 MCP 消息和 Gossip 消息
  - 文件: `src/messaging/dispatch.rs`

- [ ] **5.3 本地路由决策**
  - 收到消息后检查本地 `routing_table`
  - 有匹配 → 直接转发
  - 无匹配 → 广播 `QueryCapability`（扩展 Gossip）
  - 文件: `src/messaging/route.rs`

- [ ] **5.4 消息转发**
  - `forward_message(message, to_server)`
  - 更新 `visited_servers` 和 `hop_count`
  - 防循环检查（调用 Router Core）
  - 文件: `src/messaging/forward.rs`

- [ ] **5.5 集成测试**
  - Server A → Server B 消息传递
  - 无本地路由 → 广播查询 → 路由成功

---

## Phase 6: Server 运行时

- [ ] **6.1 Server Runner**
  - `ServerRunner`: 异步运行 Server
  - 启动 Gossip 后台任务
  - 启动消息监听循环
  - 文件: `src/runtime/runner.rs`

- [ ] **6.2 配置管理**
  - `ServerConfig`: { id, session_id, gossip_config, failure_config }
  - 从文件加载配置
  - 文件: `src/runtime/config.rs`

- [ ] **6.3 事件系统**
  - `ServerEvent`: enum { PeerJoined, PeerLeft, ToolAnnounced, ... }
  - `EventBus`: 发布-订阅事件总线
  - 文件: `src/runtime/event.rs`

- [ ] **6.4 日志与监控**
  - 使用 `tracing` 记录关键事件
  - 指标：消息计数、路由延迟、拓扑变更
  - 文件: `src/runtime/metrics.rs`

---

## Phase 7: 独立运行与测试

- [ ] **7.1 CLI 子命令**
  - `server start --config <config.toml>` — 启动 Server
  - `server join <session-id>` — 加入 Session
  - `server leave <server-id>` — 离开 Session
  - `server status <server-id>` — 查看状态
  - 文件: `src/cli.rs`, `src/main.rs`

- [ ] **7.2 示例配置文件**
  - `config/sample_server.toml`
  - 配置 Server ID、Session ID、Gossip 参数
  - 文件: `config/`

- [ ] **7.3 多节点测试**
  - 启动 3 个 Server，验证 Gossip 同步
  - 模拟节点失效，验证检测与移除
  - 文件: `tests/multi_node_test.rs`

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
async-trait = "0.1"
clap = { version = "4", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
tokio-stream = "0.1"

[dependencies.session-manager]
path = "../session-manager"

[dependencies.router-core]
path = "../router-core"
```

## 目录结构

```
mcp-server-framework/
├── Cargo.toml
├── config/
│   └── sample_server.toml
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── lib.rs
│   ├── protocol/
│   │   ├── mod.rs
│   │   ├── message.rs
│   │   ├── tool.rs
│   │   └── codec.rs
│   ├── server/
│   │   ├── mod.rs
│   │   ├── trait.rs
│   │   ├── base.rs
│   │   ├── lifecycle.rs
│   │   └── registration.rs
│   ├── gossip/
│   │   ├── mod.rs
│   │   ├── message.rs
│   │   ├── handler.rs
│   │   ├── heartbeat.rs
│   │   ├── failure.rs
│   │   └── topology.rs
│   ├── topology/
│   │   ├── mod.rs
│   │   ├── state.rs
│   │   ├── update.rs
│   │   ├── query.rs
│   │   └── consistency.rs
│   ├── messaging/
│   │   ├── mod.rs
│   │   ├── send.rs
│   │   ├── dispatch.rs
│   │   ├── route.rs
│   │   └── forward.rs
│   └── runtime/
│       ├── mod.rs
│       ├── runner.rs
│       ├── config.rs
│       ├── event.rs
│       └── metrics.rs
└── tests/
    ├── integration_test.rs
    ├── multi_node_test.rs
    └── fixtures/
        └── sample_server_config.toml
```

## 验收标准

1. `cargo run -- server start` 能启动 Server 并加入 Session
2. 多个 Server 能通过 Gossip 同步拓扑
3. 失效节点能被检测并移除
4. Server 间能正确路由和转发消息
5. 所有单元测试和集成测试通过
