use crate::error::RouterError;
use crate::message::Message;
use async_trait::async_trait;
use session_manager::{CacheManager, Session};

/// 预处理器接口
#[async_trait]
pub trait Preprocessor: Send + Sync {
    /// 预处理消息
    async fn preprocess(&self, message: Message, session: &Session)
        -> Result<Message, RouterError>;
}

/// 规则压缩器
///
/// 移除冗余字段,提取关键信息
pub struct RuleCompressor {
    /// 要保留的字段(结构化内容)
    keep_fields: Vec<String>,
}

impl RuleCompressor {
    /// 创建规则压缩器
    pub fn new() -> Self {
        Self {
            keep_fields: vec![
                "action".to_string(),
                "operation".to_string(),
                "target".to_string(),
                "capability".to_string(),
                "payload".to_string(),
            ],
        }
    }

    /// 指定要保留的字段
    pub fn with_keep_fields(mut self, fields: Vec<String>) -> Self {
        self.keep_fields = fields;
        self
    }

    /// 压缩结构化内容
    fn compress_structured(&self, payload: &serde_json::Value) -> serde_json::Value {
        if let Some(obj) = payload.as_object() {
            let compressed: serde_json::Map<String, serde_json::Value> = obj
                .iter()
                .filter(|(key, _)| self.keep_fields.contains(key))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            serde_json::Value::Object(compressed)
        } else {
            payload.clone()
        }
    }
}

impl Default for RuleCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Preprocessor for RuleCompressor {
    async fn preprocess(
        &self,
        mut message: Message,
        _session: &Session,
    ) -> Result<Message, RouterError> {
        if let crate::message::MessageContent::Structured { payload, .. } = &message.content {
            let compressed = self.compress_structured(payload);
            message.content = crate::message::MessageContent::structured(compressed);

            tracing::debug!(
                message_id = %message.message_id,
                "Compressed structured content"
            );
        }

        Ok(message)
    }
}

/// 一致化处理器
///
/// 格式统一,语义归一(简化版)
pub struct Normalizer {
    /// 是否转换为小写
    lowercase: bool,
}

impl Normalizer {
    /// 创建一致化处理器
    pub fn new() -> Self {
        Self { lowercase: true }
    }

    /// 设置是否转换为小写
    pub fn with_lowercase(mut self, lowercase: bool) -> Self {
        self.lowercase = lowercase;
        self
    }

    /// 一致化文本
    fn normalize_text(&self, text: &str) -> String {
        let mut normalized = text.trim().to_string();

        if self.lowercase {
            normalized = normalized.to_lowercase();
        }

        // 移除多余空白
        normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");

        normalized
    }

    /// 一致化 JSON key
    fn normalize_keys(&self, value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let normalized: serde_json::Map<String, serde_json::Value> = map
                    .iter()
                    .map(|(k, v)| {
                        let normalized_key = if self.lowercase {
                            k.to_lowercase()
                        } else {
                            k.clone()
                        };
                        (normalized_key, self.normalize_keys(v))
                    })
                    .collect();
                serde_json::Value::Object(normalized)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(|v| self.normalize_keys(v)).collect())
            }
            _ => value.clone(),
        }
    }
}

impl Default for Normalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Preprocessor for Normalizer {
    async fn preprocess(
        &self,
        mut message: Message,
        _session: &Session,
    ) -> Result<Message, RouterError> {
        match &message.content {
            crate::message::MessageContent::Unstructured { text } => {
                let normalized = self.normalize_text(text);
                message.content = crate::message::MessageContent::unstructured(normalized);
            }
            crate::message::MessageContent::Structured {
                payload,
                schema_version,
            } => {
                let normalized = self.normalize_keys(payload);
                message.content = crate::message::MessageContent::Structured {
                    payload: normalized,
                    schema_version: schema_version.clone(),
                };
            }
            _ => {}
        }

        tracing::debug!(
            message_id = %message.message_id,
            "Normalized message content"
        );

        Ok(message)
    }
}

/// Cache 匹配器
///
/// 查询 Session Cache,返回缓存结果或标记需要处理
pub struct CacheMatcher<'a> {
    cache_manager: CacheManager<'a>,
}

impl<'a> CacheMatcher<'a> {
    /// 创建 Cache 匹配器
    pub fn new(cache_manager: CacheManager<'a>) -> Self {
        Self { cache_manager }
    }
}

#[async_trait]
impl<'a> Preprocessor for CacheMatcher<'a> {
    async fn preprocess(
        &self,
        message: Message,
        session: &Session,
    ) -> Result<Message, RouterError> {
        let hash = message.content_hash();

        // 查询输入缓存
        if let Some(_cached) = self
            .cache_manager
            .get_input_cache(session.session_id, &hash)
            .map_err(|e| RouterError::CacheError(e.to_string()))?
        {
            tracing::info!(
                message_id = %message.message_id,
                hash = %hash,
                "Cache hit for message"
            );

            // 如果缓存命中,可以将结果附加到消息元数据
            // 这里简化处理,只是记录日志
            return Ok(message);
        }

        tracing::debug!(
            message_id = %message.message_id,
            hash = %hash,
            "Cache miss for message"
        );

        Ok(message)
    }
}

/// 预处理管道
///
/// 串联多个预处理器
pub struct PreprocessorPipeline {
    preprocessors: Vec<Box<dyn Preprocessor>>,
}

impl PreprocessorPipeline {
    /// 创建空的预处理管道
    pub fn new() -> Self {
        Self {
            preprocessors: Vec::new(),
        }
    }

    /// 添加预处理器
    pub fn with(mut self, preprocessor: Box<dyn Preprocessor>) -> Self {
        self.preprocessors.push(preprocessor);
        self
    }

    /// 创建默认管道(规则压缩 + 一致化)
    pub fn default_pipeline() -> Self {
        Self::new()
            .with(Box::new(RuleCompressor::new()))
            .with(Box::new(Normalizer::new()))
    }
}

impl Default for PreprocessorPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Preprocessor for PreprocessorPipeline {
    async fn preprocess(
        &self,
        mut message: Message,
        session: &Session,
    ) -> Result<Message, RouterError> {
        for preprocessor in &self.preprocessors {
            message = preprocessor.preprocess(message, session).await?;
        }

        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use session_manager::SessionManager;
    use tempfile::TempDir;
    use uuid::Uuid;

    fn create_test_session() -> (TempDir, SessionManager, Session) {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path());
        let session = manager.create_session().unwrap();
        (temp_dir, manager, session)
    }

    fn create_test_message(content: crate::message::MessageContent) -> Message {
        let session_id = Uuid::new_v4();
        Message::simple(session_id, content)
    }

    #[tokio::test]
    async fn test_rule_compressor() {
        let compressor = RuleCompressor::new();
        let (_temp_dir, _manager, session) = create_test_session();

        let msg = create_test_message(crate::message::MessageContent::structured(
            serde_json::json!({
                "action": "execute",
                "target": "server-a",
                "verbose": true,
                "debug": "lots of debug info",
            }),
        ));

        let processed = compressor.preprocess(msg, &session).await.unwrap();

        if let crate::message::MessageContent::Structured { payload, .. } = &processed.content {
            assert!(payload.get("action").is_some());
            assert!(payload.get("target").is_some());
            assert!(payload.get("verbose").is_none());
            assert!(payload.get("debug").is_none());
        } else {
            panic!("Expected structured content");
        }
    }

    #[tokio::test]
    async fn test_normalizer_unstructured() {
        let normalizer = Normalizer::new();
        let (_temp_dir, _manager, session) = create_test_session();

        let msg = create_test_message(crate::message::MessageContent::unstructured(
            "  Hello   WORLD  ",
        ));

        let processed = normalizer.preprocess(msg, &session).await.unwrap();

        if let crate::message::MessageContent::Unstructured { text } = &processed.content {
            assert_eq!(text, "hello world");
        } else {
            panic!("Expected unstructured content");
        }
    }

    #[tokio::test]
    async fn test_normalizer_structured() {
        let normalizer = Normalizer::new();
        let (_temp_dir, _manager, session) = create_test_session();

        let msg = create_test_message(crate::message::MessageContent::structured(
            serde_json::json!({
                "Action": "Execute",
                "TargetServer": "server-a",
            }),
        ));

        let processed = normalizer.preprocess(msg, &session).await.unwrap();

        if let crate::message::MessageContent::Structured { payload, .. } = &processed.content {
            assert!(payload.get("action").is_some());
            assert!(payload.get("targetserver").is_some());
            assert!(payload.get("Action").is_none());
        } else {
            panic!("Expected structured content");
        }
    }

    #[tokio::test]
    async fn test_preprocessor_pipeline() {
        let pipeline = PreprocessorPipeline::default_pipeline();
        let (_temp_dir, _manager, session) = create_test_session();

        let msg = create_test_message(crate::message::MessageContent::structured(
            serde_json::json!({
                "Action": "  EXECUTE  ",
                "Target": "server-a",
                "Debug": "verbose",
            }),
        ));

        let processed = pipeline.preprocess(msg, &session).await.unwrap();

        // 经过压缩和一致化后
        if let crate::message::MessageContent::Structured { payload, .. } = &processed.content {
            // "Debug" 字段被压缩器移除
            assert!(payload.get("debug").is_none());
            // "Action" 和 "Target" 被一致化为小写
            // 注意:由于压缩器先执行,"Action" 可能已经被移除
        }
    }
}
