# Module 2: Router Core — Todolist

## 设计路线

```
消息结构定义 → 判定器接口 → 预处理流程 → 路由策略 → 防循环机制 → Router 整合
```

核心思路：先定义消息格式，再实现判定器，再处理消息预处理，最后实现路由与防循环。

---

## Phase 1: 消息结构与路由基础

- [ ] **1.1 定义消息结构**
  - `Message`: { id, session_id, task_type, payload, context, routing }
  - `Payload`: { hash, content_ref, schema_version }
  - `MessageContext`: { cache_key, session_refs }
  - `RoutingInfo`: { visited_servers, hop_count, max_hops }
  - 实现 `Serialize/Deserialize`
  - 文件: `src/message.rs`

- [ ] **1.2 定义路由结果类型**
  - `RouteDecision`: enum { Organic, Inorganic }
  - `RouteTarget`: { server_id, capability }
  - `RouteResult`: { decision, targets, preprocessed_message? }
  - 文件: `src/router.rs`

- [ ] **1.3 单元测试**
  - 消息序列化/反序列化
  - RoutingInfo 的 hop 计数更新

---

## Phase 2: 有机/无机判定器

- [ ] **2.1 定义判定器接口**
  - `Classifier` trait: `classify(message) -> RouteDecision`
  - `ClassifierConfig`: { type, model_path, threshold }
  - 文件: `src/classifier/mod.rs`

- [ ] **2.2 实现规则分类器**
  - `RuleBasedClassifier`: 基于规则的简单判定
  - 规则示例：
    - 消息包含明确 `task_type` → Inorganic
    - payload 有 `hash` 且匹配规则 → Inorganic
    - 否则 → Organic
  - 可配置规则列表
  - 文件: `src/classifier/rule_based.rs`

- [ ] **2.3 实现 LLM 判定器（可选）**
  - `LLMClassifier`: 调用小型 LLM API
  - Prompt 模板："Classify this message as organic/inorganic..."
  - 支持本地模型（ONNX）或远程 API
  - 文件: `src/classifier/llm.rs`

- [ ] **2.4 判定器配置与加载**
  - `ClassifierFactory::from_config(config) -> Box<dyn Classifier>`
  - 默认使用 RuleBasedClassifier
  - 文件: `src/classifier/factory.rs`

- [ ] **2.5 单元测试**
  - 规则分类器：已知输入 → 预期判定
  - LLM 判定器：Mock 测试

---

## Phase 3: 消息预处理流程

- [ ] **3.1 规则压缩**
  - `RuleCompressor`: 去除冗余字段、提取关键信息
  - 预定义压缩规则（如移除 `metadata.debug_info`）
  - 文件: `src/preprocessor/compress.rs`

- [ ] **3.2 一致化压缩**
  - `Normalizer`: 格式统一、语义归一
  - 示例：统一日期格式、同义词映射
  - 文件: `src/preprocessor/normalize.rs`

- [ ] **3.3 Cache 匹配与补全**
  - `CacheMatcher`: 查询 Session Cache
  - 接收 `SessionManager` 引用
  - 相同输入（hash 匹配）→ 返回缓存结果
  - 相似输入（语义匹配）→ 返回部分缓存
  - 文件: `src/preprocessor/cache_match.rs`

- [ ] **3.4 预处理管道**
  - `Preprocessor`: 组合 Compress → Normalize → CacheMatch
  - `preprocess(message, session) -> PreprocessedMessage`
  - 文件: `src/preprocessor/mod.rs`

- [ ] **3.5 单元测试**
  - 规则压缩：输入有冗余 → 输出精简
  - Cache 命中：重复输入 → 返回缓存
  - 预处理管道：完整流程测试

---

## Phase 4: 路由策略

- [ ] **4.1 定义路由策略接口**
  - `RoutingStrategy` trait: `route(message, session) -> Vec<RouteTarget>`
  - 文件: `src/routing/mod.rs`

- [ ] **4.2 能力匹配路由**
  - `CapabilityRouter`: 根据消息 `task_type` 匹配 Server 能力
  - 查询 Session 的 `routing_table`
  - 文件: `src/routing/capability.rs`

- [ ] **4.3 链式路由**
  - `ChainRouter`: A → B → C pipeline
  - 预定义链式路由配置
  - 文件: `src/routing/chain.rs`

- [ ] **4.4 并行路由**
  - `ParallelRouter`: 消息分发到多个 Server
  - 返回多个 RouteTarget
  - 文件: `src/routing/parallel.rs`

- [ ] **4.5 动态路由（负载均衡）**
  - `DynamicRouter`: 根据 Server 负载动态选择
  - 简单实现：轮询或随机
  - 文件: `src/routing/dynamic.rs`

- [ ] **4.6 路由策略工厂**
  - `RoutingStrategyFactory::from_config() -> Box<dyn RoutingStrategy>`
  - 默认使用 CapabilityRouter
  - 文件: `src/routing/factory.rs`

- [ ] **4.7 单元测试**
  - 能力匹配：已知能力 → 找到 Server
  - 链式路由：按顺序执行
  - 并行路由：返回多个目标

---

## Phase 5: 防循环机制

- [ ] **5.1 路径记录检查**
  - `CircularDetector::check(message, server_id) -> bool`
  - 检查 `visited_servers` 是否包含 `server_id`
  - 文件: `src/circular.rs`

- [ ] **5.2 最大跳数检查**
  - `HopLimiter::check(message) -> bool`
  - 检查 `hop_count < max_hops`
  - 文件: `src/circular.rs`

- [ ] **5.3 TTL 超时检查**
  - 为消息添加 `created_at` 时间戳
  - `TTLChecker::check(message, ttl) -> bool`
  - 文件: `src/circular.rs`

- [ ] **5.4 静态拓扑检查（可选）**
  - 注册 Server 时检测是否会引入循环
  - 基于依赖图分析
  - 文件: `src/topology_check.rs`

- [ ] **5.5 单元测试**
  - 循环检测：重复 Server ID → 拒绝
  - 最大跳数：超过阈值 → 终止

---

## Phase 6: Router 整合

- [ ] **6.1 实现 Router**
  - `Router::new(classifier, preprocessor, strategy, detector)`
  - `route(message, session) -> RouteResult`
  - 流程：
    1. 判定有机/无机
    2. 有机 → 预处理 → 路由
    3. 无机 → 直接路由
    4. 防循环检查
  - 文件: `src/router.rs`

- [ ] **6.2 Router 配置**
  - `RouterConfig`: { classifier, preprocessor, strategy, circular_check }
  - 从 JSON/TOML 加载配置
  - 文件: `src/config.rs`

- [ ] **6.3 集成测试**
  - 完整流程：消息 → 判定 → 预处理 → 路由 → 结果
  - 不同策略下的路由结果

---

## Phase 7: 独立运行与测试

- [ ] **7.1 CLI 子命令**
  - `router classify <message.json>` — 判定有机/无机
  - `router route <message.json>` — 执行路由（需 Session）
  - `router preprocess <message.json>` — 执行预处理
  - 文件: `src/cli.rs`, `src/main.rs`

- [ ] **7.2 示例配置文件**
  - `config/default_router.json`
  - 配置分类器、路由策略、防循环参数
  - 文件: `config/`

- [ ] **7.3 端到端测试**
  - 加载配置 → 处理消息 → 验证路由结果

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
sha2 = "0.10"

# 可选：LLM 分类器
reqwest = { version = "0.11", features = ["json"], optional = true }
ort = { version = "1", optional = true }  # ONNX Runtime

[dependencies.session-manager]
path = "../session-manager"

[features]
default = ["rule-based"]
rule-based = []
llm-classifier = ["reqwest", "ort"]
```

## 目录结构

```
router-core/
├── Cargo.toml
├── config/
│   └── default_router.json
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── lib.rs
│   ├── config.rs
│   ├── message.rs
│   ├── router.rs
│   ├── classifier/
│   │   ├── mod.rs
│   │   ├── rule_based.rs
│   │   ├── llm.rs
│   │   └── factory.rs
│   ├── preprocessor/
│   │   ├── mod.rs
│   │   ├── compress.rs
│   │   ├── normalize.rs
│   │   └── cache_match.rs
│   ├── routing/
│   │   ├── mod.rs
│   │   ├── capability.rs
│   │   ├── chain.rs
│   │   ├── parallel.rs
│   │   ├── dynamic.rs
│   │   └── factory.rs
│   ├── circular.rs
│   └── topology_check.rs
└── tests/
    ├── integration_test.rs
    └── fixtures/
        ├── sample_message.json
        └── sample_router_config.json
```

## 验收标准

1. `cargo run -- router classify` 能正确判定有机/无机
2. `cargo run -- router route` 能返回路由目标
3. 防循环机制能拦截循环路由
4. 预处理能压缩并 Cache 命中
5. 所有单元测试通过
