use crate::models::{MessageRecord, MessageRole, Session, SessionId, SessionSummary};
use crate::store::{JsonSessionStore, SessionStore, StoreError};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Session Manager 错误
#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("Store error: {0}")]
    Store(#[from] StoreError),

    #[error("Session not found: {0}")]
    NotFound(SessionId),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Session Manager
pub struct SessionManager {
    store: JsonSessionStore,
    base_path: PathBuf,
}

impl SessionManager {
    /// 创建新的 Session Manager
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            store: JsonSessionStore::new(),
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// 获取 Session 文件路径
    fn session_path(&self, session_id: SessionId) -> PathBuf {
        self.base_path.join(format!("{}.json", session_id))
    }

    /// 创建新 Session
    pub fn create_session(&self) -> Result<Session, ManagerError> {
        self.create_session_with_name(None)
    }

    /// 创建带名称的 Session
    pub fn create_session_with_name(&self, name: Option<String>) -> Result<Session, ManagerError> {
        let session = Session::with_name(name);
        let path = self.session_path(session.session_id);
        self.store.save(&path, &session)?;
        Ok(session)
    }

    /// 加载 Session
    pub fn load_session(&self, session_id: SessionId) -> Result<Session, ManagerError> {
        let path = self.session_path(session_id);
        self.store.load(&path).map_err(ManagerError::from)
    }

    /// 保存 Session
    pub fn save_session(&self, session: &Session) -> Result<(), ManagerError> {
        let path = self.session_path(session.session_id);
        self.store.save(&path, session)?;
        Ok(())
    }

    /// 列出所有 Session 摘要
    pub fn list_sessions(&self) -> Result<Vec<SessionSummary>, ManagerError> {
        let mut summaries = Vec::new();

        if !self.base_path.exists() {
            return Ok(summaries);
        }

        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json") {
                if let Ok(session) = self.store.load(&path) {
                    summaries.push(SessionSummary::from(&session));
                }
            }
        }

        // 按创建时间倒序排列
        summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(summaries)
    }

    /// 删除 Session
    pub fn delete_session(&self, session_id: SessionId) -> Result<(), ManagerError> {
        let path = self.session_path(session_id);
        self.store.delete(&path)?;
        Ok(())
    }

    /// 终止 Session（标记为 Terminated）
    pub fn terminate_session(&self, session_id: SessionId) -> Result<(), ManagerError> {
        let mut session = self.load_session(session_id)?;
        session.state = crate::models::SessionState::Terminated;
        session.touch();
        self.save_session(&session)
    }

    /// 添加消息到 Session
    pub fn add_message(
        &self,
        session_id: SessionId,
        role: MessageRole,
        content: String,
    ) -> Result<MessageRecord, ManagerError> {
        let mut session = self.load_session(session_id)?;

        let message = MessageRecord {
            id: uuid::Uuid::new_v4(),
            role,
            content,
            timestamp: chrono::Utc::now(),
            metadata: None,
        };

        session.message_history.push(message.clone());
        session.touch();
        self.save_session(&session)?;

        Ok(message)
    }

    /// 获取消息历史
    pub fn get_messages(
        &self,
        session_id: SessionId,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<MessageRecord>, ManagerError> {
        let session = self.load_session(session_id)?;

        let offset = offset.unwrap_or(0);
        let messages: Vec<MessageRecord> = session
            .message_history
            .into_iter()
            .skip(offset)
            .take(limit.unwrap_or(usize::MAX))
            .collect();

        Ok(messages)
    }

    /// 清空消息历史
    pub fn clear_messages(&self, session_id: SessionId) -> Result<(), ManagerError> {
        let mut session = self.load_session(session_id)?;
        session.message_history.clear();
        session.touch();
        self.save_session(&session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_session() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session = manager.create_session().unwrap();
        assert!(session.session_id.to_string().len() > 0);
        assert!(temp_dir
            .path()
            .join(format!("{}.json", session.session_id))
            .exists());
    }

    #[test]
    fn test_load_session() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let created = manager.create_session().unwrap();
        let loaded = manager.load_session(created.session_id).unwrap();

        assert_eq!(created.session_id, loaded.session_id);
    }

    #[test]
    fn test_list_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        manager.create_session().unwrap();
        manager.create_session().unwrap();
        manager.create_session().unwrap();

        let summaries = manager.list_sessions().unwrap();
        assert_eq!(summaries.len(), 3);
    }

    #[test]
    fn test_delete_session() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session = manager.create_session().unwrap();
        assert!(manager.load_session(session.session_id).is_ok());

        manager.delete_session(session.session_id).unwrap();
        assert!(manager.load_session(session.session_id).is_err());
    }

    #[test]
    fn test_terminate_session() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session = manager.create_session().unwrap();
        manager.terminate_session(session.session_id).unwrap();

        let loaded = manager.load_session(session.session_id).unwrap();
        assert_eq!(loaded.state, crate::models::SessionState::Terminated);
    }

    #[test]
    fn test_add_message() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session = manager.create_session().unwrap();
        let message = manager
            .add_message(session.session_id, MessageRole::User, "hello".to_string())
            .unwrap();

        assert_eq!(message.content, "hello");

        let messages = manager
            .get_messages(session.session_id, None, None)
            .unwrap();
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_get_messages_with_pagination() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session = manager.create_session().unwrap();

        for i in 0..10 {
            manager
                .add_message(
                    session.session_id,
                    MessageRole::User,
                    format!("message {}", i),
                )
                .unwrap();
        }

        let page1 = manager
            .get_messages(session.session_id, Some(5), Some(0))
            .unwrap();
        let page2 = manager
            .get_messages(session.session_id, Some(5), Some(5))
            .unwrap();

        assert_eq!(page1.len(), 5);
        assert_eq!(page2.len(), 5);
    }

    #[test]
    fn test_clear_messages() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SessionManager::new(temp_dir.path());

        let session = manager.create_session().unwrap();
        manager
            .add_message(session.session_id, MessageRole::User, "test".to_string())
            .unwrap();

        manager.clear_messages(session.session_id).unwrap();

        let messages = manager
            .get_messages(session.session_id, None, None)
            .unwrap();
        assert!(messages.is_empty());
    }
}
