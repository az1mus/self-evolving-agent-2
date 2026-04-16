use crate::manager::{ManagerError, SessionManager};
use crate::models::{ServerId, ServerInfo, ServerStatus, SessionId};
use crate::routing_table::RoutingTable;
use chrono::Utc;
use std::collections::HashMap;
use thiserror::Error;

/// 生命周期错误
#[derive(Debug, Error)]
pub enum LifecycleError {
    #[error("Server not found: {0}")]
    NotFound(ServerId),

    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),

    #[error("Manager error: {0}")]
    Manager(#[from] ManagerError),
}

/// Server 生命周期管理器
pub struct ServerLifecycle<'a> {
    session_manager: &'a SessionManager,
}

impl<'a> ServerLifecycle<'a> {
    pub fn new(session_manager: &'a SessionManager) -> Self {
        Self { session_manager }
    }

    /// 注册 Server
    pub fn register_server(
        &self,
        session_id: SessionId,
        server_id: ServerId,
        tools: Vec<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<(), LifecycleError> {
        self.register_server_with_name(session_id, server_id, None, tools, metadata)
    }

    /// 注册带名称的 Server
    pub fn register_server_with_name(
        &self,
        session_id: SessionId,
        server_id: ServerId,
        name: Option<String>,
        tools: Vec<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<(), LifecycleError> {
        let mut session = self.session_manager.load_session(session_id)?;

        if session.servers.contains_key(&server_id) {
            return Err(LifecycleError::InvalidTransition(format!(
                "Server {} already exists",
                server_id
            )));
        }

        let server_name = name.unwrap_or_else(|| format!("server-{}", &server_id[..8.min(server_id.len())]));
        let server_info = ServerInfo {
            id: server_id.clone(),
            name: server_name,
            status: ServerStatus::Pending,
            tools,
            metadata,
            draining_since: None,
        };

        session.servers.insert(server_id, server_info);
        session.touch();
        self.session_manager.save_session(&session)?;

        Ok(())
    }

    /// 注销 Server
    pub fn deregister_server(
        &self,
        session_id: SessionId,
        server_id: &str,
    ) -> Result<(), LifecycleError> {
        let mut session = self.session_manager.load_session(session_id)?;

        if session.servers.remove(server_id).is_none() {
            return Err(LifecycleError::NotFound(server_id.to_string()));
        }

        // 清理路由表
        let mut routing_table = RoutingTable::new();
        for (cap, sid) in session.routing_table.iter() {
            if sid != server_id {
                routing_table.add_route(cap, sid.clone());
            }
        }
        session.routing_table = routing_table.entries().clone();

        session.touch();
        self.session_manager.save_session(&session)?;

        Ok(())
    }

    /// 激活 Server (Pending → Active)
    pub fn activate_server(
        &self,
        session_id: SessionId,
        server_id: &str,
    ) -> Result<(), LifecycleError> {
        let mut session = self.session_manager.load_session(session_id)?;

        let server = session
            .servers
            .get_mut(server_id)
            .ok_or_else(|| LifecycleError::NotFound(server_id.to_string()))?;

        if server.status != ServerStatus::Pending {
            return Err(LifecycleError::InvalidTransition(format!(
                "Cannot activate server in {:?} state",
                server.status
            )));
        }

        server.status = ServerStatus::Active;
        session.touch();
        self.session_manager.save_session(&session)?;

        Ok(())
    }

    /// 开始排空 Server (Active → Draining)
    pub fn drain_server(
        &self,
        session_id: SessionId,
        server_id: &str,
    ) -> Result<(), LifecycleError> {
        let mut session = self.session_manager.load_session(session_id)?;

        let server = session
            .servers
            .get_mut(server_id)
            .ok_or_else(|| LifecycleError::NotFound(server_id.to_string()))?;

        if server.status != ServerStatus::Active {
            return Err(LifecycleError::InvalidTransition(format!(
                "Cannot drain server in {:?} state",
                server.status
            )));
        }

        server.status = ServerStatus::Draining;
        server.draining_since = Some(Utc::now());
        session.touch();
        self.session_manager.save_session(&session)?;

        Ok(())
    }

    /// 移除 Server (Draining → Removed)
    pub fn remove_server(
        &self,
        session_id: SessionId,
        server_id: &str,
    ) -> Result<(), LifecycleError> {
        let mut session = self.session_manager.load_session(session_id)?;

        let server = session
            .servers
            .get_mut(server_id)
            .ok_or_else(|| LifecycleError::NotFound(server_id.to_string()))?;

        if server.status != ServerStatus::Draining {
            return Err(LifecycleError::InvalidTransition(format!(
                "Cannot remove server in {:?} state",
                server.status
            )));
        }

        server.status = ServerStatus::Removed;
        session.touch();
        self.session_manager.save_session(&session)?;

        Ok(())
    }

    /// 检查并清理超时的 Draining Server
    pub fn check_draining_timeouts(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<ServerId>, LifecycleError> {
        let mut session = self.session_manager.load_session(session_id)?;
        let timeout_seconds = session.config.drain_timeout;
        let now = Utc::now();

        let mut timed_out = Vec::new();

        for (server_id, server) in session.servers.iter_mut() {
            if server.status == ServerStatus::Draining {
                if let Some(draining_since) = server.draining_since {
                    let elapsed = (now - draining_since).num_seconds();
                    if elapsed > timeout_seconds as i64 {
                        server.status = ServerStatus::Removed;
                        timed_out.push(server_id.clone());
                    }
                }
            }
        }

        if !timed_out.is_empty() {
            session.touch();
            self.session_manager.save_session(&session)?;
        }

        Ok(timed_out)
    }

    /// 更新 Server 状态
    pub fn update_server_status(
        &self,
        session_id: SessionId,
        server_id: &str,
        new_status: ServerStatus,
    ) -> Result<(), LifecycleError> {
        match new_status {
            ServerStatus::Active => self.activate_server(session_id, server_id),
            ServerStatus::Draining => self.drain_server(session_id, server_id),
            ServerStatus::Removed => self.remove_server(session_id, server_id),
            ServerStatus::Pending => Err(LifecycleError::InvalidTransition(
                "Cannot set status to Pending".to_string(),
            )),
        }
    }

    /// 添加路由
    pub fn add_route(
        &self,
        session_id: SessionId,
        capability: &str,
        server_id: ServerId,
    ) -> Result<(), LifecycleError> {
        let mut session = self.session_manager.load_session(session_id)?;

        if !session.servers.contains_key(&server_id) {
            return Err(LifecycleError::NotFound(server_id));
        }

        session
            .routing_table
            .insert(capability.to_string(), server_id);
        session.touch();
        self.session_manager.save_session(&session)?;

        Ok(())
    }

    /// 移除路由
    pub fn remove_route(
        &self,
        session_id: SessionId,
        capability: &str,
    ) -> Result<Option<ServerId>, LifecycleError> {
        let mut session = self.session_manager.load_session(session_id)?;
        let removed = session.routing_table.remove(capability);

        if removed.is_some() {
            session.touch();
            self.session_manager.save_session(&session)?;
        }

        Ok(removed)
    }

    /// 查找路由
    pub fn lookup_route(
        &self,
        session_id: SessionId,
        capability: &str,
    ) -> Result<Option<ServerId>, LifecycleError> {
        let session = self.session_manager.load_session(session_id)?;
        Ok(session.routing_table.get(capability).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, SessionManager, SessionId) {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path());
        let session = manager.create_session().unwrap();
        (temp_dir, manager, session.session_id)
    }

    #[test]
    fn test_register_server() {
        let (_temp_dir, manager, session_id) = setup();
        let lifecycle = ServerLifecycle::new(&manager);

        lifecycle
            .register_server(
                session_id,
                "server-a".to_string(),
                vec!["tool1".to_string()],
                HashMap::new(),
            )
            .unwrap();

        let session = manager.load_session(session_id).unwrap();
        assert!(session.servers.contains_key("server-a"));
        assert_eq!(session.servers["server-a"].status, ServerStatus::Pending);
    }

    #[test]
    fn test_activate_server() {
        let (_temp_dir, manager, session_id) = setup();
        let lifecycle = ServerLifecycle::new(&manager);

        lifecycle
            .register_server(session_id, "s1".to_string(), vec![], HashMap::new())
            .unwrap();

        lifecycle.activate_server(session_id, "s1").unwrap();

        let session = manager.load_session(session_id).unwrap();
        assert_eq!(session.servers["s1"].status, ServerStatus::Active);
    }

    #[test]
    fn test_drain_server() {
        let (_temp_dir, manager, session_id) = setup();
        let lifecycle = ServerLifecycle::new(&manager);

        lifecycle
            .register_server(session_id, "s1".to_string(), vec![], HashMap::new())
            .unwrap();
        lifecycle.activate_server(session_id, "s1").unwrap();
        lifecycle.drain_server(session_id, "s1").unwrap();

        let session = manager.load_session(session_id).unwrap();
        assert_eq!(session.servers["s1"].status, ServerStatus::Draining);
        assert!(session.servers["s1"].draining_since.is_some());
    }

    #[test]
    fn test_remove_server() {
        let (_temp_dir, manager, session_id) = setup();
        let lifecycle = ServerLifecycle::new(&manager);

        lifecycle
            .register_server(session_id, "s1".to_string(), vec![], HashMap::new())
            .unwrap();
        lifecycle.activate_server(session_id, "s1").unwrap();
        lifecycle.drain_server(session_id, "s1").unwrap();
        lifecycle.remove_server(session_id, "s1").unwrap();

        let session = manager.load_session(session_id).unwrap();
        assert_eq!(session.servers["s1"].status, ServerStatus::Removed);
    }

    #[test]
    fn test_invalid_state_transitions() {
        let (_temp_dir, manager, session_id) = setup();
        let lifecycle = ServerLifecycle::new(&manager);

        lifecycle
            .register_server(session_id, "s1".to_string(), vec![], HashMap::new())
            .unwrap();

        // Cannot drain Pending server
        assert!(lifecycle.drain_server(session_id, "s1").is_err());

        // Cannot remove Pending server
        assert!(lifecycle.remove_server(session_id, "s1").is_err());
    }

    #[test]
    fn test_deregister_server() {
        let (_temp_dir, manager, session_id) = setup();
        let lifecycle = ServerLifecycle::new(&manager);

        lifecycle
            .register_server(session_id, "s1".to_string(), vec![], HashMap::new())
            .unwrap();

        lifecycle.deregister_server(session_id, "s1").unwrap();

        let session = manager.load_session(session_id).unwrap();
        assert!(!session.servers.contains_key("s1"));
    }

    #[test]
    fn test_route_management() {
        let (_temp_dir, manager, session_id) = setup();
        let lifecycle = ServerLifecycle::new(&manager);

        lifecycle
            .register_server(session_id, "s1".to_string(), vec![], HashMap::new())
            .unwrap();

        lifecycle
            .add_route(session_id, "cap:code_review", "s1".to_string())
            .unwrap();

        let found = lifecycle
            .lookup_route(session_id, "cap:code_review")
            .unwrap();
        assert_eq!(found, Some("s1".to_string()));

        lifecycle
            .remove_route(session_id, "cap:code_review")
            .unwrap();
        let not_found = lifecycle
            .lookup_route(session_id, "cap:code_review")
            .unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_route_cleanup_on_deregister() {
        let (_temp_dir, manager, session_id) = setup();
        let lifecycle = ServerLifecycle::new(&manager);

        lifecycle
            .register_server(session_id, "s1".to_string(), vec![], HashMap::new())
            .unwrap();

        lifecycle
            .add_route(session_id, "cap:1", "s1".to_string())
            .unwrap();
        lifecycle
            .add_route(session_id, "cap:2", "s1".to_string())
            .unwrap();

        lifecycle.deregister_server(session_id, "s1").unwrap();

        let session = manager.load_session(session_id).unwrap();
        assert!(!session.routing_table.contains_key("cap:1"));
        assert!(!session.routing_table.contains_key("cap:2"));
    }

    #[test]
    fn test_draining_timeout() {
        let (_temp_dir, manager, session_id) = setup();
        let lifecycle = ServerLifecycle::new(&manager);

        // Create session with short timeout
        let mut session = manager.load_session(session_id).unwrap();
        session.config.drain_timeout = 1; // 1 second
        manager.save_session(&session).unwrap();

        lifecycle
            .register_server(session_id, "s1".to_string(), vec![], HashMap::new())
            .unwrap();
        lifecycle.activate_server(session_id, "s1").unwrap();
        lifecycle.drain_server(session_id, "s1").unwrap();

        // Wait for timeout
        std::thread::sleep(std::time::Duration::from_secs(2));

        let timed_out = lifecycle.check_draining_timeouts(session_id).unwrap();
        assert_eq!(timed_out, vec!["s1"]);

        let session = manager.load_session(session_id).unwrap();
        assert_eq!(session.servers["s1"].status, ServerStatus::Removed);
    }
}
