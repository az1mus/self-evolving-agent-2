# Self-Evolving Agent 模块拆解

## 系统架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                        Entry Point                          │
│                      (main.rs / app)                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Session Manager                          │
│  Module 1: Session 容器与生命周期管理                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Router Core                              │
│  Module 2: Router 与有机/无机判定                             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   MCP Server Framework                      │
│  Module 3: MCP Server 基础框架与去中心化拓扑                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                Concrete MCP Servers                         │
│  Module 4: 具体业务 Server 实现                               │
└─────────────────────────────────────────────────────────────┘
```

## 模块划分原则

1. **独立可运行**: 每个模块都有独立的 CLI 或测试入口
2. **明确接口**: 模块间通过 trait 和消息传递交互
3. **渐进式开发**: 从基础模块到高级模块，逐步叠加功能
4. **可测试性**: 每个模块提供单元测试和集成测试

---

## Module 1: Session Manager

**职责**: Session 容器与生命周期管理

**核心功能**:
- Session 创建、加载、保存、销毁
- Session 文件 (session.json) 的读写
- Server 注册与生命周期管理
- 消息历史持久化
- Cache 管理

**依赖**: 无（基础模块）

**输出**: `session-manager/` crate

---

## Module 2: Router Core

**职责**: 消息路由与有机/无机判定

**核心功能**:
- 轻量级判定器接口与实现
- 有机/无机处理分类
- 消息预处理流程（规则压缩、一致化、Cache匹配）
- 路由策略（能力匹配、链式、并行）
- 防循环机制

**依赖**: Module 1 (Session Manager)

**输出**: `router-core/` crate

---

## Module 3: MCP Server Framework

**职责**: MCP Server 基础框架与去中心化拓扑

**核心功能**:
- MCP 协议实现（基于 mcp-rust-sdk）
- Server 基础 trait 与生命周期
- Gossip 协议实现
- 去中心化拓扑维护
- 工具注册与发现
- 消息传递与路由

**依赖**: Module 1 (Session Manager)

**输出**: `mcp-server-framework/` crate

---

## Module 4: Concrete Servers

**职责**: 具体业务 Server 实现

**核心功能**:
- 示例 Server 实现（Echo, Calculator, Code Review 等）
- 工具定义与实现
- Server 间协作示例
- 端到端测试场景

**依赖**: Module 1, 2, 3

**输出**: `concrete-servers/` crate

---

## Module 5: Integration & CLI

**职责**: 整合所有模块，提供运行时与 CLI

**核心功能**:
- 主程序入口
- 配置文件管理
- CLI 命令（创建 session、启动 server、发送消息等）
- 集成测试

**依赖**: Module 1, 2, 3, 4

**输出**: `sea-agent/` 主程序

---

## 开发顺序

```
Week 1-2: Module 1 (Session Manager)    ← 基础设施
Week 2-3: Module 3 (MCP Server Framework) ← 核心框架
Week 3-4: Module 2 (Router Core)         ← 路由逻辑
Week 4-5: Module 4 (Concrete Servers)    ← 业务实现
Week 5-6: Module 5 (Integration & CLI)   ← 整合测试
```

---

## 测试策略

每个模块包含：
1. **单元测试**: 测试内部逻辑
2. **集成测试**: 测试模块间交互
3. **示例程序**: 独立运行的 demo

---

## 技术栈

- **语言**: Rust (主要), Python (可选，用于 LLM 判定器)
- **序列化**: serde, serde_json
- **异步运行时**: tokio
- **MCP SDK**: mcp-rust-sdk (或自实现)
- **存储**: JSON 文件 (session.json)
- **日志**: tracing, tracing-subscriber

---

## 下一步

查看各模块的 `todolist.md` 以了解详细实现计划。
