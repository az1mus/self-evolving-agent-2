// Module exports
pub mod gossip;
pub mod messaging;
pub mod protocol;
pub mod runtime;
pub mod server;
pub mod topology;

// Re-exports for convenience
pub use gossip::{FailureDetector, GossipHandler, GossipMessage, HeartbeatManager};
pub use messaging::{
    forward_message, make_routing_decision, send_message, MessageDispatcher, RoutingDecision,
};
pub use protocol::{MCPCodec, MCPMessage, MCPMessageType, Tool, ToolCall, ToolResult};
pub use runtime::{EventBus, ServerConfig, ServerEvent, ServerMetrics, ServerRunner};
pub use server::{MCPServer, ServerBase, ServerHandle, ServerLifecycle, ServerState};
pub use topology::{LocalTopologyState, TopologyConsistency, TopologyQuery, TopologyUpdate};
