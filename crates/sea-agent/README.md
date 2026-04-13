# SEA Agent

SEA (Self-Evolving Agent) 主程序 - 整合所有模块，提供完整的 CLI 接口。

## 功能

- **Session 管理**: 创建、列出、查看、删除 Session
- **Server 管理**: 注册、启动、停止、列出 Servers
- **消息路由**: 发送消息并查看路由结果
- **完整系统集成**: 一键启动完整系统

## 安装

```bash
cargo install sea-agent
```

## 使用

### 启动完整系统

```bash
sea run
```

这将：
1. 创建默认 Session
2. 注册并启动默认 Servers (Echo, Calculator, Time)
3. 等待 Ctrl+C 优雅关闭

### Session 管理

```bash
# 创建 Session
sea session create

# 列出所有 Session
sea session list

# 查看 Session 详情
sea session show <session-id>

# 删除 Session
sea session delete <session-id>
```

### Server 管理

```bash
# 注册 Server 到 Session
sea server register --session <session-id> echo

# 列出所有 Servers
sea server list

# 列出指定 Session 的 Servers
sea server list --session <session-id>

# 启动 Server
sea server start <server-id>

# 停止 Server
sea server stop <server-id>

# 列出可用的 Server 类型
sea server types
```

### 消息操作

```bash
# 发送消息
sea message send --session <session-id> "Hello, world!"

# 查看消息历史
sea message history --session <session-id>

# 限制历史记录数量
sea message history --session <session-id> --limit 10 --offset 0
```

### 配置管理

```bash
# 生成默认配置文件
sea config --output config.toml
```

## 可用的 Server 类型

- `echo` - 回显服务器
- `calculator` - 计算器服务器
- `time` - 时间服务器
- `counter` - 计数器服务器
- `kvstore` - 键值存储服务器

## 配置

配置文件示例 (`config.toml`):

```toml
# Session 存储路径
session_store_path = "./sessions"

[router]
max_hops = 10
drain_timeout = 300
classifier_type = "RuleBased"

[server_defaults]
heartbeat_interval_secs = 5
heartbeat_timeout_secs = 30
failure_suspect_threshold = 3
```

## 示例工作流

```bash
# 1. 创建 Session
sea session create
# 输出: Session created: 123e4567-e89b-12d3-a456-426614174000

# 2. 注册 Servers
sea server register --session 123e4567-e89b-12d3-a456-426614174000 echo
sea server register --session 123e4567-e89b-12d3-a456-426614174000 calculator

# 3. 启动 Servers
sea server start echo-abc123
sea server start calculator-def456

# 4. 发送消息
sea message send --session 123e4567-e89b-12d3-a456-426614174000 "Hello"

# 5. 查看历史
sea message history --session 123e4567-e89b-12d3-a456-426614174000

# 6. 清理
sea session delete 123e4567-e89b-12d3-a456-426614174000
```

## 开发

运行测试：

```bash
cargo test -p sea-agent
```

构建：

```bash
cargo build -p sea-agent
```

运行：

```bash
cargo run -p sea-agent -- run
```
