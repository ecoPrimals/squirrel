// AI routing and provider selection modules

pub(crate) mod action_registry;
pub mod adapter; // Universal AI adapter (vendor-agnostic)
pub(crate) mod adapters;
pub mod bridge; // Bridge between universal and legacy interfaces
pub(crate) mod constraint_router;
pub(crate) mod constraints;
pub mod discovery; // Capability-based AI provider discovery
pub(crate) mod router;
pub(crate) mod selector;
pub(crate) mod types;
pub mod universal; // Universal AI interface (vendor-agnostic)

#[cfg(test)]
mod router_tests;

// Re-export main router for tarpc_server
pub use router::AiRouter;

// Re-export universal AI interface
pub use universal::{
    AiCapability, BoxedAiCapability, ChatMessage, CostTier, MessageRole, ProviderMetadata,
    ProviderType, TokenUsage, UniversalAiRequest, UniversalAiResponse,
};

// Re-export adapter and discovery
pub use adapter::UniversalAiAdapter;
pub use discovery::{discover_ai_provider, discover_ai_providers, has_ai_providers};
