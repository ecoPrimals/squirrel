//! Universal Compute Client Module
//!
//! This module provides modular, capability-based compute integration.

pub mod ai_metadata;
pub mod client;
pub mod providers;
pub mod types;

// Re-export main types
pub use client::UniversalComputeClient;
pub use types::{
    ComputeClientConfig, ComputeOperation, ComputeSecurityRequirements, ResourceRequirements,
    UniversalComputeRequest, UniversalComputeResponse,
};
