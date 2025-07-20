//! Universal Storage Client Module
//!
//! This module provides modular, capability-based storage integration.

pub mod ai_metadata;
pub mod client;
pub mod providers;
pub mod types;

// Re-export main types
pub use client::UniversalStorageClient;
pub use types::{
    DataClassification, PerformanceRequirements, StorageClientConfig, StorageOperation,
    UniversalStorageRequest, UniversalStorageResponse,
};
