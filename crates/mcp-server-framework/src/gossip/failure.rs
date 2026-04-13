use session_manager::ServerId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 失效状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureState {
    /// 存活
    Alive,
    /// 可疑
    Suspected,
    /// 确认失效
    ConfirmedDead,
}

/// 失效检测配置
#[derive(Debug, Clone)]
pub struct FailureDetectorConfig {
    /// 可疑阈值 (心跳超时次数)
    pub suspect_threshold: u32,
    /// 确认失效阈值
    pub confirm_threshold: u32,
}

impl Default for FailureDetectorConfig {
    fn default() -> Self {
        Self {
            suspect_threshold: 2,
            confirm_threshold: 4,
        }
    }
}

/// 节点失效跟踪
#[derive(Debug, Clone)]
struct FailureTracker {
    /// 当前状态
    state: FailureState,
    /// 超时计数
    timeout_count: u32,
}

/// 失效检测器
pub struct FailureDetector {
    /// 配置
    config: FailureDetectorConfig,
    /// 节点跟踪表
    trackers: Arc<RwLock<HashMap<ServerId, FailureTracker>>>,
}

impl FailureDetector {
    /// 创建新的失效检测器
    pub fn new(config: FailureDetectorConfig) -> Self {
        Self {
            config,
            trackers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加节点
    pub async fn add_node(&self, server_id: ServerId) {
        let mut trackers = self.trackers.write().await;
        trackers.insert(
            server_id,
            FailureTracker {
                state: FailureState::Alive,
                timeout_count: 0,
            },
        );
    }

    /// 报告心跳超时
    ///
    /// 返回新的状态
    pub async fn report_timeout(&self, server_id: &ServerId) -> Option<FailureState> {
        let mut trackers = self.trackers.write().await;

        if let Some(tracker) = trackers.get_mut(server_id) {
            tracker.timeout_count += 1;

            // 状态转换
            let new_state = if tracker.timeout_count >= self.config.confirm_threshold {
                FailureState::ConfirmedDead
            } else if tracker.timeout_count >= self.config.suspect_threshold {
                FailureState::Suspected
            } else {
                tracker.state
            };

            tracker.state = new_state;
            Some(new_state)
        } else {
            None
        }
    }

    /// 报告心跳恢复
    ///
    /// 重置超时计数,状态恢复为 Alive
    pub async fn report_alive(&self, server_id: &ServerId) {
        let mut trackers = self.trackers.write().await;

        if let Some(tracker) = trackers.get_mut(server_id) {
            tracker.timeout_count = 0;
            tracker.state = FailureState::Alive;
        }
    }

    /// 获取节点状态
    pub async fn get_state(&self, server_id: &ServerId) -> Option<FailureState> {
        let trackers = self.trackers.read().await;
        trackers.get(server_id).map(|t| t.state)
    }

    /// 移除节点
    pub async fn remove_node(&self, server_id: &ServerId) {
        let mut trackers = self.trackers.write().await;
        trackers.remove(server_id);
    }

    /// 获取所有确认失效的节点
    pub async fn get_dead_nodes(&self) -> Vec<ServerId> {
        let trackers = self.trackers.read().await;
        trackers
            .iter()
            .filter(|(_, t)| t.state == FailureState::ConfirmedDead)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// 获取所有可疑节点
    pub async fn get_suspected_nodes(&self) -> Vec<ServerId> {
        let trackers = self.trackers.read().await;
        trackers
            .iter()
            .filter(|(_, t)| t.state == FailureState::Suspected)
            .map(|(id, _)| id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_node() {
        let detector = FailureDetector::new(FailureDetectorConfig::default());
        let node_id = "server-1".to_string();

        detector.add_node(node_id.clone()).await;

        let state = detector.get_state(&node_id).await;
        assert_eq!(state, Some(FailureState::Alive));
    }

    #[tokio::test]
    async fn test_timeout_transitions() {
        let config = FailureDetectorConfig {
            suspect_threshold: 2,
            confirm_threshold: 4,
        };
        let detector = FailureDetector::new(config);
        let node_id = "server-1".to_string();

        detector.add_node(node_id.clone()).await;

        // 第 1 次超时 -> Alive
        let state = detector.report_timeout(&node_id).await;
        assert_eq!(state, Some(FailureState::Alive));

        // 第 2 次超时 -> Suspected
        let state = detector.report_timeout(&node_id).await;
        assert_eq!(state, Some(FailureState::Suspected));

        // 第 3 次超时 -> Suspected
        let state = detector.report_timeout(&node_id).await;
        assert_eq!(state, Some(FailureState::Suspected));

        // 第 4 次超时 -> ConfirmedDead
        let state = detector.report_timeout(&node_id).await;
        assert_eq!(state, Some(FailureState::ConfirmedDead));
    }

    #[tokio::test]
    async fn test_report_alive() {
        let detector = FailureDetector::new(FailureDetectorConfig::default());
        let node_id = "server-1".to_string();

        detector.add_node(node_id.clone()).await;

        // 触发超时
        detector.report_timeout(&node_id).await;
        detector.report_timeout(&node_id).await;

        // 恢复
        detector.report_alive(&node_id).await;

        let state = detector.get_state(&node_id).await;
        assert_eq!(state, Some(FailureState::Alive));
    }
}
