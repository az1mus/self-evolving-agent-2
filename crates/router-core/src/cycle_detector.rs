use crate::error::RouterError;
use crate::message::Message;
use session_manager::ServerId;

/// 循环检测器
pub struct CycleDetector;

impl CycleDetector {
    pub fn new() -> Self {
        Self
    }

    /// 检查消息是否会导致循环
    ///
    /// 返回 Ok(()) 表示无循环,Err 表示检测到循环
    pub fn check(&self, message: &Message, target_server: &str) -> Result<(), RouterError> {
        // 检查 1: 是否已访问过目标 Server
        if message.routing.has_visited(target_server) {
            return Err(RouterError::CycleDetected(target_server.to_string()));
        }

        // 检查 2: 是否超过最大跳数
        if message.routing.is_max_hops_exceeded() {
            return Err(RouterError::MaxHopsExceeded(
                message.routing.hop_count,
                message.routing.max_hops,
            ));
        }

        Ok(())
    }

    /// 检查是否可以继续路由(不考虑具体目标)
    pub fn can_route(&self, message: &Message) -> Result<(), RouterError> {
        if message.routing.is_max_hops_exceeded() {
            return Err(RouterError::MaxHopsExceeded(
                message.routing.hop_count,
                message.routing.max_hops,
            ));
        }

        Ok(())
    }
}

impl Default for CycleDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// 路由上下文
///
/// 维护当前路由状态
pub struct RoutingContext {
    /// 循环检测器
    cycle_detector: CycleDetector,
}

impl RoutingContext {
    pub fn new() -> Self {
        Self {
            cycle_detector: CycleDetector::new(),
        }
    }

    /// 检查是否可以路由到目标 Server
    pub fn can_route_to(&self, message: &Message, target: &str) -> Result<(), RouterError> {
        self.cycle_detector.check(message, target)
    }

    /// 标记消息已路由到指定 Server
    pub fn mark_visited(&self, message: &mut Message, server_id: ServerId) {
        message.visit_server(server_id);
    }
}

impl Default for RoutingContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_message_with_hops(hop_count: u32, max_hops: u32) -> Message {
        let mut msg = Message::new(
            Uuid::new_v4(),
            crate::message::MessageContent::unstructured("test"),
            max_hops,
        );

        for _ in 0..hop_count {
            msg.visit_server(Uuid::new_v4().to_string());
        }

        msg
    }

    #[test]
    fn test_cycle_detector_no_cycle() {
        let detector = CycleDetector::new();
        let msg = create_test_message_with_hops(0, 10);

        assert!(detector.check(&msg, "server-a").is_ok());
    }

    #[test]
    fn test_cycle_detector_cycle_detected() {
        let detector = CycleDetector::new();
        let mut msg = create_test_message_with_hops(0, 10);
        msg.visit_server("server-a".to_string());

        let result = detector.check(&msg, "server-a");
        assert!(matches!(result, Err(RouterError::CycleDetected(_))));
    }

    #[test]
    fn test_cycle_detector_max_hops_exceeded() {
        let detector = CycleDetector::new();
        let msg = create_test_message_with_hops(10, 10);

        let result = detector.check(&msg, "server-b");
        assert!(matches!(result, Err(RouterError::MaxHopsExceeded(_, _))));
    }

    #[test]
    fn test_routing_context() {
        let ctx = RoutingContext::new();
        let mut msg = create_test_message_with_hops(0, 10);

        // 可以路由到 server-a
        assert!(ctx.can_route_to(&msg, "server-a").is_ok());

        // 标记已访问
        ctx.mark_visited(&mut msg, "server-a".to_string());

        // 不能再路由到 server-a
        assert!(ctx.can_route_to(&msg, "server-a").is_err());

        // 可以路由到其他 server
        assert!(ctx.can_route_to(&msg, "server-b").is_ok());
    }
}
