use crate::error::RouterError;
use crate::message::{Message, ProcessingType};
use async_trait::async_trait;

/// 判定器接口
#[async_trait]
pub trait Classifier: Send + Sync {
    /// 判定消息的处理类型
    async fn classify(&self, message: &Message) -> Result<ProcessingType, RouterError>;
}

/// 基于规则的判定器
///
/// 判定规则:
/// - 路由指令 → 无机处理
/// - 结构化内容且包含明确字段 → 无机处理
/// - 其他 → 有机处理
pub struct RuleBasedClassifier {
    /// 判定为无机处理的结构化字段
    inorganic_fields: Vec<String>,
}

impl RuleBasedClassifier {
    /// 创建规则判定器
    pub fn new() -> Self {
        Self {
            inorganic_fields: vec![
                "action".to_string(),
                "command".to_string(),
                "operation".to_string(),
                "target".to_string(),
                "route_to".to_string(),
            ],
        }
    }

    /// 添加自定义字段
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.inorganic_fields.push(field.into());
        self
    }

    /// 检查结构化内容是否包含无机处理字段
    fn has_inorganic_fields(&self, payload: &serde_json::Value) -> bool {
        if let Some(obj) = payload.as_object() {
            for field in &self.inorganic_fields {
                if obj.contains_key(field) {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for RuleBasedClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Classifier for RuleBasedClassifier {
    async fn classify(&self, message: &Message) -> Result<ProcessingType, RouterError> {
        // 规则 1: 路由指令 → 无机处理
        if message.content.is_routing_command() {
            tracing::debug!(
                message_id = %message.message_id,
                "Classified as inorganic: routing command"
            );
            return Ok(ProcessingType::Inorganic);
        }

        // 规则 2: 结构化内容检查
        if let crate::message::MessageContent::Structured { payload, .. } = &message.content {
            if self.has_inorganic_fields(payload) {
                tracing::debug!(
                    message_id = %message.message_id,
                    "Classified as inorganic: structured with inorganic fields"
                );
                return Ok(ProcessingType::Inorganic);
            }
        }

        // 规则 3: 默认 → 有机处理
        tracing::debug!(
            message_id = %message.message_id,
            "Classified as organic: default"
        );
        Ok(ProcessingType::Organic)
    }
}

/// Mock 判定器(用于测试)
pub struct MockClassifier {
    processing_type: ProcessingType,
}

impl MockClassifier {
    /// 创建 Mock 判定器
    pub fn new(processing_type: ProcessingType) -> Self {
        Self { processing_type }
    }

    /// 创建有机处理判定器
    pub fn organic() -> Self {
        Self::new(ProcessingType::Organic)
    }

    /// 创建无机处理判定器
    pub fn inorganic() -> Self {
        Self::new(ProcessingType::Inorganic)
    }
}

#[async_trait]
impl Classifier for MockClassifier {
    async fn classify(&self, _message: &Message) -> Result<ProcessingType, RouterError> {
        Ok(self.processing_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_message(content: crate::message::MessageContent) -> Message {
        let session_id = Uuid::new_v4();
        Message::simple(session_id, content)
    }

    #[tokio::test]
    async fn test_rule_based_classifier_routing_command() {
        let classifier = RuleBasedClassifier::new();
        let msg = create_test_message(crate::message::MessageContent::routing_command(
            "code_review",
        ));

        let result = classifier.classify(&msg).await.unwrap();
        assert_eq!(result, ProcessingType::Inorganic);
    }

    #[tokio::test]
    async fn test_rule_based_classifier_structured_inorganic() {
        let classifier = RuleBasedClassifier::new();
        let msg = create_test_message(crate::message::MessageContent::structured(
            serde_json::json!({"action": "execute", "target": "server-a"}),
        ));

        let result = classifier.classify(&msg).await.unwrap();
        assert_eq!(result, ProcessingType::Inorganic);
    }

    #[tokio::test]
    async fn test_rule_based_classifier_structured_organic() {
        let classifier = RuleBasedClassifier::new();
        let msg = create_test_message(crate::message::MessageContent::structured(
            serde_json::json!({"description": "analyze this code"}),
        ));

        let result = classifier.classify(&msg).await.unwrap();
        assert_eq!(result, ProcessingType::Organic);
    }

    #[tokio::test]
    async fn test_rule_based_classifier_unstructured() {
        let classifier = RuleBasedClassifier::new();
        let msg = create_test_message(crate::message::MessageContent::unstructured(
            "please help me understand this code",
        ));

        let result = classifier.classify(&msg).await.unwrap();
        assert_eq!(result, ProcessingType::Organic);
    }

    #[tokio::test]
    async fn test_rule_based_classifier_custom_field() {
        let classifier = RuleBasedClassifier::new().with_field("custom_action");
        let msg = create_test_message(crate::message::MessageContent::structured(
            serde_json::json!({"custom_action": "run"}),
        ));

        let result = classifier.classify(&msg).await.unwrap();
        assert_eq!(result, ProcessingType::Inorganic);
    }

    #[tokio::test]
    async fn test_mock_classifier_organic() {
        let classifier = MockClassifier::organic();
        let msg = create_test_message(crate::message::MessageContent::unstructured("test"));

        let result = classifier.classify(&msg).await.unwrap();
        assert_eq!(result, ProcessingType::Organic);
    }

    #[tokio::test]
    async fn test_mock_classifier_inorganic() {
        let classifier = MockClassifier::inorganic();
        let msg = create_test_message(crate::message::MessageContent::unstructured("test"));

        let result = classifier.classify(&msg).await.unwrap();
        assert_eq!(result, ProcessingType::Inorganic);
    }
}
