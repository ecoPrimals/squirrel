//! Storage capability (data persistence)

use crate::error::PrimalError;
// Native async traits (Rust 1.75+) - no async_trait needed!
use serde::{Deserialize, Serialize};

/// Request to store data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreRequest {
    /// Key or path
    pub key: String,

    /// Data to store
    pub data: Vec<u8>,

    /// Optional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Request to retrieve data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveRequest {
    /// Key or path
    pub key: String,
}

/// Response with retrieved data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveResponse {
    /// Key
    pub key: String,

    /// Retrieved data
    pub data: Vec<u8>,

    /// Metadata
    pub metadata: std::collections::HashMap<String, String>,

    /// Last modified timestamp
    pub modified_at: u64,
}

/// Capability for data storage
///
/// Typically provided by LoamSpine or other storage providers.

pub trait StorageCapability: Send + Sync {
    /// Store data
    async fn store(&self, request: StoreRequest) -> Result<(), PrimalError>;

    /// Retrieve data
    async fn retrieve(&self, request: RetrieveRequest) -> Result<RetrieveResponse, PrimalError>;

    /// Delete data
    async fn delete(&self, key: String) -> Result<(), PrimalError>;

    /// List keys with optional prefix
    async fn list(&self, prefix: Option<String>) -> Result<Vec<String>, PrimalError>;
}
