use std::sync::atomic::{AtomicU64, Ordering};

/// Server 监控指标
#[derive(Debug)]
pub struct ServerMetrics {
    /// 接收消息数
    messages_received: AtomicU64,
    /// 发送消息数
    messages_sent: AtomicU64,
    /// 路由延迟 (纳秒累计)
    routing_latency_ns: AtomicU64,
    /// 路由次数
    routing_count: AtomicU64,
    /// 拓扑变更次数
    topology_changes: AtomicU64,
    /// Gossip 消息数
    gossip_messages: AtomicU64,
}

impl ServerMetrics {
    /// 创建新的指标
    pub fn new() -> Self {
        Self {
            messages_received: AtomicU64::new(0),
            messages_sent: AtomicU64::new(0),
            routing_latency_ns: AtomicU64::new(0),
            routing_count: AtomicU64::new(0),
            topology_changes: AtomicU64::new(0),
            gossip_messages: AtomicU64::new(0),
        }
    }

    /// 记录接收消息
    pub fn record_message_received(&self) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录发送消息
    pub fn record_message_sent(&self) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录路由延迟
    pub fn record_routing_latency(&self, latency_ns: u64) {
        self.routing_latency_ns
            .fetch_add(latency_ns, Ordering::Relaxed);
        self.routing_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录拓扑变更
    pub fn record_topology_change(&self) {
        self.topology_changes.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录 Gossip 消息
    pub fn record_gossip_message(&self) {
        self.gossip_messages.fetch_add(1, Ordering::Relaxed);
    }

    /// 获取指标快照
    pub fn snapshot(&self) -> MetricsSnapshot {
        let routing_count = self.routing_count.load(Ordering::Relaxed);
        let avg_routing_latency = if routing_count > 0 {
            self.routing_latency_ns.load(Ordering::Relaxed) / routing_count
        } else {
            0
        };

        MetricsSnapshot {
            messages_received: self.messages_received.load(Ordering::Relaxed),
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            avg_routing_latency_ns: avg_routing_latency,
            routing_count,
            topology_changes: self.topology_changes.load(Ordering::Relaxed),
            gossip_messages: self.gossip_messages.load(Ordering::Relaxed),
        }
    }
}

impl Default for ServerMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// 指标快照
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub messages_received: u64,
    pub messages_sent: u64,
    pub avg_routing_latency_ns: u64,
    pub routing_count: u64,
    pub topology_changes: u64,
    pub gossip_messages: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics() {
        let metrics = ServerMetrics::new();

        metrics.record_message_received();
        metrics.record_message_received();
        metrics.record_message_sent();
        metrics.record_routing_latency(1000);
        metrics.record_routing_latency(2000);
        metrics.record_topology_change();
        metrics.record_gossip_message();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.messages_received, 2);
        assert_eq!(snapshot.messages_sent, 1);
        assert_eq!(snapshot.avg_routing_latency_ns, 1500); // (1000 + 2000) / 2
        assert_eq!(snapshot.routing_count, 2);
        assert_eq!(snapshot.topology_changes, 1);
        assert_eq!(snapshot.gossip_messages, 1);
    }
}
