//! Universal Storage Client Module
//!
//! This module provides modular, capability-based storage integration.

pub mod client;
pub mod providers;
pub mod types;
// Removed ai_metadata - was over-engineered early implementation

pub use client::UniversalStorageClient;
pub use providers::*;
pub use types::*;
