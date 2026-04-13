# Self-Evolving Agent 实施路线图

## 概述

本文档总结了将 DESIGN_PHILOSOPHY.md 中的设计哲学转化为可实施模块的完整计划。

---

## 模块总览

系统被拆分为 **5 个独立模块**，每个模块都可独立开发、测试和运行：

| 模块 | 职责 | 依赖 | 开发周期 |
|------|------|------|----------|
| **Module 1: Session Manager** | Session 容器与生命周期管理 | 无 | Week 1-2 |
| **Module 2: Router Core** | 消息路由与有机/无机判定 | Module 1 | Week 3-4 |
| **Module 3: MCP Server Framework** | MCP Server 基础框架与去中心化拓扑 | Module 1 | Week 2-3 |
| **Module 4: Concrete Servers** | 具体业务 Server 实现 | Module 1, 2, 3 | Week 4-5 |
| **Module 5: Integration & CLI** | 整合所有模块，提供运行时与 CLI | Module 1, 2, 3, 4 | Week 5-6 |

---

## 核心设计原则

### 1. 独立可运行

每个模块都提供：
- **独立的 CLI 入口**：可单独测试功能
- **单元测试与集成测试**：完整的测试覆盖
- **示例配置**：开箱即用的配置文件

### 2. 明确的接口边界

模块间通过 **trait 和消息传递** 交互，避免紧耦合：

```
Session Manager ──(Session API)──> Router Core
                 │
                 └──(Session API)──> MCP Server Framework
                                            │
                                            └──(Server Trait)──> Concrete Servers
                                                                    │
                                                                    └──(组合)──> SEA Agent
```

### 3. 渐进式开发

从基础设施到高级功能，逐步叠加：

```
基础设施 (Session Manager)
     ↓
核心框架 (MCP Server Framework)
     ↓
路由逻辑 (Router Core)
     ↓
业务实现 (Concrete Servers)
     ↓
系统整合 (SEA Agent)
```

### 4. 可测试性

每个模块包含：
- **单元测试**：测试内部逻辑（mock 外部依赖）
- **集成测试**：测试模块间交互
- **端到端测试**：测试完整场景

---

## Workspace 结构

项目已转换为 Rust Workspace，所有模块位于 `crates/` 目录下：

```
self-evolving-agent-2/
├── Cargo.toml                   # Workspace 配置
├── MODULES.md                   # 模块设计文档
├── DESIGN_PHILOSOPHY.md         # 设计哲学
├── crates/
│   ├── session-manager/         # Module 1
│   │   ├── Cargo.toml
│   │   ├── todolist.md
│   │   └── src/
│   ├── router-core/             # Module 2
│   │   ├── Cargo.toml
│   │   ├── todolist.md
│   │   └── src/
│   ├── mcp-server-framework/    # Module 3
│   │   ├── Cargo.toml
│   │   ├── todolist.md
│   │   └── src/
│   ├── concrete-servers/        # Module 4
│   │   ├── Cargo.toml
│   │   ├── todolist.md
│   │   └── src/
│   └── sea-agent/               # Module 5
│       ├── Cargo.toml
│       ├── todolist.md
│       └── src/
└── src/                         # 原有代码（可迁移或废弃）
```

---

## 开发流程建议

### 阶段 1: 基础设施 (Week 1-2)

**目标**: 完成模块 1 (Session Manager)

**关键里程碑**:
- [ ] Session 数据结构定义完成
- [ ] 文件持久化实现
- [ ] Session Manager API 可用
- [ ] Server 生命周期管理完成
- [ ] CLI 可创建/查询/删除 Session

**验收**: `cargo run -p session-manager -- session create` 成功

---

### 阶段 2: 核心框架 (Week 2-3)

**目标**: 完成模块 3 (MCP Server Framework)

**关键里程碑**:
- [ ] MCP 协议实现
- [ ] Server Trait 定义
- [ ] Gossip 协议实现
- [ ] 去中心化拓扑维护
- [ ] 消息传递与路由

**验收**: 多个 Server 能通过 Gossip 同步拓扑

**注意**: 模块 3 与模块 2 可并行开发

---

### 阶段 3: 路由逻辑 (Week 3-4)

**目标**: 完成模块 2 (Router Core)

**关键里程碑**:
- [ ] 消息结构定义
- [ ] 有机/无机判定器
- [ ] 预处理流程
- [ ] 路由策略
- [ ] 防循环机制

**验收**: `cargo run -p router-core -- router classify` 正确判定

---

### 阶段 4: 业务实现 (Week 4-5)

**目标**: 完成模块 4 (Concrete Servers)

**关键里程碑**:
- [ ] 基础示例 Servers (Echo, Calculator)
- [ ] 状态管理 Server (Counter, KV Store)
- [ ] 外部集成 Server (HTTP Client, LLM Gateway)
- [ ] 业务 Server (Code Review, Task Orchestrator)
- [ ] Server 间协作场景

**验收**: Code Review 流程端到端测试通过

---

### 阶段 5: 系统整合 (Week 5-6)

**目标**: 完成模块 5 (Integration & CLI)

**关键里程碑**:
- [ ] Workspace 配置管理
- [ ] 完整 CLI 实现
- [ ] 系统初始化与运行时
- [ ] 优雅关闭与容错
- [ ] 端到端测试

**验收**: `sea-agent run` 能启动完整系统并处理消息

---

## 技术栈统一

所有模块共享统一的依赖版本（在 `Cargo.toml` 的 `[workspace.dependencies]` 中定义）：

- **异步运行时**: Tokio 1.x
- **序列化**: serde + serde_json
- **日志**: tracing + tracing-subscriber
- **CLI**: clap 4.x
- **工具**: uuid, chrono, thiserror, anyhow

---

## 测试策略

### 单元测试

每个模块的单元测试位于 `src/` 下，使用 `#[cfg(test)]` 模块。

示例：
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new();
        assert_eq!(session.state, SessionState::Active);
    }
}
```

### 集成测试

集成测试位于 `tests/` 目录，测试模块间交互。

示例：
```rust
// tests/integration_test.rs
use session_manager::SessionManager;
use router_core::Router;

#[tokio::test]
async fn test_session_with_router() {
    let manager = SessionManager::new();
    let router = Router::new(/* ... */);

    let session = manager.create_session().unwrap();
    // ... 测试交互
}
```

### 端到端测试

位于 `crates/sea-agent/tests/`，测试完整系统流程。

示例：
```rust
// tests/e2e_full_test.rs
#[tokio::test]
async fn test_full_workflow() {
    // 初始化系统
    let system = SEA::new(config).await.unwrap();

    // 创建 Session
    let session_id = system.create_session().await.unwrap();

    // 注册 Servers
    system.register_server(session_id, "echo", Default::default()).await.unwrap();

    // 发送消息
    let result = system.send_message(session_id, message).await.unwrap();

    // 验证结果
    assert!(result.is_success());
}
```

---

## 代码质量保证

### 格式化

```bash
cargo fmt --all
```

### Lint 检查

```bash
cargo clippy --all --all-features -- -D warnings
```

### 文档生成

```bash
cargo doc --all --no-deps --open
```

### 测试运行

```bash
# 运行所有测试
cargo test --all

# 运行特定模块测试
cargo test -p session-manager
```

---

## 从现有代码迁移

项目中已有一些初始实现（`src/` 目录）。建议的迁移策略：

1. **保留有用部分**：
   - `src/logger/` → 可复用到各模块
   - `src/config/` → 可改造为 Workspace 配置

2. **重构部分**：
   - `src/gateway/` → 可融入 Router Core
   - `src/session/` → 可作为 Session Manager 的起点

3. **废弃部分**：
   - 与新架构不兼容的代码可暂时保留为 `legacy/`，待新模块完成后删除

---

## 下一步行动

1. **创建模块目录结构**：
   ```bash
   mkdir -p crates/session-manager/src
   mkdir -p crates/router-core/src
   mkdir -p crates/mcp-server-framework/src
   mkdir -p crates/concrete-servers/src
   mkdir -p crates/sea-agent/src
   ```

2. **初始化各模块的 Cargo.toml**：
   - 从对应的 todolist.md 中复制依赖配置

3. **开始模块 1 开发**：
   - 按照 `session-manager/todolist.md` 的 Phase 1 开始

4. **持续迭代**：
   - 每完成一个 Phase，更新 todolist.md 中的勾选状态
   - 定期运行测试确保质量

---

## 参考文档

- [DESIGN_PHILOSOPHY.md](../DESIGN_PHILOSOPHY.md) — 系统设计哲学
- [MODULES.md](../MODULES.md) — 模块架构说明
- 各模块的 `todolist.md` — 详细实施计划

---

**祝开发顺利！**
