use crate::models::ServerId;

/// 路由表：capability → server_id 映射
#[derive(Debug, Clone, Default)]
pub struct RoutingTable {
    entries: std::collections::HashMap<String, ServerId>,
}

impl RoutingTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加路由
    pub fn add_route(&mut self, capability: &str, server_id: ServerId) {
        self.entries.insert(capability.to_string(), server_id);
    }

    /// 移除路由
    pub fn remove_route(&mut self, capability: &str) -> Option<ServerId> {
        self.entries.remove(capability)
    }

    /// 查找路由
    pub fn lookup(&self, capability: &str) -> Option<&ServerId> {
        self.entries.get(capability)
    }

    /// 移除与指定 Server 关联的所有路由
    pub fn remove_routes_for_server(&mut self, server_id: &str) -> Vec<String> {
        let removed: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, sid)| sid.as_str() == server_id)
            .map(|(cap, _)| cap.clone())
            .collect();

        for cap in &removed {
            self.entries.remove(cap);
        }

        removed
    }

    /// 获取所有路由条目
    pub fn entries(&self) -> &std::collections::HashMap<String, ServerId> {
        &self.entries
    }

    /// 路由数量
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_lookup_route() {
        let mut table = RoutingTable::new();
        table.add_route("capability:code_review", "server-a".to_string());

        assert_eq!(
            table.lookup("capability:code_review"),
            Some(&"server-a".to_string())
        );
        assert_eq!(table.lookup("capability:unknown"), None);
    }

    #[test]
    fn test_remove_route() {
        let mut table = RoutingTable::new();
        table.add_route("cap:1", "server-a".to_string());

        let removed = table.remove_route("cap:1");
        assert_eq!(removed, Some("server-a".to_string()));
        assert!(table.lookup("cap:1").is_none());
    }

    #[test]
    fn test_remove_routes_for_server() {
        let mut table = RoutingTable::new();
        table.add_route("cap:1", "server-a".to_string());
        table.add_route("cap:2", "server-a".to_string());
        table.add_route("cap:3", "server-b".to_string());

        let removed = table.remove_routes_for_server("server-a");

        assert_eq!(removed.len(), 2);
        assert!(table.lookup("cap:1").is_none());
        assert!(table.lookup("cap:2").is_none());
        assert_eq!(table.lookup("cap:3"), Some(&"server-b".to_string()));
    }

    #[test]
    fn test_empty_table() {
        let table = RoutingTable::new();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }
}
