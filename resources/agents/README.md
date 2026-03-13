# Agent 定义模板

本目录包含 Agent Definition 的模板文件，用于创建可复用的 Agent 配置。

## 文件说明

| 文件 | 说明 |
|------|------|
| `template.json` | 完整模板，包含所有可用配置选项和详细注释 |
| `assistant.json` | 简洁示例：通用助手 |
| `rust-expert.json` | 完整示例：Rust 专家 |
| `agent-creator.json` | 特殊示例：Agent 创建助手 |

## 使用方法

1. 复制模板文件到 Gateway 的 agent 定义目录
2. 修改配置（`id`、`name`、`structured_prompt` 等）
3. 重启应用或重新加载

## 结构说明

### AgentDefinition

```json
{
  "id": "唯一标识符",
  "name": "Agent 名称",
  "description": "描述（可选）",
  "structured_prompt": { ... },
  "prompt_template": { ... },  // 可选
  "llm_config": { ... },       // 可选
  "created_at": "ISO 8601 时间",
  "updated_at": "ISO 8601 时间"
}
```

### StructuredPrompt（核心）

动态 key-value 结构，格式为 `{"prompt1": "xx", "prompt2": "xx"}`：

```json
{
  "structured_prompt": {
    "role": "你是一个助手",
    "style": "回答要简洁",
    "instruction": "请使用中文"
  }
}
```

**重要**：JSON 中键值对的顺序即为 prompt 组装的顺序。

### 变量替换

prompt 内容支持 `{{变量名}}` 格式的变量替换：

```json
{
  "structured_prompt": {
    "role": "你是一个{{role_name}}",
    "greeting": "你好，{{user_name}}！"
  }
}
```

运行时通过 `PromptContext` 提供变量值：
- `{{变量名}}`: 访问程序暴露的变量，在 Session 中维护
- `{{函数名(参数)}}`: 调用 Agent 可访问的函数（待实现）

### PromptTemplate（可选）

自定义 prompt 的组装格式：

```json
{
  "prompt_template": {
    "separator": "\n\n",
    "include_titles": true,
    "title_format": "## {title}",
    "disabled": ["examples"]
  }
}
```

| 字段 | 说明 |
|------|------|
| `separator` | 组件之间的分隔符（默认 `\n\n`） |
| `include_titles` | 是否添加标题（默认 true） |
| `title_format` | 标题格式（默认 `## {title}`） |
| `disabled` | 禁用的组件列表 |

### LLM 配置（可选）

`llm_config` 字段是可选的：
- **留空**：实例化时使用程序默认配置（推荐）
- **指定**：使用自定义配置

```json
// 使用默认配置（推荐）
{
  "id": "my-agent",
  "name": "我的助手",
  "structured_prompt": { ... }
}

// 使用自定义配置
{
  "id": "my-agent",
  "name": "我的助手",
  "structured_prompt": { ... },
  "llm_config": {
    "model": "gpt-4-turbo",
    "temperature": 0.3,
    "max_tokens": 8192
  }
}
```

## 内置变量

以下变量由程序自动提供：

| 变量名 | 来源 | 说明 |
|--------|------|------|
| `session_overview` | Session | 会话概要 |
| `session_summary` | Session | 会话总结 |
| `context` | Session | 对话上下文 |
| `global_memory` | Gateway | 全局记忆 |

## 注意事项

- `system_prompt` 字段已废弃，建议使用 `structured_prompt`
- `id` 必须唯一，建议使用 UUID 或有意义的标识符
- `structured_prompt` 中键值对的顺序即为 prompt 组装顺序
- `llm_config` 可选，留空时实例化使用程序默认配置
- 时间字段会在创建/更新时自动设置
- 不要在输出中包含 `_comment` 字段（仅用于模板注释）