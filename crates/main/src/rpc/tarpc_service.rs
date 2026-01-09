//! tarpc Service Definition
//!
//! High-performance binary RPC service for Squirrel-to-Squirrel communication.
//! This enables federation, distributed AI coordination, and cross-tower mesh.

use serde::{Deserialize, Serialize};

/// AI query request for tarpc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarpcQueryRequest {
    pub prompt: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
}

/// AI query response for tarpc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarpcQueryResponse {
    pub response: String,
    pub provider: String,
    pub model: String,
    pub tokens_used: Option<usize>,
    pub latency_ms: u64,
}

/// Provider information for tarpc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarpcProviderInfo {
    pub id: String,
    pub name: String,
    pub capabilities: Vec<String>,
    pub online: bool,
    pub cost_tier: String,
}

/// Health status for tarpc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TarpcHealthStatus {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub active_providers: usize,
}

/// tarpc service definition for Squirrel AI coordination
#[tarpc::service]
pub trait SquirrelRpc {
    /// Query AI with a prompt
    async fn query_ai(request: TarpcQueryRequest) -> Result<TarpcQueryResponse, String>;

    /// List available AI providers
    async fn list_providers() -> Result<Vec<TarpcProviderInfo>, String>;

    /// Get health status
    async fn health_check() -> Result<TarpcHealthStatus, String>;

    /// Announce capabilities to the mesh
    async fn announce_capabilities(capabilities: Vec<String>) -> Result<bool, String>;

    /// Discover other Squirrel instances in the mesh
    async fn discover_peers() -> Result<Vec<String>, String>;
}
