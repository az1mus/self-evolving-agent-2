use crate::classifier::Classifier;
use crate::cycle_detector::RoutingContext;
use crate::error::RouterError;
use crate::message::{Message, ProcessingType};
use crate::preprocessor::Preprocessor;
use async_trait::async_trait;
use session_manager::{ServerId, Session};

/// 路由器接口
#[async_trait]
pub trait Router: Send + Sync {
    /// 路由消息到目标 Server(s)
    ///
    /// 返回目标 Server ID 列表
    async fn route(
        &self,
        message: &Message,
        session: &Session,
    ) -> Result<Vec<ServerId>, RouterError>;
}

/// 能力匹配路由器
///
/// 根据能力查找对应的 Server
pub struct CapabilityRouter;

impl CapabilityRouter {
    pub fn new() -> Self {
        Self
    }

    /// 从消息中提取能力标识
    fn extract_capability(message: &Message) -> Option<String> {
        // 优先从路由指令中提取
        if let Some(cap) = message.content.target_capability() {
            return Some(format!("capability:{}", cap));
        }

        // 从结构化内容中提取
        if let crate::message::MessageContent::Structured { payload, .. } = &message.content {
            if let Some(obj) = payload.as_object() {
                // 尝试多个可能的字段名
                for field in ["capability", "action", "operation", "target"] {
                    if let Some(value) = obj.get(field) {
                        if let Some(cap) = value.as_str() {
                            return Some(format!("capability:{}", cap));
                        }
                    }
                }
            }
        }

        None
    }
}

impl Default for CapabilityRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Router for CapabilityRouter {
    async fn route(
        &self,
        message: &Message,
        session: &Session,
    ) -> Result<Vec<ServerId>, RouterError> {
        // 提取能力标识
        let capability = Self::extract_capability(message).ok_or_else(|| {
            RouterError::RoutingFailed("No capability found in message".to_string())
        })?;

        // 查找路由表
        let server_id = session
            .routing_table
            .get(&capability)
            .cloned()
            .ok_or_else(|| RouterError::NoCapableServer(capability.clone()))?;

        tracing::debug!(
            message_id = %message.message_id,
            capability = %capability,
            server_id = %server_id,
            "Routed message by capability"
        );

        Ok(vec![server_id])
    }
}

/// 链式路由器
///
/// 按顺序路由到多个 Server (pipeline)
pub struct ChainedRouter {
    servers: Vec<ServerId>,
}

impl ChainedRouter {
    pub fn new(servers: Vec<ServerId>) -> Self {
        Self { servers }
    }
}

#[async_trait]
impl Router for ChainedRouter {
    async fn route(
        &self,
        _message: &Message,
        _session: &Session,
    ) -> Result<Vec<ServerId>, RouterError> {
        if self.servers.is_empty() {
            return Err(RouterError::RoutingFailed(
                "No servers in chain".to_string(),
            ));
        }

        Ok(self.servers.clone())
    }
}

/// 并行路由器
///
/// 同时路由到多个 Server
pub struct ParallelRouter {
    servers: Vec<ServerId>,
}

impl ParallelRouter {
    pub fn new(servers: Vec<ServerId>) -> Self {
        Self { servers }
    }
}

#[async_trait]
impl Router for ParallelRouter {
    async fn route(
        &self,
        _message: &Message,
        _session: &Session,
    ) -> Result<Vec<ServerId>, RouterError> {
        if self.servers.is_empty() {
            return Err(RouterError::RoutingFailed(
                "No servers in parallel router".to_string(),
            ));
        }

        Ok(self.servers.clone())
    }
}

/// 有机消息路由器
///
/// 将有机处理的消息路由到 LLM Gateway
pub struct OrganicRouter {
    /// LLM Gateway Server ID
    llm_gateway_id: ServerId,
}

impl OrganicRouter {
    /// 尝试查找 LLM Gateway Server
    fn find_llm_gateway(session: &Session) -> Option<ServerId> {
        // 在 Session 的 Server 中查找 LLM Gateway
        for (server_id, server_info) in &session.servers {
            if let Some(server_type) = server_info.metadata.get("server_type") {
                if let Some(type_str) = server_type.as_str() {
                    // 支持两种格式：llm_gateway 和 llmgateway
                    if type_str == "llm_gateway" || type_str == "llmgateway" {
                        return Some(server_id.clone());
                    }
                }
            }
        }
        None
    }
}

#[async_trait]
impl Router for OrganicRouter {
    async fn route(
        &self,
        _message: &Message,
        session: &Session,
    ) -> Result<Vec<ServerId>, RouterError> {
        // 尝试查找 LLM Gateway
        let server_id = Self::find_llm_gateway(session)
            .unwrap_or_else(|| self.llm_gateway_id.clone());

        tracing::debug!(
            server_id = %server_id,
            "Routed organic message to LLM Gateway"
        );

        Ok(vec![server_id])
    }
}

/// 组合路由器
///
/// 按优先级尝试多个路由策略
pub struct CompositeRouter {
    routers: Vec<Box<dyn Router>>,
}

impl CompositeRouter {
    pub fn new() -> Self {
        Self {
            routers: Vec::new(),
        }
    }

    pub fn with(mut self, router: Box<dyn Router>) -> Self {
        self.routers.push(router);
        self
    }

    /// 创建默认组合路由器
    pub fn default_composite() -> Self {
        Self::new().with(Box::new(CapabilityRouter::new()))
    }
}

impl Default for CompositeRouter {
    fn default() -> Self {
        Self::default_composite()
    }
}

#[async_trait]
impl Router for CompositeRouter {
    async fn route(
        &self,
        message: &Message,
        session: &Session,
    ) -> Result<Vec<ServerId>, RouterError> {
        // 如果是有机处理消息，尝试找 LLM Gateway
        if let Some(ProcessingType::Organic) = message.routing.processing_type {
            // 查找 LLM Gateway
            if let Some(llm_id) = OrganicRouter::find_llm_gateway(session) {
                tracing::info!(
                    message_id = %message.message_id,
                    server_id = %llm_id,
                    "Routed organic message to LLM Gateway"
                );
                return Ok(vec![llm_id]);
            }
        }

        // 否则使用能力路由
        for router in &self.routers {
            match router.route(message, session).await {
                Ok(servers) => return Ok(servers),
                Err(RouterError::NoCapableServer(_)) => continue, // 尝试下一个路由器
                Err(e) => return Err(e),                          // 其他错误直接返回
            }
        }

        Err(RouterError::RoutingFailed("All routers failed".to_string()))
    }
}

/// Router Core 主结构
pub struct RouterCore {
    /// 判定器
    classifier: Box<dyn Classifier>,
    /// 预处理器列表
    preprocessors: Vec<Box<dyn Preprocessor>>,
    /// 路由器
    router: Box<dyn Router>,
    /// 路由上下文
    routing_context: RoutingContext,
}

impl RouterCore {
    pub fn new(classifier: Box<dyn Classifier>, router: Box<dyn Router>) -> Self {
        Self {
            classifier,
            preprocessors: Vec::new(),
            router,
            routing_context: RoutingContext::new(),
        }
    }

    /// 添加预处理器
    pub fn with_preprocessor(mut self, preprocessor: Box<dyn Preprocessor>) -> Self {
        self.preprocessors.push(preprocessor);
        self
    }

    /// 处理消息
    ///
    /// 流程:
    /// 1. 判定处理类型
    /// 2. 预处理(如果是有机处理)
    /// 3. 路由选择
    /// 4. 循环检测
    /// 5. 返回目标 Server 列表
    pub async fn process(
        &self,
        mut message: Message,
        session: &Session,
    ) -> Result<Vec<ServerId>, RouterError> {
        // Step 1: 判定处理类型
        let processing_type = self.classifier.classify(&message).await?;
        message = message.with_processing_type(processing_type);

        tracing::info!(
            message_id = %message.message_id,
            processing_type = %processing_type,
            "Classified message"
        );

        // Step 2: 预处理(有机处理)
        if processing_type == ProcessingType::Organic {
            for preprocessor in &self.preprocessors {
                message = preprocessor.preprocess(message, session).await?;
            }
        }

        // Step 3: 路由选择
        let target_servers = self.router.route(&message, session).await?;

        tracing::info!(
            message_id = %message.message_id,
            target_count = target_servers.len(),
            "Routed message to servers"
        );

        // Step 4: 循环检测
        for server_id in &target_servers {
            self.routing_context.can_route_to(&message, server_id)?;
        }

        Ok(target_servers)
    }
}

/// Router Builder
pub struct RouterBuilder {
    classifier: Option<Box<dyn Classifier>>,
    preprocessors: Vec<Box<dyn Preprocessor>>,
    router: Option<Box<dyn Router>>,
}

impl RouterBuilder {
    pub fn new() -> Self {
        Self {
            classifier: None,
            preprocessors: Vec::new(),
            router: None,
        }
    }

    pub fn classifier(mut self, classifier: Box<dyn Classifier>) -> Self {
        self.classifier = Some(classifier);
        self
    }

    pub fn preprocessor(mut self, preprocessor: Box<dyn Preprocessor>) -> Self {
        self.preprocessors.push(preprocessor);
        self
    }

    pub fn router(mut self, router: Box<dyn Router>) -> Self {
        self.router = Some(router);
        self
    }

    pub fn build(self) -> Result<RouterCore, RouterError> {
        let classifier = self
            .classifier
            .ok_or_else(|| RouterError::RoutingFailed("Classifier not set".to_string()))?;

        let router = self
            .router
            .ok_or_else(|| RouterError::RoutingFailed("Router not set".to_string()))?;

        let mut core = RouterCore::new(classifier, router);
        for preprocessor in self.preprocessors {
            core = core.with_preprocessor(preprocessor);
        }

        Ok(core)
    }
}

impl Default for RouterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classifier::MockClassifier;
    use crate::preprocessor::RuleCompressor;
    use session_manager::{ServerInfo, ServerStatus, SessionManager};
    use std::collections::HashMap;
    use tempfile::TempDir;
    use uuid::Uuid;

    fn create_test_session_with_route() -> (TempDir, SessionManager, Session) {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path());
        let mut session = manager.create_session().unwrap();

        // 添加 server
        session.servers.insert(
            "server-a".to_string(),
            ServerInfo {
                id: "server-a".to_string(),
                name: "test-server".to_string(),
                status: ServerStatus::Active,
                tools: vec!["code_review".to_string()],
                metadata: HashMap::new(),
                draining_since: None,
            },
        );

        // 添加路由
        session
            .routing_table
            .insert("capability:code_review".to_string(), "server-a".to_string());

        manager.save_session(&session).unwrap();

        (temp_dir, manager, session)
    }

    fn create_test_message(content: crate::message::MessageContent) -> Message {
        let session_id = Uuid::new_v4();
        Message::simple(session_id, content)
    }

    #[tokio::test]
    async fn test_capability_router() {
        let router = CapabilityRouter::new();
        let (_temp_dir, _manager, session) = create_test_session_with_route();

        let msg = create_test_message(crate::message::MessageContent::routing_command(
            "code_review",
        ));

        let servers = router.route(&msg, &session).await.unwrap();
        assert_eq!(servers, vec!["server-a"]);
    }

    #[tokio::test]
    async fn test_capability_router_structured() {
        let router = CapabilityRouter::new();
        let (_temp_dir, _manager, session) = create_test_session_with_route();

        let msg = create_test_message(crate::message::MessageContent::structured(
            serde_json::json!({"capability": "code_review"}),
        ));

        let servers = router.route(&msg, &session).await.unwrap();
        assert_eq!(servers, vec!["server-a"]);
    }

    #[tokio::test]
    async fn test_capability_router_no_capable_server() {
        let router = CapabilityRouter::new();
        let (_temp_dir, _manager, session) = create_test_session_with_route();

        let msg = create_test_message(crate::message::MessageContent::routing_command("unknown"));

        let result = router.route(&msg, &session).await;
        assert!(matches!(result, Err(RouterError::NoCapableServer(_))));
    }

    #[tokio::test]
    async fn test_chained_router() {
        let router = ChainedRouter::new(vec!["server-a".to_string(), "server-b".to_string()]);

        let session = Session::new();
        let msg = create_test_message(crate::message::MessageContent::unstructured("test"));

        let servers = router.route(&msg, &session).await.unwrap();
        assert_eq!(servers.len(), 2);
    }

    #[tokio::test]
    async fn test_parallel_router() {
        let router = ParallelRouter::new(vec!["server-a".to_string(), "server-b".to_string()]);

        let session = Session::new();
        let msg = create_test_message(crate::message::MessageContent::unstructured("test"));

        let servers = router.route(&msg, &session).await.unwrap();
        assert_eq!(servers.len(), 2);
    }

    #[tokio::test]
    async fn test_composite_router() {
        let router = CompositeRouter::default_composite();
        let (_temp_dir, _manager, session) = create_test_session_with_route();

        let msg = create_test_message(crate::message::MessageContent::routing_command(
            "code_review",
        ));

        let servers = router.route(&msg, &session).await.unwrap();
        assert_eq!(servers, vec!["server-a"]);
    }

    #[tokio::test]
    async fn test_router_core_inorganic() {
        let core = RouterCore::new(
            Box::new(MockClassifier::inorganic()),
            Box::new(CapabilityRouter::new()),
        );

        let (_temp_dir, _manager, session) = create_test_session_with_route();
        let msg = create_test_message(crate::message::MessageContent::routing_command(
            "code_review",
        ));

        let servers = core.process(msg, &session).await.unwrap();
        assert_eq!(servers, vec!["server-a"]);
    }

    #[tokio::test]
    async fn test_router_core_organic() {
        let core = RouterCore::new(
            Box::new(MockClassifier::organic()),
            Box::new(CapabilityRouter::new()),
        )
        .with_preprocessor(Box::new(RuleCompressor::new()));

        let (_temp_dir, _manager, session) = create_test_session_with_route();

        let msg = create_test_message(crate::message::MessageContent::structured(
            serde_json::json!({"capability": "code_review", "data": "extra"}),
        ));

        let servers = core.process(msg, &session).await.unwrap();
        assert_eq!(servers, vec!["server-a"]);
    }

    #[tokio::test]
    async fn test_router_builder() {
        let core = RouterBuilder::new()
            .classifier(Box::new(MockClassifier::inorganic()))
            .router(Box::new(CapabilityRouter::new()))
            .build()
            .unwrap();

        let (_temp_dir, _manager, session) = create_test_session_with_route();
        let msg = create_test_message(crate::message::MessageContent::routing_command(
            "code_review",
        ));

        let servers = core.process(msg, &session).await.unwrap();
        assert_eq!(servers, vec!["server-a"]);
    }
}
