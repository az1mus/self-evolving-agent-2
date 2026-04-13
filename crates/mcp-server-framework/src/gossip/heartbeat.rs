use crate::gossip::GossipMessage;
use chrono::{DateTime, Duration, Utc};
use session_manager::ServerId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Heartbeat 配置
#[derive(Debug, Clone)]
pub struct HeartbeatConfig {
    /// 心跳间隔 (秒)
    pub interval_secs: u64,
    /// 心跳超时 (秒)
    pub timeout_secs: u64,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_secs: 5,
            timeout_secs: 15,
        }
    }
}

/// 心跳状态
#[derive(Debug, Clone)]
pub struct HeartbeatState {
    /// 最后收到的心跳时间
    pub last_heartbeat: DateTime<Utc>,
}

/// Heartbeat 管理器
pub struct HeartbeatManager {
    /// 本地 Server ID
    server_id: ServerId,
    /// 配置
    config: HeartbeatConfig,
    /// 心跳状态表
    heartbeat_states: Arc<RwLock<HashMap<ServerId, HeartbeatState>>>,
}

impl HeartbeatManager {
    /// 创建新的 Heartbeat 管理器
    pub fn new(server_id: ServerId, config: HeartbeatConfig) -> Self {
        Self {
            server_id,
            config,
            heartbeat_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 启动心跳发送任务
    pub fn start_heartbeat_task(
        &self,
        gossip_tx: tokio::sync::mpsc::Sender<GossipMessage>,
    ) -> tokio::task::JoinHandle<()> {
        let server_id = self.server_id.clone();
        let interval = self.config.interval_secs;

        tokio::spawn(async move {
            let mut interval_timer =
                tokio::time::interval(tokio::time::Duration::from_secs(interval));

            loop {
                interval_timer.tick().await;

                // 发送心跳
                let heartbeat = GossipMessage::heartbeat(server_id.clone());
                if gossip_tx.send(heartbeat).await.is_err() {
                    tracing::error!("Failed to send heartbeat");
                    break;
                }

                tracing::trace!("Heartbeat sent");
            }
        })
    }

    /// 接收心跳
    pub async fn receive_heartbeat(&self, server_id: &ServerId) {
        let mut states = self.heartbeat_states.write().await;
        states.insert(
            server_id.clone(),
            HeartbeatState {
                last_heartbeat: Utc::now(),
            },
        );
    }

    /// 检查超时的节点
    pub async fn check_timeouts(&self) -> Vec<ServerId> {
        let states = self.heartbeat_states.read().await;
        let now = Utc::now();
        let timeout_duration = Duration::seconds(self.config.timeout_secs as i64);

        let mut timed_out = Vec::new();

        for (server_id, state) in states.iter() {
            if now - state.last_heartbeat > timeout_duration {
                timed_out.push(server_id.clone());
            }
        }

        timed_out
    }

    /// 移除节点
    pub async fn remove_node(&self, server_id: &ServerId) {
        let mut states = self.heartbeat_states.write().await;
        states.remove(server_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_receive_heartbeat() {
        let server_id = "server-1".to_string();
        let manager = HeartbeatManager::new(server_id, HeartbeatConfig::default());

        let peer_id = "server-2".to_string();
        manager.receive_heartbeat(&peer_id).await;

        let states = manager.heartbeat_states.read().await;
        assert!(states.contains_key(&peer_id));
    }

    #[tokio::test]
    async fn test_check_timeouts() {
        let server_id = "server-1".to_string();
        let config = HeartbeatConfig {
            interval_secs: 1,
            timeout_secs: 1,
        };
        let manager = HeartbeatManager::new(server_id, config);

        let peer_id = "server-2".to_string();

        // 模拟过期的心跳
        {
            let mut states = manager.heartbeat_states.write().await;
            states.insert(
                peer_id.clone(),
                HeartbeatState {
                    last_heartbeat: Utc::now() - Duration::seconds(10),
                },
            );
        }

        // 检查超时
        let timed_out = manager.check_timeouts().await;
        assert!(timed_out.contains(&peer_id));
    }

    #[tokio::test]
    async fn test_remove_node() {
        let server_id = "server-1".to_string();
        let manager = HeartbeatManager::new(server_id, HeartbeatConfig::default());

        let peer_id = "server-2".to_string();
        manager.receive_heartbeat(&peer_id).await;

        manager.remove_node(&peer_id).await;

        let states = manager.heartbeat_states.read().await;
        assert!(!states.contains_key(&peer_id));
    }
}
