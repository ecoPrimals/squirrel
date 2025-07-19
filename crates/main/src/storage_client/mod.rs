//! Universal Storage Client Module
//!
//! This module provides modular, capability-based storage integration.

pub mod types;
pub mod client;
pub mod ai_metadata;
pub mod providers;

// Re-export main types
pub use client::UniversalStorageClient;
pub use types::{
    UniversalStorageRequest, UniversalStorageResponse, StorageClientConfig,
    StorageOperation, DataClassification, PerformanceRequirements,
}; 