// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

// AI routing and provider selection modules
#![allow(dead_code)] // AI API modules reserved for planned consumer integration

pub mod action_registry;
pub mod adapter; // Universal AI adapter (vendor-agnostic)
pub mod adapters;
pub mod bridge; // Bridge between universal and legacy interfaces
pub mod constraint_router;
pub mod constraints;
pub mod dignity;
pub mod discovery; // Capability-based AI provider discovery
pub mod http_provider_config; // HTTP provider configuration (vendor-agnostic)
pub mod router;
pub mod selector;
pub mod types;
pub mod universal; // Universal AI interface (vendor-agnostic)

// Re-export main router for tarpc_server
pub use router::AiRouter;

// Re-export universal AI interface (public API for downstream consumers)
#[expect(unused_imports, reason = "re-export for planned consumer")]
pub use universal::{
    AiCapability, BoxedAiCapability, ChatMessage, CostTier, MessageRole, ProviderMetadata,
    ProviderType, TokenUsage, UniversalAiRequest, UniversalAiResponse,
};

// Re-export adapter and discovery (public API for downstream consumers)
#[expect(unused_imports, reason = "re-export for planned consumer")]
pub use adapter::UniversalAiAdapter;
#[expect(unused_imports, reason = "re-export for planned consumer")]
pub use discovery::{discover_ai_provider, discover_ai_providers, has_ai_providers};
