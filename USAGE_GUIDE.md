# SEA Agent 使用指南

本文档提供 SEA (Self-Evolving Agent) 系统的完整使用示例和最佳实践。

## 快速开始

### 1. 构建项目

```bash
cd self-evolving-agent-2
cargo build --release
```

### 2. 启动完整系统

```bash
# 方式 1: 使用 cargo run
cargo run -p sea-agent -- run

# 方式 2: 直接运行二进制文件
./target/release/sea run
```

输出示例：
```
Starting SEA Agent...
SEA Agent is running with session: 123e4567-e89b-12d3-a456-426614174000

Available servers:
  - echo-abc12345 (echo) [running]
  - calculator-def45678 (calculator) [running]
  - time-ghi90123 (time) [running]

Press Ctrl+C to shutdown...
```

---

## 完整示例：构建一个计算服务

### 步骤 1: 创建 Session

```bash
$ sea session create
Session created: 550e8400-e29b-41d4-a716-446655440000
```

### 步骤 2: 注册 Servers

```bash
# 注册计算器 Server
$ sea server register --session 550e8400-e29b-41d4-a716-446655440000 calculator
Server registered: calculator-a1b2c3d4

# 注册 Echo Server (用于测试)
$ sea server register --session 550e8400-e29b-41d4-a716-446655440000 echo
Server registered: echo-e5f6g7h8

# 查看 Servers
$ sea server list --session 550e8400-e29b-41d4-a716-446655440000
Servers:
  calculator-a1b2c3d4 | calculator | session: 550e8400... | [stopped]
  echo-e5f6g7h8 | echo | session: 550e8400... | [stopped]
```

### 步骤 3: 启动 Servers

```bash
$ sea server start calculator-a1b2c3d4
Server started: calculator-a1b2c3d4

$ sea server start echo-e5f6g7h8
Server started: echo-e5f6g7h8
```

### 步骤 4: 发送消息

#### 使用 Echo Server

```bash
$ sea message send --session 550e8400... '{"action": "echo", "text": "Hello, SEA!"}'
Processing: Inorganic
Routed to: echo-e5f6g7h8
Response: Routed to 1 server(s): echo-e5f6g7h8 (processing: Inorganic)
```

#### 使用 Calculator Server

```bash
# 加法
$ sea message send --session 550e8400... '{"action": "add", "a": 10, "b": 20}'
Processing: Inorganic
Routed to: calculator-a1b2c3d4
Response: Routed to 1 server(s): calculator-a1b2c3d4 (processing: Inorganic)

# 乘法
$ sea message send --session 550e8400... '{"action": "multiply", "a": 7, "b": 8}'
Processing: Inorganic
Routed to: calculator-a1b2c3d4
Response: Routed to 1 server(s): calculator-a1b2c3d4 (processing: Inorganic)
```

### 步骤 5: 查看历史

```bash
$ sea message history --session 550e8400...
Message history:
  [User] 2026-04-11 14:51:00 UTC: {"action": "echo", "text": "Hello, SEA!"}
  [Assistant] 2026-04-11 14:51:01 UTC: Routed to 1 server(s): echo-e5f6g7h8 (processing: Inorganic)
  [User] 2026-04-11 14:51:15 UTC: {"action": "add", "a": 10, "b": 20}
  [Assistant] 2026-04-11 14:51:16 UTC: Routed to 1 server(s): calculator-a1b2c3d4 (processing: Inorganic)
```

### 步骤 6: 清理

```bash
# 停止 Servers
$ sea server stop calculator-a1b2c3d4
Server stopped: calculator-a1b2c3d4

$ sea server stop echo-e5f6g7h8
Server stopped: echo-e5f6g7h8

# 删除 Session
$ sea session delete 550e8400-e29b-41d4-a716-446655440000
Session deleted: 550e8400-e29b-41d4-a716-446655440000
```

---

## 配置文件使用

### 生成默认配置

```bash
$ sea config --output config.toml
Default configuration saved to: config.toml
```

### 配置文件示例

```toml
# Session 存储路径
session_store_path = "./sessions"

[router]
# 最大路由跳数
max_hops = 10
# Drain 超时（秒）
drain_timeout = 300
# 分类器类型: "RuleBased" 或 { Mock = { default_organic = true } }
classifier_type = "RuleBased"

[server_defaults]
# Gossip 心跳间隔（秒）
heartbeat_interval_secs = 5
# Gossip 心跳超时（秒）
heartbeat_timeout_secs = 30
# 失效检测阈值
failure_suspect_threshold = 3
```

### 使用自定义配置

```bash
# 方式 1: 使用 --config 参数
$ sea --config /path/to/config.toml run

# 方式 2: 将配置放在默认位置
# Linux/macOS: ~/.config/sea-agent/config.toml
# Windows: %APPDATA%\sea\sea-agent\config.toml
$ sea run
```

---

## 高级用法

### 1. 使用 Mock 分类器（测试模式）

```bash
$ sea run --mock-classifier
```

Mock 分类器会将所有消息标记为"有机处理"或"无机处理"（可配置），用于测试。

### 2. 查看 Session 详情

```bash
$ sea session show 550e8400-e29b-41d4-a716-446655440000
Session: 550e8400-e29b-41d4-a716-446655440000
  State: Active
  Created: 2026-04-11 14:50:00 UTC
  Updated: 2026-04-11 14:51:16 UTC
  Servers: 2
  Messages: 4
  Routing entries: 6

  Registered servers:
    - calculator-a1b2c3d4 [Active] tools: ["add", "subtract", "multiply", "divide"]
    - echo-e5f6g7h8 [Active] tools: ["echo"]

  Routing table:
    capability:add -> calculator-a1b2c3d4
    capability:subtract -> calculator-a1b2c3d4
    capability:multiply -> calculator-a1b2c3d4
    capability:divide -> calculator-a1b2c3d4
    capability:echo -> echo-e5f6g7h8
```

### 3. 查看可用 Server 类型

```bash
$ sea server types
Available server types:
  echo - Echo server
  calculator - Calculator server
  time - Time server
  counter - Counter server
  kvstore - Key-Value store server
```

---

## 可用的 Server 类型

| Server 类型 | 工具 | 用途 |
|------------|------|------|
| **echo** | `echo` | 回显消息，用于测试 |
| **calculator** | `add`, `subtract`, `multiply`, `divide` | 数学运算 |
| **time** | `current_time`, `format_time` | 时间处理 |
| **counter** | `increment`, `decrement`, `get`, `list_counters` | 命名计数器 |
| **kvstore** | `set`, `get`, `delete`, `list_keys` | 键值存储（支持 TTL） |

---

## 消息格式

### 结构化消息（JSON）

```json
{
  "action": "echo",
  "text": "Hello"
}
```

路由器会从以下字段提取能力标识：
- `capability`
- `action`
- `operation`
- `target`

### 非结构化消息（纯文本）

```
Hello, world!
```

纯文本消息会被分类器判定为"有机处理"（需要 LLM），但由于路由器无法提取能力标识，会导致路由失败。

**建议**: 使用 JSON 格式的结构化消息进行路由。

---

## 最佳实践

### 1. Session 隔离

每个 Session 是完全独立的，适合用于：
- 不同的用户或租户
- 不同的项目或环境
- 测试和生产环境分离

### 2. Server 命名

```bash
# 好的命名
$ sea server register --session <id> calculator --id calc-prod-001

# 避免使用无意义的 ID
$ sea server register --session <id> calculator --id abc123  # 不推荐
```

### 3. 资源清理

定期清理不用的 Session：

```bash
# 列出所有 Session
$ sea session list

# 删除不用的 Session
$ sea session delete <old-session-id>
```

### 4. 日志级别

```bash
# 设置日志级别
$ sea --log-level debug run
```

可用级别：`error`, `warn`, `info`, `debug`, `trace`

---

## 故障排除

### 问题 1: 路由失败

**错误**: `RoutingFailed("No capability found in message")`

**原因**: 消息中不包含能力标识。

**解决**: 使用 JSON 格式消息，包含 `action`/`capability`/`operation`/`target` 字段。

### 问题 2: Server 未找到

**错误**: `NotFound("Server 'xxx' not found")`

**原因**: Server ID 不存在或对应的 Session 已被删除。

**解决**:
1. 使用 `sea server list` 查看所有 Server
2. 检查 Server ID 是否正确
3. 注意：Server 注册信息会持久化到 Session 文件，即使进程重启也能通过 `server list` 查看到

### 问题 3: Server 状态说明

**行为**: Server 的 `running` 状态显示为 `[stopped]`，即使之前执行过 `start` 命令。

**原因**: 这是预期行为。SEA Agent 是 CLI 工具，每次命令执行都是独立进程：
- **注册信息**（ID、类型、工具列表）会持久化到 `sessions/*.json`
- **运行状态**（running）是进程内状态，进程退出后丢失
- 下次执行命令时，会自动将持久化状态中的 `Active` 重置为 `Pending`

**建议**: 如果需要长期运行的服务，请使用 `sea run` 命令启动持续运行的 Agent 进程。

### 问题 3: Session 不存在

**错误**: `Session error: NotFound(...)`

**原因**: Session ID 不存在或已被删除。

**解决**: 使用 `sea session list` 查看现有 Session。

---

## 下一步

- 查看各模块的 README 了解详细信息
- 阅读 DESIGN_PHILOSOPHY.md 了解系统设计
- 查看 MODULES.md 了解模块架构
- 运行测试验证功能

---

**版本**: v0.2.0
**更新日期**: 2026-04-11
