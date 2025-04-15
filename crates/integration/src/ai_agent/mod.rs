//! AI Agent integration for Squirrel
//!
//! This module provides integration between the AI Agent system and other
//! core components of the Squirrel platform.

// Comment out modules until we resolve dependency issues
mod config;
mod error;
pub mod types;
pub mod adapter;

// Re-export key components
pub use adapter::{
    AIAgentAdapter,
    AdapterStatus,
    create_ai_agent_adapter,
    create_ai_agent_adapter_with_config,
};
pub use config::{
    AIAgentConfig,
    CircuitBreakerConfig,
    ResourceLimits,
};
pub use error::AIAgentError;
pub use types::{
    AgentCapabilities,
    AgentContext,
    AgentRequest,
    AgentResponse,
    AnalysisOptions,
    CircuitBreakerState,
    Content,
    GenerationOptions,
    Prompt,
    Usage,
};

// Comment out until we resolve dependency issues
// pub use adapter::AIAgentAdapter;
// pub use config::AIAgentConfig;
// pub use error::AIAgentError;
// pub use types::{
//     AgentCapabilities, AgentContext, AgentRequest, AgentResponse,
//     AnalysisOptions, Content, GenerationOptions, Prompt, ResourceLimits,
// }; 