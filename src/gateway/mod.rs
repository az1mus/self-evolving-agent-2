//! Gateway 模块
//!
//! 作为 CLI 和 Agent 之间的消息路由中介
//! Gateway 管理：
//! - AgentDefinition: Agent 定义/模板，独立存储，可复用
//! - Session: 运行时容器，包含消息历史
//! - AgentInstance: Agent 实例，在 Session 中运行
//! - GlobalMemory: 全局记忆，跨 Session 共享的信息
//!
//! 多层提示词变量更新位置：
//! - Session: 对话记忆（session_overview, session_summary, context）
//! - Gateway: 全局记忆（global_memory）

mod config;
mod event;
mod message;

pub use config::GatewayConfig;
pub use event::{GatewayEvent, EventListener};
pub use message::format_context;

use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

use crate::agent::{
    AgentDefinition, AgentDefinitionId, AgentDefinitionInfo, AgentInput, AgentInstance,
    AgentInstanceInfo, AgentInstanceId, AgentOutput,
};
use crate::config::ConfigManager;
use crate::llm::LlmClientConfig;
use crate::logger::Logger;
use crate::prompt::PromptContext;
use crate::session::{Session, SessionId, SessionInfo, SessionManager};

/// Agent 定义更新参数
#[derive(Debug, Clone, Default)]
pub struct AgentDefinitionUpdates {
    pub name: Option<String>,
    pub system_prompt: Option<String>,
    pub llm_config: Option<LlmClientConfig>,
}

/// Gateway - 消息路由中介
pub struct Gateway {
    /// Agent 定义集合
    agent_definitions: HashMap<AgentDefinitionId, AgentDefinition>,
    /// Agent 定义存储目录
    agent_definition_dir: PathBuf,
    /// Session 集合
    sessions: HashMap<SessionId, Session>,
    /// 当前 Session ID
    current_session_id: Option<SessionId>,
    /// Agent 实例集合
    agent_instances: HashMap<AgentInstanceId, AgentInstance>,
    /// 当前 Agent 实例 ID
    current_agent_instance_id: Option<AgentInstanceId>,
    /// 配置管理器
    config_manager: ConfigManager,
    /// Gateway 配置
    gateway_config: GatewayConfig,
    /// 日志记录器
    logger: Arc<Logger>,
    /// 事件监听器
    event_listeners: Vec<EventListener>,
    /// Session 管理器
    session_manager: SessionManager,
    /// 全局记忆
    /// 跨 Session 共享的信息，用于多层提示词
    global_memory: Option<String>,
}

impl Gateway {
    /// 创建新的 Gateway
    pub fn new(config_manager: ConfigManager, logger: Logger) -> Result<Self> {
        let logger = Arc::new(logger);

        // 初始化目录
        let base_dir = config_manager.config_path().parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::temp_dir());

        let agent_definition_dir = base_dir.join("agent_definitions");
        let session_dir = base_dir.join("sessions");

        fs::create_dir_all(&agent_definition_dir)?;

        let max_history = config_manager.config().session.max_history;
        let session_manager = SessionManager::new(session_dir, max_history)?;

        let mut gateway = Self {
            agent_definitions: HashMap::new(),
            agent_definition_dir: agent_definition_dir.clone(),
            sessions: HashMap::new(),
            current_session_id: None,
            agent_instances: HashMap::new(),
            current_agent_instance_id: None,
            config_manager,
            gateway_config: GatewayConfig::default(),
            logger,
            event_listeners: Vec::new(),
            session_manager,
            global_memory: None,
        };

        // 加载已存在的 Agent 定义
        gateway.load_agent_definitions()?;

        // 从可执行文件同级目录的 sources/agents 导入 Agent 定义
        gateway.import_agents_from_sources()?;

        Ok(gateway)
    }

    /// 从可执行文件同级目录的 resources/agents 导入 Agent 定义
    fn import_agents_from_sources(&mut self) -> Result<()> {
        // 获取可执行文件所在目录
        let exe_dir = std::env::current_exe()
            .map(|p| p.parent().map(|p| p.to_path_buf()))
            .ok()
            .flatten();

        if let Some(exe_dir) = exe_dir {
            let resources_dir = exe_dir.join("resources").join("agents");

            if resources_dir.exists() {
                // 遍历 resources/agents 目录下的所有 json 文件
                for entry in fs::read_dir(&resources_dir)? {
                    let entry = entry?;
                    let path = entry.path();

                    // 只处理 json 文件，跳过 README.md 等
                    if path.extension().map_or(false, |ext| ext == "json") {
                        if let Ok(json) = fs::read_to_string(&path) {
                            // 跳过模板文件（包含 _comment 字段的）
                            if json.contains("\"_comment\"") {
                                continue;
                            }

                            if let Ok(definition) = serde_json::from_str::<AgentDefinition>(&json) {
                                // 检查是否已存在相同 ID 的定义
                                if !self.agent_definitions.contains_key(&definition.id) {
                                    // 保存到 agent_definitions 目录
                                    self.save_agent_definition(&definition)?;
                                    // 添加到内存
                                    self.agent_definitions.insert(definition.id.clone(), definition);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 设置 Gateway 配置
    pub fn with_gateway_config(mut self, config: GatewayConfig) -> Self {
        self.gateway_config = config;
        self
    }

    /// 添加事件监听器
    pub fn add_event_listener(&mut self, listener: EventListener) {
        self.event_listeners.push(listener);
    }

    /// 触发事件
    fn emit_event(&self, event: GatewayEvent) {
        for listener in &self.event_listeners {
            listener(&event);
        }
    }

    /// 加载 Agent 定义
    fn load_agent_definitions(&mut self) -> Result<()> {
        if !self.agent_definition_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.agent_definition_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(definition) = serde_json::from_str::<AgentDefinition>(&json) {
                        self.agent_definitions.insert(definition.id.clone(), definition);
                    }
                }
            }
        }

        Ok(())
    }

    /// 保存 Agent 定义
    fn save_agent_definition(&self, definition: &AgentDefinition) -> Result<()> {
        let file_path = self.agent_definition_dir.join(format!("{}.json", definition.id));
        let json = serde_json::to_string_pretty(definition)?;
        fs::write(&file_path, json)?;
        Ok(())
    }

    /// 删除 Agent 定义文件
    fn delete_agent_definition_file(&self, id: &str) -> Result<()> {
        let file_path = self.agent_definition_dir.join(format!("{}.json", id));
        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }
        Ok(())
    }

    // ==================== Agent Definition 管理 ====================

    /// 创建 Agent 定义
    pub fn create_agent_definition(
        &mut self,
        name: String,
        system_prompt: Option<String>,
    ) -> Result<AgentDefinitionId> {
        if self.agent_definitions.len() >= self.gateway_config.max_agent_definitions {
            anyhow::bail!("Maximum number of agent definitions reached");
        }

        // 创建时不设置 llm_config，实例化时使用程序默认配置
        let definition = AgentDefinition::new(name.clone(), system_prompt, None);
        let id = definition.id.clone();

        self.save_agent_definition(&definition)?;
        self.agent_definitions.insert(id.clone(), definition);

        self.emit_event(GatewayEvent::AgentDefinitionCreated { id: id.clone(), name });

        Ok(id)
    }

    /// 获取 Agent 定义
    pub fn get_agent_definition(&self, id: &str) -> Option<&AgentDefinition> {
        self.agent_definitions.get(id)
    }

    /// 获取 Agent 定义（可变）
    pub fn get_agent_definition_mut(&mut self, id: &str) -> Option<&mut AgentDefinition> {
        self.agent_definitions.get_mut(id)
    }

    /// 列出所有 Agent 定义
    pub fn list_agent_definitions(&self) -> Vec<AgentDefinitionInfo> {
        self.agent_definitions
            .values()
            .map(|d| AgentDefinitionInfo {
                id: d.id.clone(),
                name: d.name.clone(),
                description: d.description.clone(),
                has_system_prompt: d.system_prompt.is_some(),
            })
            .collect()
    }

    /// 删除 Agent 定义
    pub fn delete_agent_definition(&mut self, id: &str) -> Result<()> {
        if self.agent_definitions.remove(id).is_some() {
            self.delete_agent_definition_file(id)?;
            self.emit_event(GatewayEvent::AgentDefinitionDeleted { id: id.to_string() });
        }
        Ok(())
    }

    /// 更新 Agent 定义
    pub fn update_agent_definition(&mut self, id: &str, updates: AgentDefinitionUpdates) -> Result<()> {
        {
            let definition = self.agent_definitions.get_mut(id)
                .ok_or_else(|| anyhow::anyhow!("Agent definition not found"))?;

            if let Some(name) = updates.name {
                definition.name = name;
            }
            if let Some(prompt) = updates.system_prompt {
                definition.system_prompt = Some(prompt);
            }
            if let Some(config) = updates.llm_config {
                definition.llm_config = Some(config);
            }

            definition.updated_at = chrono::Utc::now();
        }

        // 释放借用后再保存
        let definition = self.agent_definitions.get(id).unwrap();
        self.save_agent_definition(definition)?;

        Ok(())
    }

    /// Agent 定义数量
    pub fn agent_definition_count(&self) -> usize {
        self.agent_definitions.len()
    }

    // ==================== Session 管理 ====================

    /// 创建 Session
    pub fn create_session(&mut self, name: String) -> Result<SessionId> {
        if self.sessions.len() >= self.gateway_config.max_sessions {
            anyhow::bail!("Maximum number of sessions reached");
        }

        let session = Session::new(name.clone());
        let id = session.id.clone();

        self.sessions.insert(id.clone(), session);

        if self.current_session_id.is_none() {
            self.current_session_id = Some(id.clone());
        }

        self.emit_event(GatewayEvent::SessionCreated { id: id.clone(), name });

        Ok(id)
    }

    /// 切换 Session
    pub fn switch_session(&mut self, id: &str) -> Result<()> {
        if !self.sessions.contains_key(id) {
            anyhow::bail!("Session not found: {}", id);
        }

        self.current_session_id = Some(id.to_string());
        self.current_agent_instance_id = None;

        // 尝试设置该 session 的第一个 agent instance
        if let Some(session) = self.sessions.get(id) {
            if let Some(instance_id) = session.agent_instance_ids().iter().next() {
                self.current_agent_instance_id = Some(instance_id.clone());
            }
        }

        self.emit_event(GatewayEvent::SessionSwitched { id: id.to_string() });

        Ok(())
    }

    /// 删除 Session
    pub fn delete_session(&mut self, id: &str) -> Result<()> {
        if let Some(session) = self.sessions.remove(id) {
            // 删除该 session 下的所有 agent instance
            for instance_id in session.agent_instance_ids() {
                self.agent_instances.remove(instance_id);
            }

            self.session_manager.delete_session(id)?;

            if self.current_session_id.as_deref() == Some(id) {
                self.current_session_id = self.sessions.keys().next().cloned();
                self.current_agent_instance_id = None;

                if let Some(new_session_id) = &self.current_session_id {
                    if let Some(new_session) = self.sessions.get(new_session_id) {
                        if let Some(instance_id) = new_session.agent_instance_ids().iter().next() {
                            self.current_agent_instance_id = Some(instance_id.clone());
                        }
                    }
                }
            }

            self.emit_event(GatewayEvent::SessionDeleted { id: id.to_string() });
        }

        Ok(())
    }

    /// 获取当前 Session
    pub fn current_session(&self) -> Option<&Session> {
        self.current_session_id.as_ref().and_then(|id| self.sessions.get(id))
    }

    /// 获取当前 Session ID
    pub fn current_session_id(&self) -> Option<&str> {
        self.current_session_id.as_deref()
    }

    /// 列出所有 Session
    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions
            .values()
            .map(|s| SessionInfo {
                id: s.id.clone(),
                name: s.name.clone(),
                created_at: s.created_at,
                updated_at: s.updated_at,
                message_count: s.messages.len(),
                agent_count: s.agent_instance_ids.len(),
            })
            .collect()
    }

    /// Session 数量
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// 保存当前 Session
    pub fn save_current_session(&self) -> Result<()> {
        if let Some(session) = self.current_session() {
            self.session_manager.save_session(session)?;
        }
        Ok(())
    }

    // ==================== Agent Instance 管理 ====================

    /// 在当前 Session 中实例化 Agent
    pub fn instantiate_agent(&mut self, definition_id: &str) -> Result<AgentInstanceId> {
        let session_id = self.current_session_id.clone()
            .ok_or_else(|| anyhow::anyhow!("No active session"))?;

        let session = self.sessions.get(&session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        if session.agent_count() >= self.gateway_config.max_agents_per_session {
            anyhow::bail!("Maximum number of agents per session reached");
        }

        let definition = self.agent_definitions.get(definition_id)
            .ok_or_else(|| anyhow::anyhow!("Agent definition not found"))?;

        // 获取 LLM 配置：优先使用 Agent 定义的配置，否则使用程序默认配置
        let llm_config = definition.llm_config.clone().unwrap_or_else(|| {
            let config = self.config_manager.config();
            LlmClientConfig {
                api_base: config.llm.api_base.clone(),
                api_key: config.llm.api_key.clone(),
                model: config.llm.model.clone(),
                max_tokens: config.llm.max_tokens,
                temperature: config.llm.temperature,
            }
        });

        let instance_id = Uuid::new_v4().to_string();
        let instance = AgentInstance::new(
            instance_id.clone(),
            definition_id.to_string(),
            session_id.clone(),
            llm_config,
        )?.with_logger(self.logger.clone());

        self.agent_instances.insert(instance_id.clone(), instance);

        // 添加到 session
        let session = self.sessions.get_mut(&session_id).unwrap();
        session.add_agent_instance(instance_id.clone());

        // 如果是第一个 instance，设为当前
        if self.current_agent_instance_id.is_none() {
            self.current_agent_instance_id = Some(instance_id.clone());
        }

        self.emit_event(GatewayEvent::AgentInstanceCreated {
            id: instance_id.clone(),
            definition_id: definition_id.to_string(),
            session_id,
        });

        Ok(instance_id)
    }

    /// 切换当前 Agent 实例
    pub fn switch_agent_instance(&mut self, id: &str) -> Result<()> {
        if !self.agent_instances.contains_key(id) {
            anyhow::bail!("Agent instance not found: {}", id);
        }

        let instance = self.agent_instances.get(id).unwrap();
        if self.current_session_id.as_deref() != Some(instance.session_id()) {
            anyhow::bail!("Agent instance does not belong to current session");
        }

        self.current_agent_instance_id = Some(id.to_string());
        self.emit_event(GatewayEvent::AgentInstanceSwitched { id: id.to_string() });

        Ok(())
    }

    /// 删除 Agent 实例
    pub fn delete_agent_instance(&mut self, id: &str) -> Result<()> {
        if let Some(instance) = self.agent_instances.remove(id) {
            let session_id = instance.session_id().to_string();

            if let Some(session) = self.sessions.get_mut(&session_id) {
                session.remove_agent_instance(id);
            }

            self.emit_event(GatewayEvent::AgentInstanceDeleted { id: id.to_string() });

            if self.current_agent_instance_id.as_deref() == Some(id) {
                self.current_agent_instance_id = None;

                if let Some(session) = self.sessions.get(&session_id) {
                    if let Some(new_id) = session.agent_instance_ids().iter().next() {
                        self.current_agent_instance_id = Some(new_id.clone());
                    }
                }
            }
        }

        Ok(())
    }

    /// 获取当前 Agent 实例
    pub fn current_agent_instance(&self) -> Option<&AgentInstance> {
        self.current_agent_instance_id.as_ref().and_then(|id| self.agent_instances.get(id))
    }

    /// 获取当前 Agent 实例 ID
    pub fn current_agent_instance_id(&self) -> Option<&str> {
        self.current_agent_instance_id.as_deref()
    }

    /// 列出当前 Session 的 Agent 实例
    pub fn list_agent_instances(&self) -> Vec<AgentInstanceInfo> {
        if let Some(session_id) = &self.current_session_id {
            if let Some(session) = self.sessions.get(session_id) {
                return session.agent_instance_ids()
                    .iter()
                    .filter_map(|id| {
                        self.agent_instances.get(id).map(|instance| {
                            let definition = self.agent_definitions.get(instance.definition_id());
                            AgentInstanceInfo {
                                id: id.clone(),
                                definition_id: instance.definition_id().to_string(),
                                definition_name: definition.map(|d| d.name.clone()).unwrap_or_default(),
                                is_current: self.current_agent_instance_id.as_deref() == Some(id.as_str()),
                            }
                        })
                    })
                    .collect();
            }
        }
        Vec::new()
    }

    /// Agent 实例数量（当前 Session）
    pub fn agent_instance_count(&self) -> usize {
        self.current_session()
            .map(|s| s.agent_count())
            .unwrap_or(0)
    }

    /// 是否有 Agent 实例
    pub fn has_agent_instances(&self) -> bool {
        self.current_session()
            .map(|s| s.agent_count() > 0)
            .unwrap_or(false)
    }

    // ==================== 消息处理 ====================

    /// 发送消息（使用多层提示词）
    pub async fn send_message(&mut self, input: AgentInput) -> Result<AgentOutput> {
        let instance_id = self.current_agent_instance_id.clone()
            .ok_or_else(|| anyhow::anyhow!("No active agent instance"))?;

        let session_id = self.current_session_id.clone()
            .ok_or_else(|| anyhow::anyhow!("No active session"))?;

        let content_preview = input.content.chars().take(50).collect::<String>();

        // 获取 Agent 定义
        let definition_id = {
            let instance = self.agent_instances.get(&instance_id)
                .ok_or_else(|| anyhow::anyhow!("Agent instance not found"))?;
            instance.definition_id().to_string()
        };

        // 添加用户消息到 Session
        {
            let session = self.sessions.get_mut(&session_id)
                .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
            session.add_user_message(&input.content, Some(instance_id.clone()));
        }

        // 构建提示词上下文
        let prompt_context = {
            let session = self.sessions.get(&session_id)
                .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

            // 构建运行时变量
            let mut ctx = PromptContext::new();

            // 添加 session 相关变量
            if let Some(overview) = session.get_session_overview() {
                ctx.insert("session_overview", overview);
            }
            if let Some(summary) = session.get_session_summary() {
                ctx.insert("session_summary", summary);
            }
            if let Some(memory) = &self.global_memory {
                ctx.insert("global_memory", memory);
            }

            // 添加对话上下文
            let agent_messages = session.get_agent_context(&instance_id);
            if !agent_messages.is_empty() {
                ctx.insert("context", &format_context(&agent_messages));
            }

            ctx
        };

        // 构建消息列表
        let messages = {
            let session = self.sessions.get(&session_id)
                .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
            session.to_chat_messages()
        };

        // 处理消息
        let output = {
            let instance = self.agent_instances.get_mut(&instance_id)
                .ok_or_else(|| anyhow::anyhow!("Agent instance not found"))?;
            let definition = self.agent_definitions.get(&definition_id)
                .ok_or_else(|| anyhow::anyhow!("Agent definition not found"))?;
            instance.process(messages, &prompt_context, definition).await
        };

        // 添加助手回复到 Session
        {
            let session = self.sessions.get_mut(&session_id)
                .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
            session.add_assistant_message(&output.content, Some(instance_id.clone()));
        }

        // 触发事件
        self.emit_event(GatewayEvent::MessageSent {
            agent_instance_id: instance_id.clone(),
            content: content_preview,
        });

        self.emit_event(GatewayEvent::MessageReceived {
            agent_instance_id: instance_id.clone(),
            output: output.clone(),
        });

        Ok(output)
    }

    // ==================== 其他方法 ====================

    /// 清空当前 Session 历史
    pub fn clear_current_session_history(&mut self) -> Result<()> {
        let session_id = self.current_session_id.clone()
            .ok_or_else(|| anyhow::anyhow!("No active session"))?;

        let session = self.sessions.get_mut(&session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        session.clear_messages();
        Ok(())
    }

    /// 获取消息数量
    pub fn message_count(&self) -> usize {
        self.current_session()
            .map(|s| s.message_count())
            .unwrap_or(0)
    }

    /// 更新配置
    pub fn update_config(&mut self) -> Result<()> {
        let config = self.config_manager.config();
        let new_llm_config = LlmClientConfig {
            api_base: config.llm.api_base.clone(),
            api_key: config.llm.api_key.clone(),
            model: config.llm.model.clone(),
            max_tokens: config.llm.max_tokens,
            temperature: config.llm.temperature,
        };

        // 更新所有 Agent 实例的 LLM 配置
        for instance in self.agent_instances.values_mut() {
            instance.update_llm_config(new_llm_config.clone());
        }

        Ok(())
    }

    /// 获取配置管理器
    pub fn config_manager(&self) -> &ConfigManager {
        &self.config_manager
    }

    /// 获取可变配置管理器
    pub fn config_manager_mut(&mut self) -> &mut ConfigManager {
        &mut self.config_manager
    }

    /// 获取日志记录器
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    // ==================== 全局记忆管理 ====================

    /// 设置全局记忆
    pub fn set_global_memory(&mut self, memory: String) {
        self.global_memory = Some(memory);
    }

    /// 获取全局记忆
    pub fn get_global_memory(&self) -> Option<&String> {
        self.global_memory.as_ref()
    }

    /// 追加全局记忆
    pub fn append_global_memory(&mut self, content: &str) {
        match &mut self.global_memory {
            Some(memory) => {
                memory.push('\n');
                memory.push_str(content);
            }
            None => {
                self.global_memory = Some(content.to_string());
            }
        }
    }

    /// 清除全局记忆
    pub fn clear_global_memory(&mut self) {
        self.global_memory = None;
    }

    // ==================== Session 概要/总结管理 ====================

    /// 设置当前 Session 的概要
    pub fn set_session_overview(&mut self, overview: String) -> Result<()> {
        let session_id = self.current_session_id.clone()
            .ok_or_else(|| anyhow::anyhow!("No active session"))?;

        let session = self.sessions.get_mut(&session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        session.set_session_overview(overview);
        Ok(())
    }

    /// 设置当前 Session 的总结
    pub fn set_session_summary(&mut self, summary: String) -> Result<()> {
        let session_id = self.current_session_id.clone()
            .ok_or_else(|| anyhow::anyhow!("No active session"))?;

        let session = self.sessions.get_mut(&session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        session.set_session_summary(summary);
        Ok(())
    }

    /// 获取当前 Session 的概要
    pub fn get_session_overview(&self) -> Option<&String> {
        self.current_session().and_then(|s| s.get_session_overview())
    }

    /// 获取当前 Session 的总结
    pub fn get_session_summary(&self) -> Option<&String> {
        self.current_session().and_then(|s| s.get_session_summary())
    }

    /// 清除当前 Session 的总结
    pub fn clear_session_summary(&mut self) -> Result<()> {
        let session_id = self.current_session_id.clone()
            .ok_or_else(|| anyhow::anyhow!("No active session"))?;

        let session = self.sessions.get_mut(&session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        session.clear_session_summary();
        Ok(())
    }
}