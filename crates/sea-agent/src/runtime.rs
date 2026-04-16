//! SEA Agent 运行时
//!
//! 整合所有模块，提供系统初始化、消息处理和优雅关闭

use std::collections::HashMap;
use std::sync::Arc;

use session_manager::{
    MessageRole, ServerLifecycle as SessionServerLifecycle, SessionManager, SessionId,
};
use router_core::{
    Classifier, Message, PreprocessorPipeline, ProcessingType, Router, RouterBuilder, RouterCore,
};
use concrete_servers::{ServerFactory, ServerRegistry};
use concrete_servers::factory::{ServerConfig, ServerType};

use crate::config::SeaConfig;
use crate::error::{Result, SeaError};

/// SEA Agent 主结构
pub struct SeaAgent {
    /// 配置
    config: SeaConfig,
    /// Session Manager
    session_manager: Arc<SessionManager>,
    /// Router Core
    router_core: RouterCore,
    /// 活跃的 Server
    server_runners: HashMap<String, ServerHolder>,
    /// Server 注册表
    server_registry: ServerRegistry,
}

/// Server 运行时持有者
struct ServerHolder {
    /// Server 类型
    server_type: ServerType,
    /// Server 名称
    name: String,
    /// Session ID
    session_id: SessionId,
    /// 是否正在运行
    running: bool,
}

/// Server 信息（对外展示）
#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub id: String,
    pub name: String,
    pub server_type: ServerType,
    pub session_id: SessionId,
    pub running: bool,
}

/// 消息处理结果
#[derive(Debug, Clone)]
pub struct MessageResult {
    pub processing_type: ProcessingType,
    pub routed_servers: Vec<session_manager::ServerId>,
    pub response: String,
}

impl SeaAgent {
    /// 创建新的 SEA Agent
    pub async fn new(config: SeaConfig) -> Result<Self> {
        // 确保存储目录存在
        std::fs::create_dir_all(&config.session_store_path).map_err(SeaError::Io)?;

        // 初始化 Session Manager
        let session_manager = Arc::new(SessionManager::new(&config.session_store_path));

        // 初始化 Router Core
        let router_core = Self::build_router(&config)?;

        // 初始化 Server Registry
        let server_registry = ServerRegistry::new();

        // 从持久化的 Sessions 中恢复 server_runners
        let server_runners = Self::load_servers_from_sessions(&session_manager)?;

        Ok(Self {
            config,
            session_manager,
            router_core,
            server_runners,
            server_registry,
        })
    }

    /// 从所有 Session 中加载已注册的 Servers 到内存
    fn load_servers_from_sessions(
        session_manager: &SessionManager,
    ) -> Result<HashMap<String, ServerHolder>> {
        use session_manager::ServerStatus;

        let mut server_runners = HashMap::new();

        let sessions = session_manager
            .list_sessions()
            .map_err(SeaError::Session)?;

        for summary in sessions {
            if let Ok(mut session) = session_manager.load_session(summary.session_id) {
                let mut session_modified = false;

                for (server_id, server_info) in session.servers.iter_mut() {
                    // 从 metadata 中解析 server_type
                    if let Some(type_value) = server_info.metadata.get("server_type") {
                        if let Ok(server_type) = serde_json::from_value::<ServerType>(type_value.clone()) {
                            // 进程重启后，running 状态应该与持久化状态同步
                            // 如果持久化状态是 Active，需要重置为 Pending（因为进程重启后 server 实际未运行）
                            let running = if server_info.status == ServerStatus::Active {
                                server_info.status = ServerStatus::Pending;
                                session_modified = true;
                                false
                            } else {
                                false
                            };

                            // 获取 server name
                            let name = server_info.name.clone();

                            server_runners.insert(
                                server_id.clone(),
                                ServerHolder {
                                    server_type,
                                    name,
                                    session_id: session.session_id,
                                    running,
                                },
                            );
                        }
                    }
                }

                // 如果 session 被修改，保存回存储
                if session_modified {
                    session.touch();
                    session_manager
                        .save_session(&session)
                        .map_err(SeaError::Session)?;
                }
            }
        }

        Ok(server_runners)
    }

    /// 构建 Router Core
    fn build_router(config: &SeaConfig) -> Result<RouterCore> {
        let classifier: Box<dyn Classifier> = match &config.router.classifier_type {
            crate::config::ClassifierType::RuleBased => Box::new(
                router_core::RuleBasedClassifier::new()
                    .with_field("task_type")
                    .with_field("payload"),
            ),
            crate::config::ClassifierType::Mock { default_organic: _ } => {
                Box::new(router_core::MockClassifier::organic())
            }
        };

        let preprocessor_pipeline = PreprocessorPipeline::default_pipeline();

        let router: Box<dyn Router> = Box::new(router_core::CompositeRouter::default_composite());

        let router_core = RouterBuilder::new()
            .classifier(classifier)
            .preprocessor(Box::new(preprocessor_pipeline))
            .router(router)
            .build()
            .map_err(SeaError::Router)?;

        Ok(router_core)
    }

    /// 创建新的 Session
    pub async fn create_session(&self) -> Result<SessionId> {
        self.create_session_with_name(None).await
    }

    /// 创建带名称的 Session
    pub async fn create_session_with_name(&self, name: Option<String>) -> Result<SessionId> {
        let session = self
            .session_manager
            .create_session_with_name(name)
            .map_err(SeaError::Session)?;
        Ok(session.session_id)
    }

    /// 列出所有 Session
    pub async fn list_sessions(&self) -> Result<Vec<session_manager::SessionSummary>> {
        let summaries = self
            .session_manager
            .list_sessions()
            .map_err(SeaError::Session)?;
        Ok(summaries)
    }

    /// 显示 Session 详情
    pub async fn show_session(&self, session_id: SessionId) -> Result<session_manager::Session> {
        let session = self
            .session_manager
            .load_session(session_id)
            .map_err(SeaError::Session)?;
        Ok(session)
    }

    /// 删除 Session
    pub async fn delete_session(&self, session_id: SessionId) -> Result<()> {
        self.session_manager
            .delete_session(session_id)
            .map_err(SeaError::Session)?;
        Ok(())
    }

    /// 注册 Server 到 Session
    pub async fn register_server(
        &mut self,
        session_id: SessionId,
        server_type: ServerType,
        server_id: Option<String>,
    ) -> Result<String> {
        self.register_server_with_name(session_id, server_type, server_id, None).await
    }

    /// 注册带名称的 Server 到 Session
    pub async fn register_server_with_name(
        &mut self,
        session_id: SessionId,
        server_type: ServerType,
        server_id: Option<String>,
        server_name: Option<String>,
    ) -> Result<String> {
        let id = server_id.unwrap_or_else(|| {
            format!(
                "{}-{}",
                server_type,
                &uuid::Uuid::new_v4().to_string()[..8]
            )
        });

        let name = server_name.unwrap_or_else(|| format!("{}-server", server_type));

        // 创建 Server 实例（验证可以创建）
        let config = ServerConfig::new(server_type.clone()).with_id(&id);
        let server = ServerFactory::create(config, session_id)
            .map_err(|e| SeaError::Server(e.to_string()))?;

        // 获取 Server 提供的工具
        let tools = server.tools();
        let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();

        // 注册到 Session（将 server_type 存入 metadata 以便重启后恢复）
        let mut metadata = HashMap::new();
        metadata.insert(
            "server_type".to_string(),
            serde_json::to_value(&server_type).unwrap_or_default(),
        );
        metadata.insert(
            "server_name".to_string(),
            serde_json::to_value(&name).unwrap_or_default(),
        );
        let lifecycle = SessionServerLifecycle::new(&*self.session_manager);
        lifecycle
            .register_server_with_name(session_id, id.clone(), Some(name.clone()), tool_names.clone(), metadata)
            .map_err(|e| SeaError::Server(e.to_string()))?;

        // 添加路由表条目
        let lifecycle = SessionServerLifecycle::new(&*self.session_manager);
        for tool_name in tool_names {
            let capability = format!("capability:{}", tool_name);
            lifecycle
                .add_route(session_id, &capability, id.clone())
                .map_err(|e| SeaError::Server(e.to_string()))?;
        }

        // 记录运行时持有者
        self.server_runners.insert(
            id.clone(),
            ServerHolder {
                server_type,
                name,
                session_id,
                running: false,
            },
        );

        Ok(id)
    }

    /// 启动 Server
    pub async fn start_server(&mut self, server_id: &str) -> Result<()> {
        let holder = self
            .server_runners
            .get_mut(server_id)
            .ok_or_else(|| SeaError::NotFound(format!("Server '{}' not found", server_id)))?;

        if holder.running {
            return Err(SeaError::InvalidOperation(format!(
                "Server '{}' is already running",
                server_id
            )));
        }

        holder.running = true;

        // 激活 Session 中的 Server
        let lifecycle = SessionServerLifecycle::new(&*self.session_manager);
        lifecycle
            .activate_server(holder.session_id, server_id)
            .map_err(|e| SeaError::Server(e.to_string()))?;

        tracing::info!("Server '{}' started", server_id);
        Ok(())
    }

    /// 停止 Server
    pub async fn stop_server(&mut self, server_id: &str) -> Result<()> {
        let holder = self
            .server_runners
            .get_mut(server_id)
            .ok_or_else(|| SeaError::NotFound(format!("Server '{}' not found", server_id)))?;

        if !holder.running {
            return Err(SeaError::InvalidOperation(format!(
                "Server '{}' is not running",
                server_id
            )));
        }

        holder.running = false;

        // 在 Session 中将 Server 设为 Draining
        let lifecycle = SessionServerLifecycle::new(&*self.session_manager);
        lifecycle
            .drain_server(holder.session_id, server_id)
            .map_err(|e| SeaError::Server(e.to_string()))?;

        tracing::info!("Server '{}' stopped (draining)", server_id);
        Ok(())
    }

    /// 列出所有 Server
    pub fn list_servers(&self) -> Vec<ServerInfo> {
        self.server_runners
            .iter()
            .map(|(id, h)| ServerInfo {
                id: id.clone(),
                name: h.name.clone(),
                server_type: h.server_type.clone(),
                session_id: h.session_id,
                running: h.running,
            })
            .collect()
    }

    /// 获取 Session 中的 Server 列表
    pub fn list_session_servers(&self, session_id: SessionId) -> Vec<ServerInfo> {
        self.server_runners
            .iter()
            .filter(|(_, h)| h.session_id == session_id)
            .map(|(id, h)| ServerInfo {
                id: id.clone(),
                name: h.name.clone(),
                server_type: h.server_type.clone(),
                session_id: h.session_id,
                running: h.running,
            })
            .collect()
    }

    /// 发送消息到 Session
    ///
    /// 消息内容可以是：
    /// - JSON 格式的结构化消息（会自动路由）
    /// - 纯文本的非结构化消息
    pub async fn send_message(
        &self,
        session_id: SessionId,
        content: &str,
    ) -> Result<MessageResult> {
        // 加载 Session
        let session = self
            .session_manager
            .load_session(session_id)
            .map_err(SeaError::Session)?;

        // 记录用户消息
        self.session_manager
            .add_message(session_id, MessageRole::User, content.to_string())
            .map_err(SeaError::Session)?;

        // 尝试解析为 JSON，如果成功则创建结构化消息
        let message_content = if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(content) {
            router_core::MessageContent::structured(json_value)
        } else {
            router_core::MessageContent::unstructured(content)
        };

        // 创建路由消息
        let message = Message::simple(session_id, message_content);

        // 通过 Router Core 处理
        let processing_type_hint = message.routing.processing_type.clone();
        let result = self.router_core.process(message, &session).await;

        match result {
            Ok(server_ids) => {
                let processing_type = processing_type_hint.unwrap_or(ProcessingType::Inorganic);

                let response = format!(
                    "Routed to {} server(s): {} (processing: {})",
                    server_ids.len(),
                    server_ids.join(", "),
                    processing_type
                );

                // 记录助手消息
                self.session_manager
                    .add_message(session_id, MessageRole::Assistant, response.clone())
                    .map_err(SeaError::Session)?;

                Ok(MessageResult {
                    processing_type,
                    routed_servers: server_ids,
                    response,
                })
            }
            Err(e) => {
                let error_msg = format!("Routing error: {}", e);
                self.session_manager
                    .add_message(session_id, MessageRole::System, error_msg.clone())
                    .map_err(SeaError::Session)?;

                Err(SeaError::Router(e))
            }
        }
    }

    /// 获取消息历史
    pub async fn get_message_history(
        &self,
        session_id: SessionId,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<session_manager::MessageRecord>> {
        let messages = self
            .session_manager
            .get_messages(session_id, limit, offset)
            .map_err(SeaError::Session)?;
        Ok(messages)
    }

    /// 列出可用 Server 类型
    pub fn available_server_types(&self) -> Vec<String> {
        self.server_registry.available_types()
    }

    /// 初始化系统（用于 `run` 命令）
    pub async fn init(&mut self) -> Result<SessionId> {
        // 创建默认 Session
        let session_id = self.create_session().await?;

        // 注册默认 Servers
        let default_servers = vec![
            ServerType::LLMGateway,
            ServerType::Echo,
            ServerType::Calculator,
            ServerType::Time,
        ];

        for server_type in default_servers {
            if let Ok(id) = self
                .register_server(session_id, server_type.clone(), None)
                .await
            {
                if let Err(e) = self.start_server(&id).await {
                    tracing::warn!("Failed to start default server {:?}: {}", server_type, e);
                }
            }
        }

        tracing::info!("SEA Agent initialized with session: {}", session_id);
        Ok(session_id)
    }

    /// 优雅关闭
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down SEA Agent...");

        // 停止所有运行中的 Server
        let server_ids: Vec<String> = self
            .server_runners
            .iter()
            .filter(|(_, h)| h.running)
            .map(|(id, _)| id.clone())
            .collect();

        for id in server_ids {
            if let Err(e) = self.stop_server(&id).await {
                tracing::warn!("Error stopping server '{}': {}", id, e);
            }
        }

        tracing::info!("SEA Agent shutdown complete");
        Ok(())
    }

    /// 获取配置引用
    pub fn config(&self) -> &SeaConfig {
        &self.config
    }

    /// 获取 Session Manager 引用
    pub fn session_manager(&self) -> &SessionManager {
        &self.session_manager
    }
}
