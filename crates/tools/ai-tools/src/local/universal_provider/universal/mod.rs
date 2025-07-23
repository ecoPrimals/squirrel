//! Universal AI Provider Modules
//!
//! Modular implementation of the universal capability-based AI provider system.

pub mod config;
pub mod discovery;
pub mod matcher;
pub mod provider;
pub mod types;

// Re-export main types for convenience
pub use config::*;
pub use discovery::CapabilityDiscoveryEngine;
pub use matcher::{CapabilityMatcher, CapabilityWeights};
pub use provider::{
    create_universal_ai_provider, setup_development_capabilities, UniversalAIProvider,
};
pub use types::*;
