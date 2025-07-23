//! Universal Compute Client Module
//!
//! This module provides modular, capability-based compute integration.

pub mod client;
pub mod providers;
pub mod types;
// Removed ai_metadata - was over-engineered early implementation

pub use client::UniversalComputeClient;
pub use providers::*;
pub use types::*;
