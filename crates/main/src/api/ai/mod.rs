// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

// AI routing and provider selection modules

pub(crate) mod action_registry;
pub mod adapter; // Universal AI adapter (vendor-agnostic)
pub(crate) mod adapters;
pub mod bridge; // Bridge between universal and legacy interfaces
pub(crate) mod constraint_router;
pub(crate) mod constraints;
pub mod discovery; // Capability-based AI provider discovery
pub(crate) mod http_provider_config; // HTTP provider configuration (vendor-agnostic)
pub(crate) mod router;
pub(crate) mod selector;
pub(crate) mod types;
pub mod universal; // Universal AI interface (vendor-agnostic)

#[cfg(test)]
mod router_tests;

// Re-export main router for tarpc_server
pub use router::AiRouter;

// Re-export universal AI interface (public API for downstream consumers)
#[allow(unused_imports)]
pub use universal::{
    AiCapability, BoxedAiCapability, ChatMessage, CostTier, MessageRole, ProviderMetadata,
    ProviderType, TokenUsage, UniversalAiRequest, UniversalAiResponse,
};

// Re-export adapter and discovery (public API for downstream consumers)
#[allow(unused_imports)]
pub use adapter::UniversalAiAdapter;
#[allow(unused_imports)]
pub use discovery::{discover_ai_provider, discover_ai_providers, has_ai_providers};
