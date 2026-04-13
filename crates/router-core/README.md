# Router Core Module

消息路由与有机/无机判定核心模块

## 概述

Router Core 是 Self-Evolving Agent 系统的第二个模块,负责消息路由和有机/无机处理判定。基于 Session Manager 提供的基础设施,Router Core 实现了智能消息分发机制。

## 核心功能

### 1. 消息判定

智能判定消息的处理类型:

- **有机处理** (Organic): 需要调用大型 LLM API 进行推理
- **无机处理** (Inorganic): 确定性规则处理,低延迟、低成本

```rust
use router_core::{Classifier, RuleBasedClassifier, Message, MessageContent};

let classifier = RuleBasedClassifier::new();
let msg = Message::simple(session_id, MessageContent::routing_command("code_review"));
let processing_type = classifier.classify(&msg).await?;
```

### 2. 消息预处理

有机处理前的优化流程:

- **规则压缩**: 移除冗余字段,提取关键信息
- **一致化**: 格式统一,文本标准化
- **缓存匹配**: 查询 Session Cache,避免重复处理

```rust
use router_core::{PreprocessorPipeline, RuleCompressor, Normalizer};

let pipeline = PreprocessorPipeline::default_pipeline();
let processed_msg = pipeline.preprocess(msg, &session).await?;
```

### 3. 智能路由

多种路由策略:

- **能力匹配**: 根据 capability 查找目标 Server
- **链式路由**: 按顺序处理 (pipeline)
- **并行路由**: 同时分发到多个 Server
- **组合路由**: 多策略组合

```rust
use router_core::{Router, CapabilityRouter};

let router = CapabilityRouter::new();
let target_servers = router.route(&msg, &session).await?;
```

### 4. 防循环机制

防止消息在网状拓扑中无限循环:

- 路径记录 (visited_servers)
- 最大跳数限制 (max_hops)
- 循环检测与阻止

## 快速开始

### 构建完整的 Router Core

```rust
use router_core::{
    RouterBuilder, RuleBasedClassifier, CapabilityRouter,
    RuleCompressor, Normalizer, Message, MessageContent,
};

let router_core = RouterBuilder::new()
    .classifier(Box::new(RuleBasedClassifier::new()))
    .preprocessor(Box::new(RuleCompressor::new()))
    .preprocessor(Box::new(Normalizer::new()))
    .router(Box::new(CapabilityRouter::new()))
    .build()?;

// 处理消息
let msg = Message::simple(session_id, MessageContent::routing_command("echo"));
let servers = router_core.process(msg, &session).await?;
```

### 使用 CLI

```bash
# 判定消息类型
cargo run -p router-core -- classify --message '{"action":"execute"}'

# 使用 Mock 判定器
cargo run -p router-core -- classify --message 'plain text' --mock organic

# 路由消息
cargo run -p router-core -- route --message '{"capability":"code_review"}' --session /path/to/session.json
```

### 运行示例

```bash
cargo run -p router-core --example basic_router
```

## 架构设计

```
┌─────────────────────────────────────────────────────────┐
│                       RouterCore                        │
│  ┌───────────────────────────────────────────────────┐  │
│  │  1. Classifier (判定器)                           │  │
│  │     - 判定 Organic/Inorganic                      │  │
│  └───────────────────────────────────────────────────┘  │
│                           ↓                             │
│  ┌───────────────────────────────────────────────────┐  │
│  │  2. Preprocessors (预处理器链)                    │  │
│  │     - RuleCompressor (规则压缩)                   │  │
│  │     - Normalizer (一致化)                         │  │
│  │     - CacheMatcher (缓存匹配)                     │  │
│  └───────────────────────────────────────────────────┘  │
│                           ↓                             │
│  ┌───────────────────────────────────────────────────┐  │
│  │  3. Router (路由器)                               │  │
│  │     - CapabilityRouter (能力匹配)                 │  │
│  │     - ChainedRouter (链式)                        │  │
│  │     - ParallelRouter (并行)                       │  │
│  │     - CompositeRouter (组合)                      │  │
│  └───────────────────────────────────────────────────┘  │
│                           ↓                             │
│  ┌───────────────────────────────────────────────────┐  │
│  │  4. CycleDetector (防循环)                        │  │
│  │     - 检查 visited_servers                        │  │
│  │     - 检查 max_hops                               │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## 核心概念

### Message (消息)

路由的基本单位:

```rust
pub struct Message {
    pub message_id: Uuid,
    pub session_id: SessionId,
    pub content: MessageContent,
    pub routing: RoutingMetadata,
    pub timestamp: DateTime<Utc>,
}
```

### MessageContent (消息内容)

三种类型:

- **Structured**: JSON 结构化内容
- **Unstructured**: 纯文本内容
- **RoutingCommand**: 明确的路由指令

### ProcessingType (处理类型)

- **Organic**: 需要调用 LLM API,高成本、高延迟
- **Inorganic**: 规则处理,低成本、低延迟

## 测试

```bash
# 运行所有测试
cargo test -p router-core

# 运行特定测试
cargo test -p router-core --lib test_capability_router
```

**测试覆盖**: 34 个单元测试,覆盖所有核心功能

## 性能特点

- **异步设计**: 所有核心操作支持异步
- **零拷贝**: 避免不必要的数据克隆
- **缓存优化**: 预处理阶段匹配缓存,减少重复计算
- **智能判定**: 规则判定器低延迟 (<1ms)

## 扩展性

### 自定义判定器

```rust
use router_core::{Classifier, Message, ProcessingType, RouterError};
use async_trait::async_trait;

pub struct MyClassifier;

#[async_trait]
impl Classifier for MyClassifier {
    async fn classify(&self, message: &Message) -> Result<ProcessingType, RouterError> {
        // 自定义判定逻辑
        Ok(ProcessingType::Organic)
    }
}
```

### 自定义预处理器

```rust
use router_core::{Preprocessor, Message, RouterError};
use async_trait::async_trait;

pub struct MyPreprocessor;

#[async_trait]
impl Preprocessor for MyPreprocessor {
    async fn preprocess(&self, message: Message, session: &Session)
        -> Result<Message, RouterError>
    {
        // 自定义预处理逻辑
        Ok(message)
    }
}
```

### 自定义路由器

```rust
use router_core::{Router, Message, RouterError};
use async_trait::async_trait;

pub struct MyRouter;

#[async_trait]
impl Router for MyRouter {
    async fn route(&self, message: &Message, session: &Session)
        -> Result<Vec<ServerId>, RouterError>
    {
        // 自定义路由逻辑
        Ok(vec!["server-a".to_string()])
    }
}
```

## 依赖

- `session-manager`: Session 管理
- `tokio`: 异步运行时
- `serde`/`serde_json`: 序列化
- `async-trait`: 异步 trait 支持
- `tracing`: 日志追踪

## 文档

```bash
# 生成文档
cargo doc -p router-core --open
```

## 下一步

- Module 3: MCP Server Framework
- Module 4: Concrete Servers
- Module 5: Integration & CLI

## 许可证

MIT

## 作者

Az1mus