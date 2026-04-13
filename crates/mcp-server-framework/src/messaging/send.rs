use crate::protocol::MCPMessage;
use session_manager::ServerId;
use thiserror::Error;
use tokio::sync::mpsc;

/// 消息发送错误
#[derive(Debug, Error)]
pub enum SendMessageError {
    #[error("Channel closed")]
    ChannelClosed,

    #[error("Server not found: {0}")]
    ServerNotFound(ServerId),

    #[error("Send failed: {0}")]
    Failed(String),
}

/// 消息通道类型
pub type MessageSender = mpsc::Sender<MCPMessage>;

/// 消息路由表 (Server ID -> Channel)
pub type MessageRouter = std::collections::HashMap<ServerId, MessageSender>;

/// 发送消息到指定 Server
///
/// # 参数
/// - `router`: 消息路由表
/// - `to_server`: 目标 Server ID
/// - `message`: 要发送的消息
///
/// # 返回
/// 成功返回 Ok(())
pub async fn send_message(
    router: &MessageRouter,
    to_server: &ServerId,
    message: MCPMessage,
) -> Result<(), SendMessageError> {
    let sender = router
        .get(to_server)
        .ok_or_else(|| SendMessageError::ServerNotFound(to_server.clone()))?;

    sender
        .send(message)
        .await
        .map_err(|_| SendMessageError::ChannelClosed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_send_message() {
        let mut router = MessageRouter::new();
        let server_id = "server-1".to_string();
        let (tx, mut rx) = mpsc::channel(10);

        router.insert(server_id.clone(), tx);

        let msg = MCPMessage::request("test", Some(json!({"key": "value"})));
        send_message(&router, &server_id, msg.clone())
            .await
            .unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.message_type, msg.message_type);
    }

    #[tokio::test]
    async fn test_send_to_nonexistent_server() {
        let router = MessageRouter::new();
        let server_id = "server-1".to_string();

        let msg = MCPMessage::request("test", None);
        let result = send_message(&router, &server_id, msg).await;

        assert!(matches!(result, Err(SendMessageError::ServerNotFound(_))));
    }
}
