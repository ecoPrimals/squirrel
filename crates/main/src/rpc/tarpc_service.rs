//! tarpc RPC Service Definition
//!
//! Modern, high-performance binary RPC using tarpc framework.
//! This service mirrors the JSON-RPC methods but provides:
//! - Type-safe RPC calls
//! - Binary serialization (smaller payloads)
//! - Lower latency
//! - Cascading cancellation
//! - Deadline propagation
//!
//! ## Architecture
//!
//! ```text
//! Universal Transport → tarpc Protocol → Service Impl → AI Router → Response
//! ```

#![cfg(feature = "tarpc-rpc")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Query AI request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAiParams {
    /// The prompt to send to the AI
    pub prompt: String,

    /// Optional model to use
    pub model: Option<String>,

    /// Optional max tokens
    pub max_tokens: Option<usize>,

    /// Optional temperature (0.0-1.0)
    pub temperature: Option<f64>,
}

/// Query AI response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAiResult {
    /// AI response text
    pub response: String,

    /// Provider used
    pub provider: String,

    /// Model used
    pub model: String,

    /// Tokens used (if available)
    pub tokens_used: Option<usize>,

    /// Latency in milliseconds
    pub latency_ms: u64,

    /// Success flag
    pub success: bool,
}

/// Provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Provider ID
    pub id: String,

    /// Provider name
    pub name: String,

    /// Available models
    pub models: Vec<String>,

    /// Capabilities
    pub capabilities: Vec<String>,

    /// Online status
    pub online: bool,

    /// Average latency (ms)
    pub avg_latency_ms: Option<f64>,

    /// Cost tier
    pub cost_tier: String,
}

/// List providers response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListProvidersResult {
    /// Total providers
    pub total: usize,

    /// Provider list
    pub providers: Vec<ProviderInfo>,
}

/// Capability announcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceCapabilitiesParams {
    /// Service name
    pub service: String,

    /// Capabilities
    pub capabilities: Vec<String>,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Capability announcement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceCapabilitiesResult {
    /// Success flag
    pub success: bool,

    /// Response message
    pub message: String,

    /// Timestamp
    pub announced_at: String,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Status
    pub status: String,

    /// Version
    pub version: String,

    /// Uptime (seconds)
    pub uptime_seconds: u64,

    /// Active providers
    pub active_providers: usize,

    /// Requests processed
    pub requests_processed: u64,

    /// Average response time (ms)
    pub avg_response_time_ms: Option<f64>,
}

/// Squirrel RPC Service
///
/// This service defines the RPC interface for Squirrel using tarpc.
/// It mirrors the JSON-RPC interface but provides type-safe,
/// high-performance binary RPC.
#[tarpc::service]
pub trait SquirrelRpc {
    /// Query AI with a prompt
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters (prompt, model, etc.)
    ///
    /// # Returns
    ///
    /// AI response with metadata
    async fn query_ai(params: QueryAiParams) -> QueryAiResult;

    /// List available AI providers
    ///
    /// # Returns
    ///
    /// List of providers with status and capabilities
    async fn list_providers() -> ListProvidersResult;

    /// Announce service capabilities
    ///
    /// # Arguments
    ///
    /// * `params` - Service name, capabilities, metadata
    ///
    /// # Returns
    ///
    /// Acknowledgment with timestamp
    async fn announce_capabilities(
        params: AnnounceCapabilitiesParams,
    ) -> AnnounceCapabilitiesResult;

    /// Health check
    ///
    /// # Returns
    ///
    /// Server health status and metrics
    async fn health() -> HealthCheckResult;

    /// Ping (connectivity test)
    ///
    /// # Returns
    ///
    /// Pong response with timestamp
    async fn ping() -> String;

    /// Discover peers (other primals)
    ///
    /// # Returns
    ///
    /// List of discovered primal services
    async fn discover_peers() -> Vec<String>;

    /// Execute a tool
    ///
    /// # Arguments
    ///
    /// * `tool` - Tool name
    /// * `args` - Tool arguments
    ///
    /// # Returns
    ///
    /// Tool execution result
    async fn execute_tool(tool: String, args: HashMap<String, String>) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_ai_params_serialization() {
        let params = QueryAiParams {
            prompt: "Hello".to_string(),
            model: Some("gpt-4".to_string()),
            max_tokens: Some(100),
            temperature: Some(0.7),
        };

        let serialized = serde_json::to_string(&params).unwrap();
        let deserialized: QueryAiParams = serde_json::from_str(&serialized).unwrap();

        assert_eq!(params.prompt, deserialized.prompt);
        assert_eq!(params.model, deserialized.model);
    }

    #[test]
    fn test_provider_info_serialization() {
        let info = ProviderInfo {
            id: "test".to_string(),
            name: "Test Provider".to_string(),
            models: vec!["model1".to_string()],
            capabilities: vec!["text".to_string()],
            online: true,
            avg_latency_ms: Some(100.0),
            cost_tier: "free".to_string(),
        };

        let serialized = serde_json::to_string(&info).unwrap();
        let deserialized: ProviderInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(info.id, deserialized.id);
        assert_eq!(info.online, deserialized.online);
    }
}
