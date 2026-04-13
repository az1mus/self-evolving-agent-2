# MCP Server Framework

MCP Server 框架,提供 Server 基础设施、Gossip 协议和去中心化拓扑维护。

## 功能特性

### Phase 1: MCP 协议基础
- ✅ MCP 消息类型 (Request/Response/Notification)
- ✅ MCP 工具结构 (Tool/ToolCall/ToolResult)
- ✅ MCP 编解码器 (JSON 序列化)
- ✅ 完整单元测试

### Phase 2: Server Trait 与生命周期
- ✅ MCPServer trait 定义
- ✅ ServerBase 基础结构
- ✅ ServerLifecycle 生命周期管理
- ✅ Server 注册到 Session
- ✅ 完整单元测试

### Phase 3: Gossip 协议
- ✅ Gossip 消息类型定义
- ✅ GossipHandler 消息处理器
- ✅ Heartbeat 机制
- ✅ FailureDetector 失效检测
- ✅ TopologySync 拓扑同步
- ✅ 完整单元测试

### Phase 4: 去中心化拓扑维护
- ✅ LocalTopologyState 状态维护
- ✅ 拓扑更新逻辑
- ✅ 拓扑查询接口
- ✅ 拓扑一致性检查
- ✅ 完整单元测试

### Phase 5: 消息传递与路由
- ✅ 消息发送接口
- ✅ MessageDispatcher 消息分发
- ✅ 本地路由决策
- ✅ 消息转发逻辑
- ✅ 集成测试

### Phase 6: Server 运行时
- ✅ ServerRunner 运行器
- ✅ ServerConfig 配置管理
- ✅ EventBus 事件系统
- ✅ Metrics 监控指标

### Phase 7: CLI 与独立运行
- ✅ CLI 子命令 (start/join/leave/status)
- ✅ 示例配置文件
- ✅ Echo Server 示例

## 使用方法

### CLI 命令

```bash
# 启动 Server (使用默认配置)
cargo run -p mcp-server-framework -- start

# 启动 Server (指定配置文件)
cargo run -p mcp-server-framework -- start --config config/sample_server.toml

# 启动 Server (指定 ID)
cargo run -p mcp-server-framework -- start --server-id <ID> --session-id <SESSION>

# 加入 Session
cargo run -p mcp-server-framework -- join --session <SESSION_ID>

# 离开 Session
cargo run -p mcp-server-framework -- leave --server-id <ID>

# 查看状态
cargo run -p mcp-server-framework -- status --server-id <ID>
```

### 代码示例

```rust
use mcp_server_framework::{MCPServer, ServerRunner, ServerConfig, Tool, ToolCall, ToolResult};
use async_trait::async_trait;

// 实现 MCPServer trait
struct MyServer {
    id: ServerId,
}

#[async_trait]
impl MCPServer for MyServer {
    fn id(&self) -> ServerId {
        self.id.clone()
    }

    fn tools(&self) -> Vec<Tool> {
        vec![Tool::new("my_tool", "My custom tool")]
    }

    async fn handle_tool_call(&self, call: ToolCall) -> ToolResult {
        // 处理工具调用
        ToolResult::text("Result")
    }

    async fn on_message(&self, msg: MCPMessage) -> Option<MCPMessage> {
        // 处理消息
        None
    }
}

// 启动 Server
#[tokio::main]
async fn main() {
    let config = ServerConfig::default();
    let server = MyServer { id: config.server_id.clone() };
    let mut runner = ServerRunner::new(server, config);

    runner.start().await.unwrap();

    // 运行...
    tokio::signal::ctrl_c().await.unwrap();

    runner.stop().await.unwrap();
}
```

## 架构设计

```
┌─────────────────────────────────────────┐
│            Server Runner                │
│  (管理 Server 生命周期)                   │
└─────────────────────────────────────────┘
                    │
        ┌───────────┼───────────┐
        │           │           │
        ▼           ▼           ▼
┌──────────┐ ┌──────────┐ ┌──────────┐
│  Server  │ │  Gossip  │ │ Messaging│
│   Base   │ │  Handler │ │Dispatcher│
└──────────┘ └──────────┘ └──────────┘
        │           │           │
        └───────────┼───────────┘
                    ▼
        ┌───────────────────────┐
        │   Local Topology      │
        │   (State & Routing)   │
        └───────────────────────┘
```

## 核心组件

### 1. MCP 协议层 (`protocol/`)
- `message.rs`: MCP 消息定义
- `tool.rs`: 工具定义
- `codec.rs`: 编解码器

### 2. Server 层 (`server/`)
- `trait.rs`: MCPServer trait
- `base.rs`: Server 基础结构
- `lifecycle.rs`: 生命周期管理
- `registration.rs`: 注册逻辑

### 3. Gossip 协议层 (`gossip/`)
- `message.rs`: Gossip 消息定义
- `handler.rs`: 消息处理器
- `heartbeat.rs`: 心跳机制
- `failure.rs`: 失效检测
- `topology.rs`: 拓扑同步

### 4. 拓扑管理层 (`topology/`)
- `state.rs`: 本地拓扑状态
- `update.rs`: 拓扑更新
- `query.rs`: 拓扑查询
- `consistency.rs`: 一致性检查

### 5. 消息传递层 (`messaging/`)
- `send.rs`: 消息发送
- `dispatch.rs`: 消息分发
- `route.rs`: 路由决策
- `forward.rs`: 消息转发

### 6. 运行时 (`runtime/`)
- `runner.rs`: Server 运行器
- `config.rs`: 配置管理
- `event.rs`: 事件系统
- `metrics.rs`: 监控指标

## 测试

```bash
# 运行所有测试
cargo test -p mcp-server-framework

# 运行特定测试
cargo test -p mcp-server-framework --test integration_test
```

## 依赖关系

```
session-manager (Module 1)
        ↓
   router-core (Module 2)
        ↓
 mcp-server-framework (Module 3)
```

## 下一步

Module 3 (MCP Server Framework) 已完成。

接下来可以开始:
- Module 4: Concrete Servers
  - 具体 Server 实现 (Echo, Calculator, Code Review 等)
  - 工具定义与实现
  - Server 间协作示例
  - 端到端测试
