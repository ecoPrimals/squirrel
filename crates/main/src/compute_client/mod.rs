//! Universal Compute Client Module
//!
//! This module provides modular, capability-based compute integration.

pub mod types;
pub mod client;
pub mod ai_metadata;
pub mod providers;

// Re-export main types
pub use client::UniversalComputeClient;
pub use types::{
    UniversalComputeRequest, UniversalComputeResponse, ComputeClientConfig,
    ComputeOperation, ResourceRequirements, ComputeSecurityRequirements,
}; 