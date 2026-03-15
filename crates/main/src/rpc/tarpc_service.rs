// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab
// ORC-Notice: RPC service types licensed under ORC

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
//!
//! ## wateringHole IPC Types (UNIVERSAL_IPC_STANDARD_V3)
//!
//! - `Arc<str>` for identifiers (provider, model, capability names)
//! - `String` for user content (prompts, responses, descriptions)

// Note: This module is feature-gated via #[cfg(feature = "tarpc-rpc")] in mod.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Query AI request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAiParams {
    /// The prompt to send to the AI (user content)
    pub prompt: String,

    /// Optional model to use (identifier)
    pub model: Option<Arc<str>>,

    /// Optional max tokens
    pub max_tokens: Option<usize>,

    /// Optional temperature (0.0-1.0)
    pub temperature: Option<f64>,
}

/// Query AI response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAiResult {
    /// AI response text (user content)
    pub response: String,

    /// Provider used (identifier)
    pub provider: Arc<str>,

    /// Model used (identifier)
    pub model: Arc<str>,

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
    /// Provider ID (identifier)
    pub id: Arc<str>,

    /// Provider name (identifier)
    pub name: Arc<str>,

    /// Available models (identifiers)
    pub models: Vec<Arc<str>>,

    /// Capabilities (identifiers)
    pub capabilities: Vec<Arc<str>>,

    /// Online status
    pub online: bool,

    /// Average latency (ms)
    pub avg_latency_ms: Option<f64>,

    /// Cost tier (identifier)
    pub cost_tier: Arc<str>,
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
    /// Service name (identifier)
    pub service: Arc<str>,

    /// Capabilities (identifiers)
    pub capabilities: Vec<Arc<str>>,

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
///
/// ## Semantic Naming Convention (wateringHole standard)
///
/// These methods follow the semantic naming convention `{domain}_{operation}`:
///
/// | Method | Domain | Semantic Name |
/// |--------|--------|---------------|
/// | `query_ai` | ai | `ai.query` (JSON-RPC) |
/// | `list_providers` | ai | `ai.list_providers` (JSON-RPC) |
/// | `announce_capabilities` | capability | `capability.announce` (JSON-RPC) |
/// | `health` | system | `system.health` (JSON-RPC) |
/// | `ping` | system | `system.ping` (JSON-RPC) |
/// | `discover_peers` | discovery | `discovery.peers` (JSON-RPC) |
/// | `execute_tool` | tool | `tool.execute` (JSON-RPC) |
///
/// Note: tarpc uses Rust method names directly. For protocol-level semantic
/// names, use the JSON-RPC interface which supports both legacy and semantic names.
#[tarpc::service]
pub trait SquirrelRpc {
    /// Query AI with a prompt
    ///
    /// Semantic: `ai.query` (JSON-RPC)
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
    /// Semantic: `ai.list_providers` (JSON-RPC)
    ///
    /// # Returns
    ///
    /// List of providers with status and capabilities
    async fn list_providers() -> ListProvidersResult;

    /// Announce service capabilities
    ///
    /// Semantic: `capability.announce` (JSON-RPC)
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
    /// Semantic: `system.health` (JSON-RPC)
    ///
    /// # Returns
    ///
    /// Server health status and metrics
    async fn health() -> HealthCheckResult;

    /// Ping (connectivity test)
    ///
    /// Semantic: `system.ping` (JSON-RPC)
    ///
    /// # Returns
    ///
    /// Pong response with timestamp
    async fn ping() -> String;

    /// Discover peers (other primals)
    ///
    /// Semantic: `discovery.peers` (JSON-RPC)
    ///
    /// # Returns
    ///
    /// List of discovered primal services
    async fn discover_peers() -> Vec<String>;

    /// Execute a tool
    ///
    /// Semantic: `tool.execute` (JSON-RPC)
    ///
    /// # Arguments
    ///
    /// * `tool` - Tool name (identifier)
    /// * `args` - Tool arguments
    ///
    /// # Returns
    ///
    /// Tool execution result
    async fn execute_tool(tool: Arc<str>, args: HashMap<String, String>) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_ai_params_serialization() {
        let params = QueryAiParams {
            prompt: "Hello".to_string(),
            model: Some(Arc::from("gpt-4")),
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
            id: Arc::from("test"),
            name: Arc::from("Test Provider"),
            models: vec![Arc::from("model1")],
            capabilities: vec![Arc::from("text")],
            online: true,
            avg_latency_ms: Some(100.0),
            cost_tier: Arc::from("free"),
        };

        let serialized = serde_json::to_string(&info).unwrap();
        let deserialized: ProviderInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(info.id, deserialized.id);
        assert_eq!(info.online, deserialized.online);
    }
}
