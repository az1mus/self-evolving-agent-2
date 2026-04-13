use session_manager::ServerId;
use tokio::sync::broadcast;

/// Server 事件
#[derive(Debug, Clone)]
pub enum ServerEvent {
    /// Peer 加入
    PeerJoined { server_id: ServerId },
    /// Peer 离开
    PeerLeft { server_id: ServerId },
    /// 工具公告
    ToolAnnounced {
        server_id: ServerId,
        tools: Vec<String>,
    },
    /// Peer 可疑
    PeerSuspected { server_id: ServerId },
    /// Peer 确认失效
    PeerDead { server_id: ServerId },
    /// 拓扑变更
    TopologyChanged { version: u64 },
    /// 消息接收
    MessageReceived { from: ServerId },
    /// Server 启动
    ServerStarted { server_id: ServerId },
    /// Server 停止
    ServerStopped { server_id: ServerId },
}

/// 事件总线
///
/// 发布-订阅模式的事件系统
pub struct EventBus {
    sender: broadcast::Sender<ServerEvent>,
}

impl EventBus {
    /// 创建新的事件总线
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// 发布事件
    pub fn publish(&self, event: ServerEvent) {
        if let Err(e) = self.sender.send(event) {
            tracing::warn!("Failed to publish event: {}", e);
        }
    }

    /// 订阅事件
    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus() {
        let bus = EventBus::new(10);
        let mut receiver = bus.subscribe();

        let server_id = "server-1".to_string();
        bus.publish(ServerEvent::PeerJoined {
            server_id: server_id.clone(),
        });

        let event = receiver.recv().await.unwrap();
        match event {
            ServerEvent::PeerJoined { server_id: id } => {
                assert_eq!(id, server_id);
            }
            _ => panic!("Expected PeerJoined event"),
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new(10);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        let server_id = "server-1".to_string();
        bus.publish(ServerEvent::PeerLeft {
            server_id: server_id.clone(),
        });

        // 两个订阅者都应该收到事件
        let event1 = rx1.recv().await.unwrap();
        let event2 = rx2.recv().await.unwrap();

        assert!(matches!(event1, ServerEvent::PeerLeft { .. }));
        assert!(matches!(event2, ServerEvent::PeerLeft { .. }));
    }
}
