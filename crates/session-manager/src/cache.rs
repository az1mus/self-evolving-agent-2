use crate::manager::{ManagerError, SessionManager};
use crate::models::SessionId;
use sha2::{Digest, Sha256};

/// Cache Manager
pub struct CacheManager<'a> {
    session_manager: &'a SessionManager,
}

impl<'a> CacheManager<'a> {
    pub fn new(session_manager: &'a SessionManager) -> Self {
        Self { session_manager }
    }

    /// 计算内容的 SHA-256 哈希
    pub fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// 获取输入缓存
    pub fn get_input_cache(
        &self,
        session_id: SessionId,
        hash: &str,
    ) -> Result<Option<serde_json::Value>, ManagerError> {
        let session = self.session_manager.load_session(session_id)?;
        Ok(session.cache.input_cache.get(hash).cloned())
    }

    /// 设置输入缓存
    pub fn set_input_cache(
        &self,
        session_id: SessionId,
        hash: &str,
        value: serde_json::Value,
    ) -> Result<(), ManagerError> {
        let mut session = self.session_manager.load_session(session_id)?;
        session.cache.input_cache.insert(hash.to_string(), value);
        session.touch();
        self.session_manager.save_session(&session)
    }

    /// 获取推理缓存
    pub fn get_inference_cache(
        &self,
        session_id: SessionId,
        key: &str,
    ) -> Result<Option<serde_json::Value>, ManagerError> {
        let session = self.session_manager.load_session(session_id)?;
        Ok(session.cache.inference_cache.get(key).cloned())
    }

    /// 设置推理缓存
    pub fn set_inference_cache(
        &self,
        session_id: SessionId,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), ManagerError> {
        let mut session = self.session_manager.load_session(session_id)?;
        session.cache.inference_cache.insert(key.to_string(), value);
        session.touch();
        self.session_manager.save_session(&session)
    }

    /// 使缓存失效（指定 key 或全部）
    pub fn invalidate_cache(
        &self,
        session_id: SessionId,
        key: Option<&str>,
    ) -> Result<(), ManagerError> {
        let mut session = self.session_manager.load_session(session_id)?;

        match key {
            Some(k) => {
                session.cache.input_cache.remove(k);
                session.cache.inference_cache.remove(k);
            }
            None => {
                session.cache.input_cache.clear();
                session.cache.inference_cache.clear();
            }
        }

        session.touch();
        self.session_manager.save_session(&session)
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
    fn test_hash_content() {
        let hash1 = CacheManager::hash_content("hello world");
        let hash2 = CacheManager::hash_content("hello world");
        let hash3 = CacheManager::hash_content("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert!(hash1.len() == 64); // SHA-256 hex string length
    }

    #[test]
    fn test_input_cache_crud() {
        let (_temp_dir, manager, session_id) = setup();
        let cache_mgr = CacheManager::new(&manager);

        let hash = CacheManager::hash_content("test input");
        let value = serde_json::json!({"result": "cached"});

        // Set
        cache_mgr
            .set_input_cache(session_id, &hash, value.clone())
            .unwrap();

        // Get
        let cached = cache_mgr.get_input_cache(session_id, &hash).unwrap();
        assert_eq!(cached, Some(value));

        // Miss
        let miss = cache_mgr
            .get_input_cache(session_id, "nonexistent")
            .unwrap();
        assert!(miss.is_none());
    }

    #[test]
    fn test_inference_cache_crud() {
        let (_temp_dir, manager, session_id) = setup();
        let cache_mgr = CacheManager::new(&manager);

        let key = "inference_key";
        let value = serde_json::json!({"reasoning": "result"});

        cache_mgr
            .set_inference_cache(session_id, key, value.clone())
            .unwrap();

        let cached = cache_mgr.get_inference_cache(session_id, key).unwrap();
        assert_eq!(cached, Some(value));
    }

    #[test]
    fn test_invalidate_specific_key() {
        let (_temp_dir, manager, session_id) = setup();
        let cache_mgr = CacheManager::new(&manager);

        let hash1 = "key1";
        let hash2 = "key2";

        cache_mgr
            .set_input_cache(session_id, hash1, serde_json::json!("v1"))
            .unwrap();
        cache_mgr
            .set_input_cache(session_id, hash2, serde_json::json!("v2"))
            .unwrap();

        cache_mgr.invalidate_cache(session_id, Some(hash1)).unwrap();

        assert!(cache_mgr
            .get_input_cache(session_id, hash1)
            .unwrap()
            .is_none());
        assert!(cache_mgr
            .get_input_cache(session_id, hash2)
            .unwrap()
            .is_some());
    }

    #[test]
    fn test_invalidate_all_cache() {
        let (_temp_dir, manager, session_id) = setup();
        let cache_mgr = CacheManager::new(&manager);

        cache_mgr
            .set_input_cache(session_id, "k1", serde_json::json!("v1"))
            .unwrap();
        cache_mgr
            .set_inference_cache(session_id, "k2", serde_json::json!("v2"))
            .unwrap();

        cache_mgr.invalidate_cache(session_id, None).unwrap();

        assert!(cache_mgr
            .get_input_cache(session_id, "k1")
            .unwrap()
            .is_none());
        assert!(cache_mgr
            .get_inference_cache(session_id, "k2")
            .unwrap()
            .is_none());
    }
}
