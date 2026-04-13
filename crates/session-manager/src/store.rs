use crate::models::{Session, SessionId};
use std::path::Path;
use thiserror::Error;

/// Session 存储错误
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Session not found: {0}")]
    NotFound(SessionId),

    #[error("Session file already exists: {0}")]
    AlreadyExists(SessionId),
}

/// Session 存储接口
pub trait SessionStore {
    /// 加载 Session
    fn load(&self, path: &Path) -> Result<Session, StoreError>;

    /// 保存 Session
    fn save(&self, path: &Path, session: &Session) -> Result<(), StoreError>;

    /// 删除 Session 文件
    fn delete(&self, path: &Path) -> Result<(), StoreError>;

    /// 检查 Session 文件是否存在
    fn exists(&self, path: &Path) -> bool;
}

/// 基于 JSON 文件的 Session 存储
pub struct JsonSessionStore;

impl JsonSessionStore {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonSessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionStore for JsonSessionStore {
    fn load(&self, path: &Path) -> Result<Session, StoreError> {
        if !path.exists() {
            return Err(StoreError::NotFound(
                Session::new().session_id, // 返回一个占位符 ID
            ));
        }

        let content = std::fs::read_to_string(path)?;
        let session: Session = serde_json::from_str(&content)?;
        Ok(session)
    }

    fn save(&self, path: &Path, session: &Session) -> Result<(), StoreError> {
        // 确保父目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(session)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn delete(&self, path: &Path) -> Result<(), StoreError> {
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_session() {
        let temp_dir = TempDir::new().unwrap();
        let session_path = temp_dir.path().join("test_session.json");

        let store = JsonSessionStore::new();
        let mut session = Session::new();

        // 保存
        store.save(&session_path, &session).unwrap();
        assert!(session_path.exists());

        // 加载
        let loaded = store.load(&session_path).unwrap();
        assert_eq!(session.session_id, loaded.session_id);
    }

    #[test]
    fn test_load_nonexistent_session() {
        let temp_dir = TempDir::new().unwrap();
        let session_path = temp_dir.path().join("nonexistent.json");

        let store = JsonSessionStore::new();
        let result = store.load(&session_path);

        assert!(result.is_err());
    }

    #[test]
    fn test_delete_session() {
        let temp_dir = TempDir::new().unwrap();
        let session_path = temp_dir.path().join("to_delete.json");

        let store = JsonSessionStore::new();
        let session = Session::new();

        // 创建文件
        store.save(&session_path, &session).unwrap();
        assert!(session_path.exists());

        // 删除文件
        store.delete(&session_path).unwrap();
        assert!(!session_path.exists());
    }

    #[test]
    fn test_save_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let session_path = temp_dir.path().join("nested/dirs/session.json");

        let store = JsonSessionStore::new();
        let session = Session::new();

        // 保存到不存在的嵌套目录
        store.save(&session_path, &session).unwrap();
        assert!(session_path.exists());
    }

    #[test]
    fn test_session_file_pretty_print() {
        let temp_dir = TempDir::new().unwrap();
        let session_path = temp_dir.path().join("pretty.json");

        let store = JsonSessionStore::new();
        let session = Session::new();

        store.save(&session_path, &session).unwrap();

        // 读取文件内容检查格式
        let content = std::fs::read_to_string(&session_path).unwrap();
        assert!(content.contains("session_id"));
        assert!(content.contains("created_at"));
    }
}
