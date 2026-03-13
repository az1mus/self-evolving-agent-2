//! 消息处理辅助函数
//!
//! 提供 Gateway 消息处理相关的辅助功能

use crate::llm::ChatMessage;

/// 格式化上下文消息
///
/// 将消息列表格式化为字符串，用于构建多层提示词中的"上下文"部分
pub fn format_context(messages: &[ChatMessage]) -> String {
    messages
        .iter()
        .map(|msg| format!("{}: {}", msg.role, msg.content))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_context() {
        let messages = vec![
            ChatMessage::user("Hello"),
            ChatMessage::assistant("Hi there!"),
        ];
        let result = format_context(&messages);
        assert!(result.contains("user: Hello"));
        assert!(result.contains("assistant: Hi there!"));
    }
}