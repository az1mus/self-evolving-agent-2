//! 结构化提示词模块
//!
//! 实现可自由扩展的提示词构建系统。
//!
//! ## 核心概念
//!
//! - **StructuredPrompt**: 动态 key-value 结构，格式为 `{"prompt1": "xx", "prompt2": "xx"}`
//!   - 使用 IndexMap 保留 JSON 键值对的插入顺序
//! - **PromptTemplate**: 定义 prompt 的组装格式（分隔符、标题等）
//! - **PromptContext**: 运行时变量，用于替换 `{{变量名}}`
//! - **PromptBuilder**: 构建最终的系统提示词
//!
//! ## 变量替换
//!
//! - `{{变量名}}`: 访问程序暴露的变量，在 Session 中维护
//! - `{{函数名(参数)}}`: 调用 Agent 可访问的函数（待实现）
//!
//! ## 示例
//!
//! ```json
//! {
//!   "structured_prompt": {
//!     "role": "你是一个{{role_name}}",
//!     "style": "回答要{{style}}",
//!     "instruction": "请使用中文回答"
//!   }
//! }
//! ```
//!
//! JSON 中键值对的顺序即为 prompt 组装的顺序。

use anyhow::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ==================== 结构化提示词 ====================

/// 结构化提示词
///
/// 动态 key-value 结构，支持任意数量的 prompt 组件。
/// 使用 IndexMap 保留 JSON 键值对的插入顺序。
/// 值中可包含 `{{变量名}}` 用于运行时替换。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StructuredPrompt(IndexMap<String, String>);

impl StructuredPrompt {
    /// 创建新的结构化提示词
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 IndexMap 创建
    pub fn from_map(map: IndexMap<String, String>) -> Self {
        Self(map)
    }

    /// 从 HashMap 创建（不保证顺序）
    pub fn from_hashmap(map: HashMap<String, String>) -> Self {
        Self(map.into_iter().collect())
    }

    /// 添加一个 prompt 组件（链式调用）
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.0.insert(key.into(), value.into());
        self
    }

    /// 获取 prompt 组件
    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }

    /// 设置 prompt 组件
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0.insert(key.into(), value.into());
    }

    /// 移除 prompt 组件
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.0.shift_remove(key)
    }

    /// 获取所有 key（按插入顺序）
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.0.keys()
    }

    /// 获取所有条目（按插入顺序）
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.0.iter()
    }

    /// 组件数量
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// 获取有序的 key 列表
    pub fn ordered_keys(&self) -> Vec<&String> {
        self.0.keys().collect()
    }

    /// 转换为 JSON 字符串
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.0)?)
    }

    /// 从 JSON 字符串解析（保留顺序）
    pub fn from_json(json: &str) -> Result<Self> {
        let map: IndexMap<String, String> = serde_json::from_str(json)?;
        Ok(Self(map))
    }
}

impl std::ops::Deref for StructuredPrompt {
    type Target = IndexMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// ==================== 提示词模板 ====================

/// 提示词模板配置
///
/// 定义 prompt 的组装格式。
/// 注意：prompt 组件的顺序由 StructuredPrompt 的键值对顺序决定，
/// 而不是由 template.order 决定。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// 组件之间的分隔符
    #[serde(default = "default_separator")]
    pub separator: String,

    /// 是否添加标题
    /// 如果为 true，每个组件前会添加 `## {key}` 格式的标题
    #[serde(default = "default_include_titles")]
    pub include_titles: bool,

    /// 标题格式，`{title}` 会被替换为 key
    #[serde(default = "default_title_format")]
    pub title_format: String,

    /// 禁用的组件列表
    #[serde(default)]
    pub disabled: Vec<String>,
}

fn default_separator() -> String {
    "\n\n".to_string()
}

fn default_include_titles() -> bool {
    true
}

fn default_title_format() -> String {
    "## {title}".to_string()
}

impl Default for PromptTemplate {
    fn default() -> Self {
        Self {
            separator: default_separator(),
            include_titles: default_include_titles(),
            title_format: default_title_format(),
            disabled: Vec::new(),
        }
    }
}

impl PromptTemplate {
    /// 创建新的模板
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置分隔符
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    /// 设置是否包含标题
    pub fn with_titles(mut self, include: bool) -> Self {
        self.include_titles = include;
        self
    }

    /// 设置标题格式
    pub fn with_title_format(mut self, format: impl Into<String>) -> Self {
        self.title_format = format.into();
        self
    }

    /// 禁用组件
    pub fn disable(mut self, key: impl Into<String>) -> Self {
        self.disabled.push(key.into());
        self
    }

    /// 检查组件是否被禁用
    pub fn is_disabled(&self, key: &str) -> bool {
        self.disabled.contains(&key.to_string())
    }
}

// ==================== 提示词上下文 ====================

/// 提示词上下文
///
/// 运行时变量，用于替换 `{{变量名}}`
#[derive(Debug, Clone, Default)]
pub struct PromptContext {
    /// 变量映射
    variables: HashMap<String, String>,
}

impl PromptContext {
    /// 创建新的上下文
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 HashMap 创建
    pub fn from_map(map: HashMap<String, String>) -> Self {
        Self { variables: map }
    }

    /// 设置变量（链式调用）
    pub fn set(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// 设置变量（可变引用）
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(key.into(), value.into());
    }

    /// 获取变量
    pub fn get(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// 移除变量
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.variables.remove(key)
    }

    /// 获取所有变量
    pub fn variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

    /// 合并另一个上下文
    pub fn merge(&mut self, other: &PromptContext) {
        for (k, v) in other.variables.iter() {
            self.variables.insert(k.clone(), v.clone());
        }
    }
}

// ==================== 提示词构建器 ====================

/// 提示词构建器
///
/// 负责将 StructuredPrompt 和 PromptContext 组装成最终的系统提示词。
/// 组件顺序由 StructuredPrompt 的键值对插入顺序决定。
pub struct PromptBuilder {
    template: PromptTemplate,
}

impl PromptBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            template: PromptTemplate::default(),
        }
    }

    /// 使用自定义模板
    pub fn with_template(template: PromptTemplate) -> Self {
        Self { template }
    }

    /// 获取模板
    pub fn template(&self) -> &PromptTemplate {
        &self.template
    }

    /// 获取可变模板
    pub fn template_mut(&mut self) -> &mut PromptTemplate {
        &mut self.template
    }

    /// 构建系统提示词
    ///
    /// 1. 按照 StructuredPrompt 的键值对顺序组装 prompt 组件
    /// 2. 替换 `{{变量名}}` 为 context 中的值
    /// 3. 如果 include_titles 为 true，添加标题
    pub fn build(&self, prompt: &StructuredPrompt, context: &PromptContext) -> String {
        let mut sections = Vec::new();

        // 按照 IndexMap 的插入顺序遍历
        for (key, content) in prompt.iter() {
            // 跳过禁用的组件
            if self.template.is_disabled(key) {
                continue;
            }

            // 替换变量
            let resolved = self.resolve_variables(content, context);

            if !resolved.is_empty() {
                let formatted = if self.template.include_titles {
                    let title = self.template.title_format.replace("{title}", key);
                    format!("{}\n\n{}", title, resolved)
                } else {
                    resolved
                };
                sections.push(formatted);
            }
        }

        sections.join(&self.template.separator)
    }

    /// 替换变量
    ///
    /// 将 `{{变量名}}` 替换为 context 中的值
    /// 未找到的变量保持原样
    fn resolve_variables(&self, content: &str, context: &PromptContext) -> String {
        let mut result = content.to_string();
        let mut start = 0;

        while let Some(begin) = result[start..].find("{{") {
            let begin = start + begin;
            if let Some(end) = result[begin..].find("}}") {
                let end = begin + end + 2;
                let placeholder = &result[begin + 2..end - 2].trim();

                // 解析变量名或函数调用
                let replacement = if placeholder.contains('(') {
                    // 函数调用: {{func(arg)}}
                    self.resolve_function(placeholder, context)
                } else {
                    // 简单变量: {{var}}
                    context.get(placeholder).cloned().unwrap_or_else(|| {
                        // 保持原样
                        format!("{{{{{}}}}}", placeholder)
                    })
                };

                result.replace_range(begin..end, &replacement);
                start = begin + replacement.len();
            } else {
                break;
            }
        }

        result
    }

    /// 解析函数调用
    ///
    /// 格式: `{{函数名(参数)}}`
    /// TODO: 实现函数调用机制
    fn resolve_function(&self, expr: &str, _context: &PromptContext) -> String {
        // 解析函数名和参数
        if let Some(paren_pos) = expr.find('(') {
            let func_name = expr[..paren_pos].trim();
            let args_str = &expr[paren_pos + 1..expr.len() - 1].trim();

            // TODO: 实现函数注册和调用
            // 目前返回占位符
            format!("[函数: {}({})]", func_name, args_str)
        } else {
            format!("[无效函数调用: {}]", expr)
        }
    }

    /// 构建简化的提示词（无标题，仅非空内容）
    pub fn build_simple(&self, prompt: &StructuredPrompt, context: &PromptContext) -> String {
        let mut parts = Vec::new();

        for (key, content) in prompt.iter() {
            if self.template.is_disabled(key) {
                continue;
            }

            let resolved = self.resolve_variables(content, context);
            if !resolved.is_empty() {
                parts.push(resolved);
            }
        }

        parts.join(&self.template.separator)
    }
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== 辅助函数 ====================

/// 解析变量引用
///
/// 从文本中提取所有 `{{变量名}}` 格式的变量名
pub fn extract_variables(text: &str) -> Vec<String> {
    let mut variables = Vec::new();
    let mut start = 0;

    while let Some(begin) = text[start..].find("{{") {
        let begin = start + begin;
        if let Some(end) = text[begin..].find("}}") {
            let end = begin + end;
            let var_name = text[begin + 2..end].trim();

            // 如果是函数调用，提取函数名
            let name = if var_name.contains('(') {
                var_name.split('(').next().unwrap_or(var_name).trim()
            } else {
                var_name
            };

            if !variables.contains(&name.to_string()) {
                variables.push(name.to_string());
            }
            start = end + 2;
        } else {
            break;
        }
    }

    variables
}

// ==================== 测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_prompt_basic() {
        let prompt = StructuredPrompt::new()
            .with("role", "你是一个助手")
            .with("instruction", "请使用中文");

        assert_eq!(prompt.len(), 2);
        assert_eq!(prompt.get("role"), Some(&"你是一个助手".to_string()));
        assert_eq!(prompt.get("instruction"), Some(&"请使用中文".to_string()));
    }

    #[test]
    fn test_structured_prompt_order_preserved() {
        let prompt = StructuredPrompt::new()
            .with("first", "第一个")
            .with("second", "第二个")
            .with("third", "第三个");

        let keys: Vec<_> = prompt.ordered_keys();
        assert_eq!(keys, vec![&"first".to_string(), &"second".to_string(), &"third".to_string()]);
    }

    #[test]
    fn test_structured_prompt_json_order() {
        let json = r#"{"a": "1", "b": "2", "c": "3"}"#;
        let prompt = StructuredPrompt::from_json(json).unwrap();

        let keys: Vec<_> = prompt.ordered_keys();
        assert_eq!(keys, vec![&"a".to_string(), &"b".to_string(), &"c".to_string()]);
    }

    #[test]
    fn test_structured_prompt_json() {
        let prompt = StructuredPrompt::new()
            .with("role", "你是一个助手")
            .with("instruction", "请使用中文");

        let json = prompt.to_json().unwrap();
        let parsed = StructuredPrompt::from_json(&json).unwrap();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed.get("role"), Some(&"你是一个助手".to_string()));
    }

    #[test]
    fn test_prompt_context() {
        let ctx = PromptContext::new()
            .set("name", "Alice")
            .set("role", "工程师");

        assert_eq!(ctx.get("name"), Some(&"Alice".to_string()));
        assert_eq!(ctx.get("role"), Some(&"工程师".to_string()));
    }

    #[test]
    fn test_prompt_builder_basic() {
        let prompt = StructuredPrompt::new()
            .with("role", "你是一个助手")
            .with("instruction", "请使用中文");

        let ctx = PromptContext::new();
        let builder = PromptBuilder::new();
        let result = builder.build(&prompt, &ctx);

        assert!(result.contains("role"));
        assert!(result.contains("你是一个助手"));
        assert!(result.contains("instruction"));
        assert!(result.contains("请使用中文"));
    }

    #[test]
    fn test_prompt_builder_order() {
        let prompt = StructuredPrompt::new()
            .with("third", "第三个")
            .with("first", "第一个")
            .with("second", "第二个");

        let builder = PromptBuilder::with_template(
            PromptTemplate::new().with_titles(false)
        );
        let ctx = PromptContext::new();
        let result = builder.build(&prompt, &ctx);

        // 验证顺序：third -> first -> second
        let third_pos = result.find("第三个").unwrap();
        let first_pos = result.find("第一个").unwrap();
        let second_pos = result.find("第二个").unwrap();

        assert!(third_pos < first_pos);
        assert!(first_pos < second_pos);
    }

    #[test]
    fn test_prompt_builder_with_variables() {
        let prompt = StructuredPrompt::new()
            .with("role", "你是一个{{role_name}}")
            .with("greeting", "你好，{{user_name}}！");

        let ctx = PromptContext::new()
            .set("role_name", "翻译助手")
            .set("user_name", "Alice");

        let builder = PromptBuilder::with_template(
            PromptTemplate::new().with_titles(false)
        );
        let result = builder.build(&prompt, &ctx);

        assert!(result.contains("你是一个翻译助手"));
        assert!(result.contains("你好，Alice！"));
    }

    #[test]
    fn test_prompt_builder_missing_variable() {
        let prompt = StructuredPrompt::new()
            .with("role", "你是一个{{role_name}}");

        let ctx = PromptContext::new(); // 没有设置 role_name

        let builder = PromptBuilder::with_template(
            PromptTemplate::new().with_titles(false)
        );
        let result = builder.build(&prompt, &ctx);

        // 未找到的变量保持原样
        assert!(result.contains("你是一个{{role_name}}"));
    }

    #[test]
    fn test_prompt_builder_disabled() {
        let prompt = StructuredPrompt::new()
            .with("role", "你是一个助手")
            .with("instruction", "请使用中文");

        let template = PromptTemplate::new()
            .disable("instruction");

        let builder = PromptBuilder::with_template(template);
        let ctx = PromptContext::new();
        let result = builder.build(&prompt, &ctx);

        assert!(result.contains("你是一个助手"));
        assert!(!result.contains("请使用中文"));
    }

    #[test]
    fn test_extract_variables() {
        let text = "你是一个{{role}}，你好{{user}}！今天是{{date}}。";
        let vars = extract_variables(text);

        assert_eq!(vars.len(), 3);
        assert!(vars.contains(&"role".to_string()));
        assert!(vars.contains(&"user".to_string()));
        assert!(vars.contains(&"date".to_string()));
    }

    #[test]
    fn test_extract_variables_with_function() {
        let text = "时间：{{get_time()}}，用户：{{user}}";
        let vars = extract_variables(text);

        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"get_time".to_string()));
        assert!(vars.contains(&"user".to_string()));
    }

    #[test]
    fn test_prompt_builder_no_titles() {
        let prompt = StructuredPrompt::new()
            .with("role", "你是一个助手")
            .with("instruction", "请使用中文");

        let template = PromptTemplate::new()
            .with_titles(false)
            .with_separator("\n---\n");

        let builder = PromptBuilder::with_template(template);
        let ctx = PromptContext::new();
        let result = builder.build(&prompt, &ctx);

        assert!(!result.contains("## role"));
        assert!(result.contains("你是一个助手"));
        assert!(result.contains("---"));
    }
}