//! Universal Security Client Module
//!
//! This module provides modular, capability-based security integration.

pub mod client;
pub mod providers;
pub mod types;
// Removed ai_metadata - was over-engineered early implementation

pub use client::UniversalSecurityClient;
pub use providers::*;
pub use types::*;
