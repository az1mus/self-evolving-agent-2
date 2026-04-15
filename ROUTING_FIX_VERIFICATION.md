# 路由错误修复验证

## 问题描述

在 REPL 中输入自然语言消息（如 "hi"）时，遇到路由错误：
```
❌ Router error: Routing failed: All routers failed
```

## 根本原因

1. REPL 将 "hi" 包装为 `{"action": "chat", "message": "hi", ...}`
2. `RuleBasedClassifier` 检测到 `action` 字段，分类为**无机处理（Inorganic）**
3. `CompositeRouter` 跳过 LLM Gateway 查找（只在有机处理时查找）
4. 能力路由提取 capability 为 `"capability:chat"`
5. 路由表中没有对应的映射，所有路由器都失败

## 修复方案

### 修改 1：简化 REPL 消息包装逻辑

**文件**: `crates/sea-agent/src/cli/interactive.rs`

**修改前**:
```rust
// 自然语言消息，构造一个通用的对话请求
serde_json::json!({
    "action": "chat",
    "message": message,
    "type": "natural_language"
}).to_string()
```

**修改后**:
```rust
// 自然语言消息，发送纯文本（让分类器判定为有机处理，路由到 LLM Gateway）
message.to_string()
```

**理由**: 纯文本消息会被分类为有机处理，触发 LLM Gateway 路由逻辑。

### 修改 2：改进错误提示

**修改前**:
```rust
println!("💡 Suggestion: Register and start a server that can handle natural language:");
println!("   {} Register an LLM Gateway server", "/server register llm_gateway".cyan());
```

**修改后**:
```rust
println!("💡 To handle natural language messages, you need an LLM Gateway server:");
println!();
println!("   1. Register an LLM Gateway:");
println!("      {}", "/server register llm_gateway".cyan());
println!();
println!("   2. Start the server:");
println!("      {}", "/server start <server-id>".cyan());
```

## 测试步骤

### 1. 启动 SEA Agent REPL
```bash
cargo run --bin sea -- run
```

### 2. 场景 A：有 LLM Gateway 的 Session

创建新 session（自动注册 LLM Gateway）：
```
sea> 👤 Select a session
> Create new session
```

输入自然语言：
```
sea> 👤 16:42:28 hi
✓ Routed to 1 server(s): llm_gateway-xxxxx (processing: organic)
```

### 3. 场景 B：无 LLM Gateway 的 Session

使用现有 session（无 LLM Gateway）：
```
sea> 👤 Select a session
> d52668ca-8843-4a17-ac98-d5568fe5e4fc (active, 2 servers)
```

输入自然语言：
```
sea> 👤 16:42:28 hi
⚠️  No server available to handle this message.

💡 To handle natural language messages, you need an LLM Gateway server:

   1. Register an LLM Gateway:
      /server register llm_gateway

   2. Start the server:
      /server start <server-id>
```

手动注册并启动：
```
sea> /server register llm_gateway
✓ Server registered: llm_gateway-abc123

sea> /server start llm_gateway-abc123
✓ Server llm_gateway-abc123 started

sea> hi
✓ Routed to 1 server(s): llm_gateway-abc123 (processing: organic)
```

## 预期结果

- ✅ 自然语言消息被分类为有机处理
- ✅ 如果有 LLM Gateway，路由成功
- ✅ 如果没有 LLM Gateway，提供清晰的错误提示和修复建议
- ✅ 用户可以手动注册并启动 LLM Gateway

## 技术细节

### 路由流程（修复后）

1. 用户输入: "hi"
2. REPL: 发送纯文本 "hi"
3. 分类器: `RuleBasedClassifier` → **Organic**
4. 路由器: `CompositeRouter`
   - 检测到有机处理
   - 查找 LLM Gateway
   - 如果找到 → 返回 server ID
   - 如果未找到 → 尝试能力路由
   - 能力路由失败 → 返回 "All routers failed"

### 关键代码位置

- REPL 消息处理: `crates/sea-agent/src/cli/interactive.rs:253`
- 分类器逻辑: `crates/router-core/src/classifier.rs:64`
- 路由器逻辑: `crates/router-core/src/router.rs:238`
- LLM Gateway 查找: `crates/router-core/src/router.rs:169`

## 后续改进建议

1. **自动注册 LLM Gateway**: 在创建新 session 时自动注册并启动 LLM Gateway
2. **智能路由提示**: 根据消息内容推荐合适的 server 类型
3. **路由统计**: 记录路由成功率，帮助用户优化配置
