# Module 4: Concrete Servers — Todolist

## 设计路线

```
简单示例 Server → 工具定义 → 业务逻辑 → Server 间协作 → 端到端场景
```

核心思路：先实现简单的示例 Server，再实现复杂的业务 Server，最后演示 Server 间协作。

---

## Phase 1: 基础示例 Servers

- [ ] **1.1 Echo Server**
  - 功能：返回输入内容
  - 工具：`echo(text: string) -> string`
  - 用途：测试消息传递
  - 文件: `src/servers/echo.rs`

- [ ] **1.2 Calculator Server**
  - 功能：基础数学运算
  - 工具：
    - `add(a, b) -> number`
    - `subtract(a, b) -> number`
    - `multiply(a, b) -> number`
    - `divide(a, b) -> number`
  - 用途：测试工具调用
  - 文件: `src/servers/calculator.rs`

- [ ] **1.3 Time Server**
  - 功能：返回当前时间、格式转换
  - 工具：
    - `current_time() -> timestamp`
    - `format_time(timestamp, format) -> string`
  - 用途：测试无状态工具
  - 文件: `src/servers/time.rs`

- [ ] **1.4 单元测试**
  - 每个工具的正确性测试
  - 工具注册到 Server 的流程

---

## Phase 2: 状态管理 Server

- [ ] **2.1 Counter Server**
  - 功能：维护计数器状态
  - 工具：
    - `increment(key) -> number`
    - `decrement(key) -> number`
    - `get(key) -> number`
  - 状态存储：内存（可选持久化到 Session）
  - 用途：测试有状态 Server
  - 文件: `src/servers/counter.rs`

- [ ] **2.2 KeyValue Store Server**
  - 功能：键值存储
  - 工具：
    - `set(key, value, ttl?)`
    - `get(key) -> value`
    - `delete(key)`
    - `list_keys() -> Vec<key>`
  - 状态存储：内存 + Session Cache
  - 用途：测试 Session 集成
  - 文件: `src/servers/kvstore.rs`

- [ ] **2.3 单元测试**
  - 状态更新与读取
  - 多次调用的状态一致性

---

## Phase 3: 文本处理 Server

- [ ] **3.1 Text Analyzer Server**
  - 功能：文本分析
  - 工具：
    - `word_count(text) -> number`
    - `char_count(text) -> number`
    - `extract_keywords(text) -> Vec<string>`
  - 实现方式：本地算法（简单）或调用 LLM（高级）
  - 文件: `src/servers/text_analyzer.rs`

- [ ] **3.2 Text Transformer Server**
  - 功能：文本转换
  - 工具：
    - `to_uppercase(text) -> string`
    - `to_lowercase(text) -> string`
    - `reverse(text) -> string`
    - `base64_encode(text) -> string`
  - 文件: `src/servers/text_transformer.rs`

- [ ] **3.3 单元测试**
  - 各工具的正确性
  - 边界情况（空文本、超长文本）

---

## Phase 4: 外部集成 Server

- [ ] **4.1 HTTP Client Server**
  - 功能：发送 HTTP 请求
  - 工具：
    - `http_get(url) -> response`
    - `http_post(url, body, headers) -> response`
  - 依赖：`reqwest`
  - 用途：测试外部 API 集成
  - 文件: `src/servers/http_client.rs`

- [ ] **4.2 File I/O Server**
  - 功能：文件读写（受限路径）
  - 工具：
    - `read_file(path) -> content`
    - `write_file(path, content)`
    - `list_dir(path) -> Vec<string>`
  - 安全限制：只允许访问指定目录
  - 文件: `src/servers/file_io.rs`

- [ ] **4.3 LLM Gateway Server**
  - 功能：调用 LLM API（如 Claude API）
  - 工具：
    - `complete(prompt, options) -> response`
    - `chat(messages, options) -> response`
  - 依赖：`anthropic-sdk` 或 `reqwest`
  - 配置：API key、模型选择
  - 文件: `src/servers/llm_gateway.rs`

- [ ] **4.4 单元测试**
  - Mock 外部服务
  - 错误处理（超时、API 错误）

---

## Phase 5: 业务逻辑 Server

- [ ] **5.1 Code Review Server**
  - 功能：代码审查
  - 工具：
    - `review_code(code, language) -> ReviewResult`
    - `suggest_improvements(code) -> Vec<Suggestion>`
  - 实现：调用 LLM Gateway Server（有机处理）
  - 返回结构化结果
  - 文件: `src/servers/code_review.rs`

- [ ] **5.2 Task Orchestrator Server**
  - 功能：任务编排
  - 工具：
    - `create_task(description) -> TaskId`
    - `execute_workflow(workflow) -> Result`
    - `get_status(task_id) -> Status`
  - 实现：调用其他 Server（链式路由）
  - 文件: `src/servers/task_orchestrator.rs`

- [ ] **5.3 Data Pipeline Server**
  - 功能：数据处理管道
  - 工具：
    - `extract(source) -> Data`
    - `transform(data, rules) -> Data`
    - `load(data, destination) -> Result`
  - 实现：调用 KV Store、HTTP Client 等
  - 文件: `src/servers/data_pipeline.rs`

- [ ] **5.4 集成测试**
  - Code Review → LLM Gateway 协作
  - Task Orchestrator → 多 Server 协作

---

## Phase 6: Server 间协作场景

- [ ] **6.1 场景 1: 代码审查流程**
  ```
  User → Code Review Server
       → (调用) LLM Gateway Server
       → (调用) Text Analyzer Server (统计)
       → 返回结果
  ```
  - 验证链式路由
  - 文件: `examples/code_review_flow.rs`

- [ ] **6.2 场景 2: 数据处理管道**
  ```
  User → Data Pipeline Server
       → (调用) HTTP Client Server (提取)
       → (调用) Text Transformer Server (转换)
       → (调用) KV Store Server (存储)
       → 返回结果
  ```
  - 验证多 Server 协作
  - 文件: `examples/data_pipeline_flow.rs`

- [ ] **6.3 场景 3: 并行处理**
  ```
  User → Task Orchestrator
       → (并行调用) [Calculator, Text Analyzer, Time]
       → 聚合结果
  ```
  - 验证并行路由
  - 文件: `examples/parallel_processing.rs`

- [ ] **6.4 端到端测试**
  - 运行完整场景
  - 验证结果正确性
  - 文件: `tests/e2e_test.rs`

---

## Phase 7: Server 工厂与注册

- [ ] **7.1 Server 工厂**
  - `ServerFactory::create(type, config) -> Box<dyn MCPServer>`
  - 支持配置文件驱动创建
  - 文件: `src/factory.rs`

- [ ] **7.2 Server 注册表**
  - `ServerRegistry`: 维护所有可用的 Server 类型
  - 动态注册新 Server
  - 文件: `src/registry.rs`

- [ ] **7.3 配置文件示例**
  - `config/servers.yaml`: 定义所有 Server 实例
  - 文件: `config/`

---

## Phase 8: 文档与示例

- [ ] **8.1 Server 开发指南**
  - 如何定义新 Server
  - 工具定义最佳实践
  - 状态管理建议
  - 文件: `docs/server_development.md`

- [ ] **8.2 示例配置**
  - 各 Server 的配置示例
  - 文件: `examples/`

- [ ] **8.3 API 文档**
  - 为所有工具生成文档（rustdoc）
  - 文件: 代码注释

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
tracing = "0.1"

# 可选依赖
reqwest = { version = "0.11", features = ["json"], optional = true }
anthropic = { version = "0.1", optional = true }
regex = { version = "1", optional = true }
base64 = { version = "0.21", optional = true }

[dependencies.mcp-server-framework]
path = "../mcp-server-framework"

[dependencies.session-manager]
path = "../session-manager"

[dependencies.router-core]
path = "../router-core"

[features]
default = ["http", "text-tools"]
http = ["reqwest"]
text-tools = ["regex", "base64"]
llm = ["anthropic", "reqwest"]
```

## 目录结构

```
concrete-servers/
├── Cargo.toml
├── config/
│   └── servers.yaml
├── src/
│   ├── lib.rs
│   ├── factory.rs
│   ├── registry.rs
│   └── servers/
│       ├── mod.rs
│       ├── echo.rs
│       ├── calculator.rs
│       ├── time.rs
│       ├── counter.rs
│       ├── kvstore.rs
│       ├── text_analyzer.rs
│       ├── text_transformer.rs
│       ├── http_client.rs
│       ├── file_io.rs
│       ├── llm_gateway.rs
│       ├── code_review.rs
│       ├── task_orchestrator.rs
│       └── data_pipeline.rs
├── examples/
│   ├── code_review_flow.rs
│   ├── data_pipeline_flow.rs
│   └── parallel_processing.rs
├── tests/
│   ├── e2e_test.rs
│   └── integration_test.rs
└── docs/
    └── server_development.md
```

## 验收标准

1. 所有 Server 能独立启动并通过工具调用测试
2. Code Review 流程能完成代码审查
3. Data Pipeline 能完成 ETL 任务
4. 并行处理场景能正确聚合结果
5. 所有单元测试和集成测试通过
