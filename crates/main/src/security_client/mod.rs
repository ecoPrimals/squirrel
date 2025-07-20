//! Universal Security Client Module
//!
//! This module provides modular, capability-based security integration.

pub mod ai_metadata;
pub mod client;
pub mod providers;
pub mod types;

// Re-export main types
pub use client::UniversalSecurityClient;
pub use types::{
    ComplianceRequirements, SecurityClientConfig, SecurityOperation, TrustLevel,
    UniversalSecurityRequest, UniversalSecurityResponse,
};
