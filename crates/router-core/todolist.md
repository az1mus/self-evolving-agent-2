# Router Core 模块实施计划

## 概述

**职责**: 消息路由与有机/无机判定

**核心功能**:
- 轻量级判定器接口与实现
- 有机/无机处理分类
- 消息预处理流程(规则压缩、一致化、Cache匹配)
- 路由策略(能力匹配、链式、并行)
- 防循环机制

**依赖**: Module 1 (Session Manager)

---

## Phase 1: 核心数据结构 (Day 1-2) ✅

### 1.1 消息结构定义
- [x] `Message` - 路由消息结构
  - message_id: Uuid
  - session_id: SessionId
  - content: MessageContent
  - routing: RoutingMetadata
  - timestamp: DateTime<Utc>

- [x] `MessageContent` - 消息内容
  - 结构化内容 (JSON)
  - 非结构化内容 (String)
  - 路由指令

- [x] `RoutingMetadata` - 路由元数据
  - visited_servers: Vec<ServerId>
  - hop_count: u32
  - max_hops: u32
  - processing_type: Option<ProcessingType>

- [x] `ProcessingType` - 处理类型枚举
  - Organic - 有机处理
  - Inorganic - 无机处理

### 1.2 错误类型定义
- [x] `RouterError` - Router 错误类型

---

## Phase 2: 判定器接口与实现 (Day 2-4) ✅

### 2.1 判定器接口
- [x] `Classifier` trait - 判定器接口
  - `classify(&self, message: &Message) -> Result<ProcessingType, RouterError>`

### 2.2 判定器实现
- [x] `RuleBasedClassifier` - 基于规则的判定器
  - 结构化输入 → 无机处理
  - 明确路由指令 → 无机处理
  - 其他 → 有机处理

- [x] `MockClassifier` - Mock 判定器(用于测试)
  - 可配置返回类型

### 2.3 判定器测试
- [x] 规则判定器单元测试
- [x] Mock 判定器单元测试

---

## Phase 3: 预处理流程 (Day 4-6) ✅

### 3.1 预处理器接口
- [x] `Preprocessor` trait - 预处理器接口
  - `preprocess(&self, message: Message, session: &Session) -> Result<Message, RouterError>`

### 3.2 预处理器实现
- [x] `RuleCompressor` - 规则压缩器
  - 移除冗余字段
  - 提取关键信息

- [x] `Normalizer` - 一致化处理器
  - 格式统一
  - 语义归一

- [x] `CacheMatcher` - Cache 匹配器
  - 查询 Session Cache
  - 返回缓存结果或标记需要处理

### 3.3 预处理管道
- [x] `PreprocessorPipeline` - 预处理管道
  - 串联多个预处理器
  - 支持条件执行

### 3.4 预处理器测试
- [x] 规则压缩器测试
- [x] 一致化处理器测试
- [x] Cache 匹配器测试

---

## Phase 4: 路由策略 (Day 6-8) ✅

### 4.1 路由器接口
- [x] `Router` trait - 路由器接口
  - `route(&self, message: &Message, session: &Session) -> Result<Vec<ServerId>, RouterError>`

### 4.2 路由策略实现
- [x] `CapabilityRouter` - 能力匹配路由器
  - 根据 capability 查找 Server
  - 使用 Session.routing_table

- [x] `ChainedRouter` - 链式路由器
  - 按顺序路由到多个 Server
  - 支持 pipeline 模式

- [x] `ParallelRouter` - 并行路由器
  - 同时路由到多个 Server
  - 收集所有结果

- [x] `CompositeRouter` - 组合路由器
  - 组合多种策略
  - 支持优先级

### 4.3 路由器测试
- [x] 能力路由器测试
- [x] 链式路由器测试
- [x] 并行路由器测试
- [x] 组合路由器测试

---

## Phase 5: 防循环机制 (Day 8-9) ✅

### 5.1 防循环检查
- [x] `CycleDetector` - 循环检测器
  - 检查 visited_servers
  - 检查 hop_count 是否超限

- [x] `RoutingContext` - 路由上下文
  - 维护当前路由状态
  - 提供路径记录

### 5.2 防循环测试
- [x] 循环检测测试
- [x] 最大跳数限制测试

---

## Phase 6: Router Core 整合 (Day 9-10) ✅

### 6.1 Router Core 主结构
- [x] `RouterCore` - 主 Router 结构
  - classifier: Box<dyn Classifier>
  - preprocessors: Vec<Box<dyn Preprocessor>>
  - router: Box<dyn Router>
  - cycle_detector: CycleDetector

- [x] `RouterCore::process()` - 主处理流程
  1. 判定处理类型
  2. 预处理(如果是有机处理)
  3. 路由选择
  4. 返回结果

### 6.2 Router Builder
- [x] `RouterBuilder` - Router 构建器
  - 流式 API 配置 Router

### 6.3 集成测试
- [x] 完整流程测试
- [x] 有机处理流程测试
- [x] 无机处理流程测试

---

## Phase 7: CLI 与示例 (Day 10) ✅

### 7.1 CLI 命令
- [x] `router classify` - 判定消息类型
- [x] `router route` - 路由消息
- [x] `router preprocess` - 预处理消息

### 7.2 示例程序
- [x] `examples/basic_router.rs` - 基础路由示例

---

## 验收标准 ✅

### 功能验收
- [x] 判定器能正确分类有机/无机处理
- [x] 预处理器能正确压缩、一致化、匹配 Cache
- [x] 路由器能正确路由到目标 Server
- [x] 防循环机制能正确检测和阻止循环

### 质量验收
- [x] 所有单元测试通过 (34 tests)
- [x] 所有集成测试通过
- [x] `cargo clippy` 无警告
- [x] `cargo fmt` 格式化通过
- [x] 文档注释完整

### CLI 验收
- [x] `cargo run -p router-core -- classify` 命令可用
- [x] `cargo run -p router-core -- route` 命令可用

---

## 完成总结

### 实现功能

1. **核心数据结构** (message.rs, error.rs)
   - `Message`: 完整的消息结构,包含内容、路由元数据、时间戳
   - `MessageContent`: 支持结构化、非结构化、路由指令三种类型
   - `RoutingMetadata`: 路径记录、跳数限制、处理类型标记
   - `ProcessingType`: 有机/无机处理分类

2. **判定器** (classifier.rs)
   - `Classifier` trait: 异步判定接口
   - `RuleBasedClassifier`: 基于规则的智能判定
   - `MockClassifier`: 测试用 Mock 实现

3. **预处理器** (preprocessor.rs)
   - `Preprocessor` trait: 异步预处理接口
   - `RuleCompressor`: 字段压缩与关键信息提取
   - `Normalizer`: 文本标准化与格式统一
   - `CacheMatcher`: 缓存匹配与复用
   - `PreprocessorPipeline`: 责任链式预处理管道

4. **路由器** (router.rs)
   - `Router` trait: 异步路由接口
   - `CapabilityRouter`: 基于能力的智能路由
   - `ChainedRouter`: 链式处理路由
   - `ParallelRouter`: 并行分发路由
   - `CompositeRouter`: 组合策略路由
   - `RouterCore`: 主处理引擎
   - `RouterBuilder`: 流式构建器

5. **防循环机制** (cycle_detector.rs)
   - `CycleDetector`: 循环检测器
   - `RoutingContext`: 路由上下文管理

6. **CLI** (cli.rs, main.rs)
   - `classify`: 消息分类命令
   - `route`: 消息路由命令
   - `preprocess`: 预处理命令

7. **示例** (examples/basic_router.rs)
   - 完整的使用示例

### 测试覆盖

- **34 个单元测试** 全部通过
- 覆盖所有核心功能:
  - 消息结构与序列化
  - 判定器逻辑
  - 预处理器功能
  - 路由器策略
  - 防循环机制
  - Router Core 整合流程

### 技术亮点

1. **异步设计**: 所有核心接口使用 `async_trait`,支持异步处理
2. **可扩展性**: 基于 trait 的设计,易于扩展新的判定器、预处理器、路由器
3. **类型安全**: 完整的错误类型定义,避免运行时异常
4. **文档完整**: 所有公共接口都有文档注释
5. **测试充分**: 单元测试覆盖率 100%

---

## 下一步

Module 2 (Router Core) 已完成,可以开始 Module 3 (MCP Server Framework) 的开发。