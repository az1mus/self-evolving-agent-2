# Self-Evolving Agent 设计哲学

> 版本: v5
> 更新日期: 2026-04-08

## 概述

Self-Evolving Agent 是一个基于 MCP (Model Context Protocol) 的分层去中心化智能代理系统。系统采用 **Session-Router-Servers** 三层架构，Session 作为唯一持久化容器，MCP Server 在 Session 内部以去中心化网状结构运行，支持运行时动态演进。

```
┌─────────────────────────────────────────────────────────────┐
│                        Session                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                     Router                             │  │
│  │  ┌─────────────────────────────────────────────────┐   │  │
│  │  │              MCP Servers (网状)                  │   │  │
│  │  │                                                  │   │  │
│  │  │    [Server A] ←→ [Server B] ←→ [Server C]      │   │  │
│  │  │         ↑              ↑              ↑         │   │  │
│  │  │         └──────────────┴──────────────┘         │   │  │
│  │  │                                                  │   │  │
│  │  └─────────────────────────────────────────────────┘   │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  session.json (唯一持久化文件)                               │
└─────────────────────────────────────────────────────────────┘
```

---

## 核心设计哲学

### 1. 分层架构

**系统采用三层架构，各层职责明确：**

| 层级 | 组件 | 职责 |
|------|------|------|
| **容器层** | Session | 生命周期管理、状态持久化、资源隔离 |
| **路由层** | Router | 消息分发、有机/无机判定、拓扑协调 |
| **处理层** | MCP Servers | 具体业务处理、工具提供 |

#### 层间交互

```
外部输入 ──> Session 接收
              │
              ▼
         Router 判定 ──┬──> 无机处理 ──> 直接路由
                       │
                       └──> 有机处理 ──> 预处理 ──> 路由
                                              │
                                              ▼
                                        MCP Servers
                                              │
                                              ▼
                                        Session 持久化
```

---

### 2. Session 作为容器

**Session 是系统的唯一持久化单位，所有状态通过 Session 文件管理。**

#### Session 职责

- **生命周期容器**: 管理 MCP Servers 的创建、运行、销毁
- **唯一持久化**: 所有状态存储在单一 `session.json` 文件中
- **资源隔离**: 不同 Session 之间完全独立
- **上下文管理**: 维护消息历史、运行状态、缓存数据

#### Session 文件结构

```json
{
  "session_id": "uuid-v4",
  "created_at": "2026-04-08T10:00:00Z",
  "state": "active",
  "servers": {
    "server-a": {
      "status": "active",
      "tools": ["tool1", "tool2"],
      "metadata": {}
    }
  },
  "routing_table": {
    "capability:code_review": "server-a"
  },
  "message_history": [
    { "role": "user", "content": "...", "timestamp": "..." }
  ],
  "cache": {
    "input_cache": {},
    "inference_cache": {}
  },
  "config": {
    "max_hops": 10,
    "drain_timeout": 300
  }
}
```

#### Session 内 Server 生命周期

```
Pending ──> Active ──> Draining ──> Removed
  │          │           │
  │          │           └─ 等待处理完成或超时强制移除
  │          │
  │          └─ 正常运行
  │
  └─ 已注册但未启动
```

---

### 3. MCP Server 作为基础模块化单位

**MCP Server 是处理信息的基础模块化单位，遵循 MCP 协议在 Servers 之间传递信息。**

- 每个 Server 是独立的功能单元
- Server 之间通过 MCP 协议通信
- Server 可以提供工具 (Tools) 供其他 Server 调用
- **所有 Server 运行在同一 Session 容器内**

---

### 4. Router 与路由判定

**Router 负责消息分发与有机/无机处理判定。**

#### 有机/无机判定机制

Router 使用**轻量级判定器**决定处理方式：

| 判定器类型 | 规格 | 特点 |
|------------|------|------|
| **轻量级分类器** | <100M 参数 | 专门训练的分类模型，低延迟 |
| **轻量级 LLM** | <4B 参数或免费 API | 小型语言模型，通过 prompt 判定 |

#### 判定流程

```
输入消息
    │
    ▼
┌─────────────────────────────────────┐
│        轻量级判定器                   │
│  (分类器 或 轻量级 LLM)               │
└─────────────────────────────────────┘
    │
    ├─── 判定为无机处理 ──→ 规则引擎处理
    │                         │
    │                         ▼
    │                    直接路由到目标 Server
    │
    └─── 判定为有机处理 ──→ 预处理流程
                              │
                              ▼
                         大型 LLM API
```

#### 判定器配置

```json
{
  "router": {
    "classifier": {
      "type": "local_model | remote_api",
      "model_path": "/path/to/classifier.onnx",
      "threshold": 0.7
    },
    "fallback": "organic"
  }
}
```

#### 判定标准（示例）

| 特征 | 倾向无机处理 | 倾向有机处理 |
|------|--------------|--------------|
| 输入结构 | 结构化、字段完整 | 非结构化、模糊 |
| 任务类型 | 路由、分发、格式转换 | 推理、理解、生成 |
| 规则匹配度 | 高（有明确规则覆盖） | 低（无匹配规则） |
| 历史相似度 | 有精确匹配 Cache | 无匹配或语义相似 |

---

### 5. 有机处理预处理流程

**Server 进行有机处理前执行预处理流程：**

```
输入信息
    │
    ▼
┌─────────────────┐
│ 1. 规则压缩      │  ← 按预定义规则裁剪
│   - 移除冗余    │    - 去除重复、空白、无效信息
│   - 提取关键    │    - 只保留决策所需字段
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ 2. 一致化压缩    │  ← 标准化格式
│   - 格式统一    │    - 统一编码、结构、命名
│   - 语义归一    │    - 同义表达映射到标准形式
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ 3. 按需匹配持久化│  ← 查询 Session Cache
│   - Cache 查询  │    - 检查是否有相似/相同输入
│   - 信息补全    │    - 从 Session 补全必要上下文
│   - 去重        │    - 已处理过的直接返回缓存结果
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  结构化输入      │  ← 最小化、标准化、结构化
└─────────────────┘
    │
    ▼
有机处理 (LLM API)
```

**目标**: 输入最小化、Cache 命中最大化

---

### 6. 结构化输入

**有机处理的理想输入对象是结构化的：**

- 使用 JSON/YAML 等结构化格式
- 包含唯一标识用于 Cache 匹配
- 引用 Session 持久化而非内嵌完整上下文

#### 结构化输入示例

```json
{
  "message_id": "uuid-v4",
  "session_id": "session-uuid",
  "task_type": "code_review",
  "payload": {
    "hash": "sha256-of-content",
    "content_ref": "session://messages/123",
    "schema_version": "1.0"
  },
  "context": {
    "cache_key": "optional-cache-key",
    "session_refs": [
      "session://context/project",
      "session://cache/key"
    ]
  },
  "routing": {
    "visited_servers": ["server-id-1", "server-id-2"],
    "hop_count": 2,
    "max_hops": 10
  }
}
```

#### 结构化 vs 非结构化对比

| 方面 | 非结构化（不理想） | 结构化（理想） |
|------|-------------------|----------------|
| Token 消耗 | 高 | 最小化 |
| Cache 命中 | 无法命中 | hash 匹配 |
| 处理效率 | 低 | 高 |
| 结果确定性 | 强不确定性 | 更确定 |

---

### 7. 有机与无机处理

**信息处理分为有机处理与无机处理：**

| 类型 | 定义 | 特点 | 应用场景 |
|------|------|------|----------|
| **有机处理** | 调用大型 LLM API | 不确定性、高延迟、高成本 | 复杂推理、非结构化理解 |
| **无机处理** | 确定性规则处理 | 确定性、低延迟、低成本 | 信息分发、路由决策、拓扑管理 |

**大部分路由与分发决策由无机处理完成。**

---

### 8. 路由策略

**消息在 MCP 网中的流转遵循路由策略，支持多种模式：**

| 路由模式 | 描述 | 适用场景 |
|----------|------|----------|
| **能力匹配** | 根据消息内容选择具备对应能力的 Server | 任务分发 |
| **链式传递** | A → B → C → ... (pipeline) | 流式处理 |
| **并行分发** | 消息同时分发到多个 Server | 并行处理 |
| **动态路由** | 根据运行状态实时调整路径 | 负载均衡 |

---

### 9. 防循环机制

**消息携带路径记录和最大跳数限制，防止网状路由中的循环依赖和无限传递。**

#### 防循环策略

| 策略 | 实现方式 |
|------|----------|
| **路径记录** | 消息携带 `visited_servers` 列表，Server 检查自己是否已在路径中 |
| **最大跳数** | 消息携带 `hop_count`，超过 `max_hops` 阈值终止（可配置） |
| **超时机制** | 消息有 TTL，超时自动终止路由 |
| **静态拓扑检查** | 注册新 Server 时检测是否引入循环 |
| **Drain 超时** | Draining 状态超时后强制移除，防止永久阻塞 |

---

### 10. Session 内去中心化架构

**在 Session 容器内，MCP Servers 采用去中心化架构，各 Server 通过 Gossip 协议维护拓扑一致性，本地决策路由并渐进式构建路由表。**

#### 每个 Server 内部维护

```
LocalState:
  - known_peers: Set<ServerId>     ← 知道的邻居（同 Session 内）
  - local_tools: List<ToolMeta>    ← 本地工具
  - routing_table: Map<Capability, ServerId>
  - session_ref: SessionId         ← 所属 Session
  - version: u64                   ← 版本号
```

#### Gossip 消息类型

| 消息类型 | 用途 |
|----------|------|
| `Heartbeat` | "我还在" - 维护存活状态 |
| `Join` | "新成员加入" - 新 Server 加入 Session |
| `Leave` | "成员离开" - 正常退出通知 |
| `ToolAnnounce` | "我有新工具" - 工具变更通知 |
| `TopologySync` | "拓扑同步请求" - 拉取拓扑信息 |

#### Server 收到消息时

```
Server 收到消息时:

1. 检查本地 routing_table
   - 有匹配的能力？ → 直接转发

2. 本地无匹配？
   - 广播 QueryCapability 给 known_peers
   - 等待响应（超时机制）
   - 收集响应，更新 routing_table

3. 仍无响应？
   - 本地尝试处理（如果本地有能力）
   - 或返回 "No capable server found"

4. 路由成功后
   - 缓存结果到 routing_table
   - 下次同类请求直接转发
```

#### Server 生命周期管理

**新 Server 加入 Session**:
1. 向 Session 注册，获取已知 peers
2. 发送 `Join` 消息给 peers
3. Peers 更新 `known_peers`，回复 `Welcome`
4. 新节点收集 `Welcome`，构建初始拓扑视图
5. 广播 `ToolAnnounce`，声明自己提供的工具
6. Session 更新持久化文件

**Server 正常离开**:
1. 发送 `Leave` 消息给 `known_peers`
2. 各节点更新 `known_peers`
3. 清理 `routing_table` 中相关条目
4. Session 更新持久化文件

**Server 异常离开（崩溃）**:
1. `Heartbeat` 超时检测
2. 检测到超时后标记为 "疑似失效"
3. 广播 `Suspect` 消息
4. 达到确认阈值后，标记为 "确认失效"
5. 清理相关路由条目
6. Session 更新持久化文件

#### 失效判定配置

```json
{
  "failure_detection": {
    "heartbeat_interval": 5,
    "heartbeat_timeout": 30,
    "suspect_threshold": 3,
    "confirm_threshold": 0.51
  }
}
```

---

## Cache 策略

| Cache 类型 | 用途 | 匹配方式 |
|------------|------|----------|
| **输入 Cache** | 相似输入直接返回 | hash 匹配 |
| **推理 Cache** | 部分推理结果复用 | 语义相似度 |
| **上下文 Cache** | 预加载常用上下文 | 引用路径 |

**所有 Cache 存储在 Session 文件中，随 Session 生命周期管理。**

---

## 设计权衡

| 方面 | 选择 | 优势 | 挑战 |
|------|------|------|------|
| **架构** | 分层（Session-Router-Servers） | 职责清晰、便于管理 | 层间协调开销 |
| **持久化** | Session 单文件 | 简单、可视化、易迁移 | 单文件并发写入 |
| **Agent 状态** | Stateless（Server 层） | 易迁移、易复制、无状态同步 | 通过 Session 传递上下文 |
| **拓扑管理** | Session 内去中心化 | 无单点故障、天然分布式 | Session 内最终一致性 |
| **路由判定** | 轻量级判定器 | 低成本、低延迟、确定性 | 判定准确性依赖模型 |
| **信息处理** | 有机/无机分离 | 成本可控、效率优化 | 需明确边界 |

---

## 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v1 | - | 初始设计：Gateway 为消息路由中介 |
| v2 | - | 增加路由策略、防循环机制、协调者（可选） |
| v3 | - | Agent 改 Stateless，拓扑改为去中心化 Gossip |
| v4 | 2026-04-08 | 增加有机/无机处理分类，预处理流程，结构化输入规范 |
| v5 | 2026-04-08 | 重构为分层架构（Session-Router-Servers），Session 作为唯一持久化容器，增加轻量级判定器解决有机/无机边界问题，补充失效判定配置、Drain 超时机制 |

---

## 参考文献

- [MCP (Model Context Protocol)](https://modelcontextprotocol.io/)
- [Gossip Protocol](https://en.wikipedia.org/wiki/Gossip_protocol)
- [Semantic Caching for LLMs](https://www.anthropic.com/research)