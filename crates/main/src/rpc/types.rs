// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON-RPC Request/Response Types
//!
//! These types define the API contract between biomeOS and Squirrel.
//! All types are JSON-RPC 2.0 compliant.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Request to query AI with a prompt
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryAiRequest {
    /// The prompt to send to the AI
    pub prompt: String,

    /// Provider hint ("auto" for capability-based selection, or a provider ID from discovery)
    pub provider: Option<String>,

    /// Model to use (optional, provider-specific)
    pub model: Option<String>,

    /// Task priority (0-100, default 50)
    pub priority: Option<u8>,

    /// Maximum tokens in response
    pub max_tokens: Option<usize>,

    /// Temperature (0.0-2.0)
    pub temperature: Option<f32>,

    /// Whether to stream the response
    pub stream: Option<bool>,
}

/// Response from AI query
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryAiResponse {
    /// The AI's response text
    pub response: String,

    /// Provider that handled the request
    pub provider: String,

    /// Model that was used
    pub model: String,

    /// Tokens used in the request
    pub tokens_used: Option<usize>,

    /// Latency in milliseconds
    pub latency_ms: u64,

    /// Whether the request was successful
    pub success: bool,
}

/// Request to list available AI providers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListProvidersRequest {
    /// Filter by capability (optional)
    pub capability: Option<String>,

    /// Include offline providers
    pub include_offline: Option<bool>,
}

/// Information about an AI provider
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

    /// Average latency in milliseconds
    pub avg_latency_ms: Option<u64>,

    /// Cost tier
    pub cost_tier: String,
}

/// Response with list of providers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListProvidersResponse {
    /// Available providers
    pub providers: Vec<ProviderInfo>,

    /// Total count
    pub total: usize,
}

/// Request to announce a primal's capabilities and tools.
///
/// Extended to support the neuralSpring adapter pattern where remote
/// primals register their socket path and tools for routing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnounceCapabilitiesRequest {
    /// Capability namespaces to announce (e.g. `["science.physics", "science.chem"]`)
    pub capabilities: Vec<String>,

    /// Announcing primal's name (optional for backward compat, required for routing)
    #[serde(default)]
    pub primal: Option<String>,

    /// Unix socket path where the announcing primal listens (for forwarding)
    #[serde(default)]
    pub socket_path: Option<String>,

    /// Tool names the primal provides (e.g. `["science.simulate", "science.analyze"]`)
    #[serde(default)]
    pub tools: Option<Vec<String>>,

    /// Sub-federations (optional)
    pub sub_federations: Option<Vec<String>>,

    /// Genetic families (optional)
    pub genetic_families: Option<Vec<String>>,
}

/// Response from capability announcement
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnounceCapabilitiesResponse {
    /// Whether the announcement was successful
    pub success: bool,

    /// Confirmation message
    pub message: String,

    /// Timestamp of announcement
    pub announced_at: String,

    /// Number of tools registered for routing
    pub tools_registered: usize,
}

/// An announced remote primal, stored for tool routing.
/// Uses `Arc<str>` for identifiers to avoid cloning on hot paths.
#[derive(Debug, Clone)]
pub struct AnnouncedPrimal {
    /// Primal name (e.g. `"neuralSpring"`)
    pub primal: Arc<str>,
    /// Socket path for forwarding
    pub socket_path: Arc<str>,
    /// Capability namespaces
    pub capabilities: Vec<Arc<str>>,
    /// Tool names this primal serves
    pub tools: Vec<Arc<str>>,
    /// When the announcement was received
    pub announced_at: chrono::DateTime<chrono::Utc>,
}

/// Response from tool.list
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolListResponse {
    /// All available tools (local + remote)
    pub tools: Vec<ToolListEntry>,
    /// Total count
    pub total: usize,
}

/// A single tool in the tool.list response (McpToolDef pattern from neuralSpring)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolListEntry {
    /// Tool name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Domain (e.g. "system", "discovery", "science")
    pub domain: String,
    /// Whether the tool is built-in or announced by a remote primal
    pub source: ToolSource,
    /// JSON Schema for input parameters (neuralSpring McpToolDef pattern)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
}

/// Where a tool comes from
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolSource {
    /// Built into squirrel
    Builtin,
    /// Announced by a remote primal
    Remote {
        /// ID of the remote primal that announced the tool.
        primal: String,
    },
}

/// Request for health check
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthCheckRequest {}

/// Health status information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// Overall status
    pub status: String,

    /// Squirrel version
    pub version: String,

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Number of active AI providers
    pub active_providers: usize,

    /// Number of requests processed
    pub requests_processed: u64,

    /// Average response time in milliseconds
    pub avg_response_time_ms: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_ai_request_serialization() {
        let request = QueryAiRequest {
            prompt: "Test prompt".to_string(),
            provider: Some("auto".to_string()),
            model: None,
            priority: Some(50),
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: Some(false),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: QueryAiRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.prompt, deserialized.prompt);
        assert_eq!(request.provider, deserialized.provider);
        assert_eq!(request.priority, deserialized.priority);
    }

    #[test]
    fn test_list_providers_request_serialization() {
        let request = ListProvidersRequest {
            capability: Some("ai.inference".to_string()),
            include_offline: Some(false),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ListProvidersRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.capability, deserialized.capability);
        assert_eq!(request.include_offline, deserialized.include_offline);
    }

    #[test]
    fn test_health_check_response_serialization() {
        let response = HealthCheckResponse {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            uptime_seconds: 3600,
            active_providers: 3,
            requests_processed: 1000,
            avg_response_time_ms: Some(150.5),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: HealthCheckResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.status, deserialized.status);
        assert_eq!(response.uptime_seconds, deserialized.uptime_seconds);
    }

    #[test]
    fn test_query_ai_response_serialization() {
        let response = QueryAiResponse {
            response: "Hello, world!".to_string(),
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            tokens_used: Some(42),
            latency_ms: 150,
            success: true,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: QueryAiResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.response, "Hello, world!");
        assert_eq!(deserialized.provider, "openai");
        assert!(deserialized.success);
    }

    #[test]
    fn test_provider_info_serialization() {
        let info = ProviderInfo {
            id: "openai-1".to_string(),
            name: "OpenAI".to_string(),
            models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
            capabilities: vec!["chat".to_string(), "completion".to_string()],
            online: true,
            avg_latency_ms: Some(150),
            cost_tier: "premium".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: ProviderInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "openai-1");
        assert_eq!(deserialized.models.len(), 2);
        assert!(deserialized.online);
    }

    #[test]
    fn test_list_providers_response_serialization() {
        let response = ListProvidersResponse {
            providers: vec![ProviderInfo {
                id: "test".to_string(),
                name: "Test".to_string(),
                models: vec![],
                capabilities: vec![],
                online: false,
                avg_latency_ms: None,
                cost_tier: "free".to_string(),
            }],
            total: 1,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ListProvidersResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total, 1);
        assert_eq!(deserialized.providers.len(), 1);
    }

    #[test]
    fn test_announce_capabilities_request_serialization() {
        let request = AnnounceCapabilitiesRequest {
            capabilities: vec!["ai.inference".to_string(), "ai.embedding".to_string()],
            primal: Some("neuralSpring".to_string()),
            socket_path: Some("/tmp/biomeos/neural.sock".to_string()),
            tools: Some(vec!["science.simulate".to_string()]),
            sub_federations: Some(vec!["federation-1".to_string()]),
            genetic_families: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: AnnounceCapabilitiesRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.capabilities.len(), 2);
        assert!(deserialized.sub_federations.is_some());
        assert!(deserialized.genetic_families.is_none());
    }

    #[test]
    fn test_announce_capabilities_response_serialization() {
        let response = AnnounceCapabilitiesResponse {
            success: true,
            message: "Capabilities registered".to_string(),
            announced_at: "2026-01-01T00:00:00Z".to_string(),
            tools_registered: 3,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: AnnounceCapabilitiesResponse = serde_json::from_str(&json).unwrap();
        assert!(deserialized.success);
        assert!(!deserialized.announced_at.is_empty());
    }

    #[test]
    fn test_health_check_request_serialization() {
        let request = HealthCheckRequest {};
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: HealthCheckRequest = serde_json::from_str(&json).unwrap();
        let _ = format!("{deserialized:?}");
    }

    #[test]
    fn test_query_ai_request_minimal() {
        let request = QueryAiRequest {
            prompt: "Hello".to_string(),
            provider: None,
            model: None,
            priority: None,
            max_tokens: None,
            temperature: None,
            stream: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: QueryAiRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.prompt, "Hello");
        assert!(deserialized.provider.is_none());
    }
}
