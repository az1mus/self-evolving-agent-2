mod classifier;
mod cycle_detector;
mod error;
mod message;
mod preprocessor;
mod router;

pub use classifier::{Classifier, MockClassifier, RuleBasedClassifier};
pub use cycle_detector::{CycleDetector, RoutingContext};
pub use error::RouterError;
pub use message::{Message, MessageContent, ProcessingType, RoutingMetadata};
pub use preprocessor::{
    CacheMatcher, Normalizer, Preprocessor, PreprocessorPipeline, RuleCompressor,
};
pub use router::{
    CapabilityRouter, ChainedRouter, CompositeRouter, ParallelRouter, Router, RouterBuilder,
    RouterCore,
};

// 重新导出 session-manager 类型,方便使用
pub use session_manager::{CacheManager, ServerId, Session, SessionId, SessionManager};
