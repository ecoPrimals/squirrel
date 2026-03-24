// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors
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
//! Universal Transport → tarpc Protocol → Service Impl → JsonRpcServer Handlers → Response
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

// =============================================================================
// AI domain types
// =============================================================================

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

// =============================================================================
// Capability domain types
// =============================================================================

/// Capability announcement parameters (matches JSON-RPC AnnounceCapabilitiesRequest)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceCapabilitiesParams {
    /// Capability namespaces to announce
    pub capabilities: Vec<String>,

    /// Announcing primal's name (optional, required for routing)
    pub primal: Option<String>,

    /// Unix socket path where the announcing primal listens
    pub socket_path: Option<String>,

    /// Tool names the primal provides
    pub tools: Option<Vec<String>>,

    /// Sub-federations (optional)
    pub sub_federations: Option<Vec<String>>,

    /// Genetic families (optional)
    pub genetic_families: Option<Vec<String>>,
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

    /// Number of tools registered for routing
    pub tools_registered: usize,
}

/// Capability discover response (JSON-serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDiscoverResult {
    /// Primal name
    pub primal: String,

    /// Capability method names
    pub capabilities: Vec<String>,

    /// Version
    pub version: String,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

// =============================================================================
// System domain types
// =============================================================================

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

/// System metrics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetricsResult {
    /// Requests handled
    pub requests_handled: u64,

    /// Errors
    pub errors: u64,

    /// Uptime (seconds)
    pub uptime_seconds: u64,

    /// Average response time (ms)
    pub avg_response_time_ms: Option<f64>,

    /// Success rate (0.0-1.0)
    pub success_rate: f64,
}

/// Ping response (structured)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    /// Pong flag
    pub pong: bool,

    /// Timestamp
    pub timestamp: String,

    /// Version
    pub version: String,
}

// =============================================================================
// Discovery domain types
// =============================================================================

/// Peer info from discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer ID
    pub id: String,

    /// Socket path
    pub socket: String,

    /// Capabilities
    pub capabilities: Vec<String>,

    /// Discovery method
    pub discovered_via: String,
}

/// Discovery peers response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryPeersResult {
    /// List of peers
    pub peers: Vec<PeerInfo>,

    /// Total count
    pub total: usize,

    /// Discovery method used
    pub discovery_method: String,
}

// =============================================================================
// Tool domain types
// =============================================================================

/// Tool source (built-in or remote)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolSource {
    /// Built into squirrel
    Builtin,

    /// Announced by a remote primal
    Remote {
        /// ID of the remote primal that announced the tool
        primal: String,
    },
}

/// Tool list entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolListEntry {
    /// Tool name
    pub name: String,

    /// Description
    pub description: String,

    /// Domain
    pub domain: String,

    /// Source of the tool
    pub source: ToolSource,

    /// Optional input schema (JSON)
    pub input_schema: Option<serde_json::Value>,
}

/// Tool list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolListResult {
    /// Tools
    pub tools: Vec<ToolListEntry>,

    /// Total count
    pub total: usize,
}

/// Tool execute result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecuteResult {
    /// Tool name
    pub tool: String,

    /// Success flag
    pub success: bool,

    /// Output
    pub output: String,

    /// Error (if any)
    pub error: Option<String>,

    /// Timestamp
    pub timestamp: String,
}

// =============================================================================
// Context domain types
// =============================================================================

/// Context create params
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCreateParams {
    /// Optional session ID (auto-generated if omitted)
    pub session_id: Option<String>,

    /// Initial metadata
    pub metadata: Option<serde_json::Value>,
}

/// Context create result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCreateResult {
    /// Context ID
    pub id: String,

    /// Version
    pub version: u64,

    /// Created at
    pub created_at: String,

    /// Metadata
    pub metadata: serde_json::Value,
}

/// Context update params
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUpdateParams {
    /// Context ID to update
    pub id: String,

    /// Data to merge
    pub data: serde_json::Value,
}

/// Context update result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUpdateResult {
    /// Context ID
    pub id: String,

    /// Version
    pub version: u64,

    /// Updated at
    pub updated_at: String,
}

/// Context summarize params
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummarizeParams {
    /// Context ID
    pub id: String,
}

/// Context summarize result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummarizeResult {
    /// Context ID
    pub id: String,

    /// Version
    pub version: u64,

    /// Summary
    pub summary: String,

    /// Data
    pub data: serde_json::Value,

    /// Synchronized flag
    pub synchronized: bool,
}

// =============================================================================
// Lifecycle domain types
// =============================================================================

/// Lifecycle register result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleRegisterResult {
    /// Success flag
    pub success: bool,

    /// Message
    pub message: String,
}

/// Lifecycle status result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleStatusResult {
    /// Status
    pub status: String,

    /// Version
    pub version: String,

    /// Uptime (seconds)
    pub uptime_seconds: u64,
}

// =============================================================================
// Service trait
// =============================================================================

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
/// | tarpc Method | Semantic Name (JSON-RPC) |
/// |--------------|--------------------------|
/// | `ai_query` | `ai.query` |
/// | `ai_complete` | `ai.complete` |
/// | `ai_chat` | `ai.chat` |
/// | `ai_list_providers` | `ai.list_providers` |
/// | `system_health` | `system.health` |
/// | `system_ping` | `system.ping` |
/// | `system_metrics` | `system.metrics` |
/// | `system_status` | `system.status` |
/// | `capability_discover` | `capability.discover` |
/// | `capability_announce` | `capability.announce` |
/// | `discovery_peers` | `discovery.peers` |
/// | `tool_execute` | `tool.execute` |
/// | `tool_list` | `tool.list` |
/// | `context_create` | `context.create` |
/// | `context_update` | `context.update` |
/// | `context_summarize` | `context.summarize` |
/// | `lifecycle_register` | `lifecycle.register` |
/// | `lifecycle_status` | `lifecycle.status` |
#[tarpc::service]
pub trait SquirrelRpc {
    /// Query AI with a prompt
    ///
    /// Semantic: `ai.query` (JSON-RPC)
    async fn ai_query(params: QueryAiParams) -> QueryAiResult;

    /// Text completion (alias for ai.query)
    ///
    /// Semantic: `ai.complete` (JSON-RPC)
    async fn ai_complete(params: QueryAiParams) -> QueryAiResult;

    /// Chat completion with message history (alias for ai.query)
    ///
    /// Semantic: `ai.chat` (JSON-RPC)
    async fn ai_chat(params: QueryAiParams) -> QueryAiResult;

    /// List available AI providers
    ///
    /// Semantic: `ai.list_providers` (JSON-RPC)
    async fn ai_list_providers() -> ListProvidersResult;

    /// Health check
    ///
    /// Semantic: `system.health` (JSON-RPC)
    async fn system_health() -> HealthCheckResult;

    /// Ping (connectivity test)
    ///
    /// Semantic: `system.ping` (JSON-RPC)
    async fn system_ping() -> PingResult;

    /// Server metrics
    ///
    /// Semantic: `system.metrics` (JSON-RPC)
    async fn system_metrics() -> SystemMetricsResult;

    /// System status (alias for system.health)
    ///
    /// Semantic: `system.status` (JSON-RPC)
    async fn system_status() -> HealthCheckResult;

    /// Report own capabilities for socket scanning
    ///
    /// Semantic: `capability.discover` (JSON-RPC)
    async fn capability_discover() -> CapabilityDiscoverResult;

    /// Announce service capabilities
    ///
    /// Semantic: `capability.announce` (JSON-RPC)
    async fn capability_announce(params: AnnounceCapabilitiesParams) -> AnnounceCapabilitiesResult;

    /// Discover peers (other primals)
    ///
    /// Semantic: `discovery.peers` (JSON-RPC)
    async fn discovery_peers() -> DiscoveryPeersResult;

    /// Execute a tool
    ///
    /// Semantic: `tool.execute` (JSON-RPC)
    async fn tool_execute(
        tool: String,
        args: HashMap<String, serde_json::Value>,
    ) -> ToolExecuteResult;

    /// List available tools
    ///
    /// Semantic: `tool.list` (JSON-RPC)
    async fn tool_list() -> ToolListResult;

    /// Create a new context session
    ///
    /// Semantic: `context.create` (JSON-RPC)
    async fn context_create(params: ContextCreateParams) -> ContextCreateResult;

    /// Update an existing context
    ///
    /// Semantic: `context.update` (JSON-RPC)
    async fn context_update(params: ContextUpdateParams) -> ContextUpdateResult;

    /// Summarize a context session
    ///
    /// Semantic: `context.summarize` (JSON-RPC)
    async fn context_summarize(params: ContextSummarizeParams) -> ContextSummarizeResult;

    /// Register with biomeOS orchestrator
    ///
    /// Semantic: `lifecycle.register` (JSON-RPC)
    async fn lifecycle_register() -> LifecycleRegisterResult;

    /// Heartbeat status report to biomeOS
    ///
    /// Semantic: `lifecycle.status` (JSON-RPC)
    async fn lifecycle_status() -> LifecycleStatusResult;
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

        let serialized = serde_json::to_string(&params).expect("should succeed");
        let deserialized: QueryAiParams =
            serde_json::from_str(&serialized).expect("should succeed");

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

        let serialized = serde_json::to_string(&info).expect("should succeed");
        let deserialized: ProviderInfo = serde_json::from_str(&serialized).expect("should succeed");

        assert_eq!(info.id, deserialized.id);
        assert_eq!(info.online, deserialized.online);
    }
}
