//! Universal Security Client Module
//!
//! This module provides modular, capability-based security integration.

pub mod types;
pub mod client;
pub mod ai_metadata;
pub mod providers;

// Re-export main types
pub use client::UniversalSecurityClient;
pub use types::{
    UniversalSecurityRequest, UniversalSecurityResponse, SecurityClientConfig,
    SecurityOperation, ComplianceRequirements, TrustLevel,
}; 