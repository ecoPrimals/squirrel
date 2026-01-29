//! Universal AI Adapter - Works with ANY AI Provider
//!
//! This adapter implements the `AiCapability` trait for any provider
//! that exposes AI capabilities via JSON-RPC over Unix sockets.
//!
//! TRUE PRIMAL: Zero vendor-specific code, pure capability-based.

use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::api::ai::universal::{
    AiCapability, CostTier, ProviderMetadata, ProviderType, TokenUsage, UniversalAiRequest,
    UniversalAiResponse,
};
use crate::capabilities::discovery::CapabilityProvider;
use crate::error::PrimalError;

/// Universal AI Adapter
///
/// This adapter can work with ANY AI provider that:
/// 1. Exposes a Unix socket
/// 2. Implements JSON-RPC 2.0
/// 3. Provides an AI capability (ai.complete, ai.chat, etc.)
///
/// No vendor-specific code. No hardcoded endpoints. Pure capability-based.
#[derive(Debug, Clone)]
pub struct UniversalAiAdapter {
    /// Socket path to the provider
    socket: PathBuf,

    /// Provider unique identifier
    provider_id: String,

    /// Capability this adapter uses (e.g., "ai.complete")
    capability: String,

    /// Provider metadata
    metadata: ProviderMetadata,
}

impl UniversalAiAdapter {
    /// Create adapter from a capability provider
    ///
    /// This is the primary constructor - it takes the output of capability
    /// discovery and creates an adapter ready to use.
    pub async fn from_capability_provider(
        provider: CapabilityProvider,
        capability: String,
    ) -> Result<Self, PrimalError> {
        // Extract metadata from provider
        let metadata = Self::extract_metadata(&provider, &capability);

        Ok(Self {
            socket: provider.socket,
            provider_id: provider.id,
            capability,
            metadata,
        })
    }

    /// Extract metadata from capability provider
    fn extract_metadata(provider: &CapabilityProvider, _capability: &str) -> ProviderMetadata {
        // Try to extract provider type from metadata
        let provider_type = provider
            .metadata
            .get("provider_type")
            .map(|s| match s.as_str() {
                "local" => ProviderType::Local,
                "cloud" => ProviderType::Cloud,
                "custom" => ProviderType::Custom,
                _ => ProviderType::Custom,
            })
            .unwrap_or(ProviderType::Custom);

        // Try to extract models from metadata
        // Metadata is String values, so we'll try to parse as JSON if it looks like an array
        let models = provider
            .metadata
            .get("models")
            .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
            .unwrap_or_default();

        // Extract cost tier if available
        let cost_tier = provider
            .metadata
            .get("cost_tier")
            .and_then(|s| match s.as_str() {
                "free" => Some(CostTier::Free),
                "low" => Some(CostTier::Low),
                "medium" => Some(CostTier::Medium),
                "high" => Some(CostTier::High),
                _ => None,
            });

        ProviderMetadata {
            name: provider.id.clone(),
            provider_type,
            models,
            capabilities: provider
                .capabilities
                .iter()
                .map(|s| s.to_string())
                .collect(),
            avg_latency_ms: None,
            cost_tier,
            extra: provider
                .metadata
                .iter()
                .map(|(k, v)| (k.clone(), json!(v)))
                .collect(),
        }
    }

    /// Call provider via JSON-RPC
    async fn call_provider(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        // Connect to provider
        let stream = UnixStream::connect(&self.socket).await.map_err(|e| {
            PrimalError::NetworkError(format!(
                "Failed to connect to provider at {:?}: {}",
                self.socket, e
            ))
        })?;

        // Build JSON-RPC request
        let rpc_request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": Uuid::new_v4().to_string(),
        });

        // Serialize and send
        let mut request_str = serde_json::to_string(&rpc_request)?;
        request_str.push('\n');

        let (read_half, mut write_half) = stream.into_split();
        write_half
            .write_all(request_str.as_bytes())
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Write error: {}", e)))?;

        // Read response
        let mut reader = BufReader::new(read_half);
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Read error: {}", e)))?;

        // Parse JSON-RPC response
        let rpc_response: serde_json::Value = serde_json::from_str(&response_line)?;

        // Check for error
        if let Some(error) = rpc_response.get("error") {
            return Err(PrimalError::NetworkError(format!(
                "Provider error: {}",
                error
            )));
        }

        // Extract result
        rpc_response
            .get("result")
            .cloned()
            .ok_or_else(|| PrimalError::ParsingError("No result in response".to_string()))
    }
}

#[async_trait]
impl AiCapability for UniversalAiAdapter {
    async fn complete(
        &self,
        request: UniversalAiRequest,
    ) -> Result<UniversalAiResponse, PrimalError> {
        let start = Instant::now();

        // Build params from universal request
        let mut params = json!({});

        if let Some(prompt) = &request.prompt {
            params["prompt"] = json!(prompt);
        }

        if let Some(messages) = &request.messages {
            params["messages"] = json!(messages);
        }

        if let Some(max_tokens) = request.max_tokens {
            params["max_tokens"] = json!(max_tokens);
        }

        if let Some(temperature) = request.temperature {
            params["temperature"] = json!(temperature);
        }

        if let Some(top_p) = request.top_p {
            params["top_p"] = json!(top_p);
        }

        if let Some(model) = &request.model {
            params["model"] = json!(model);
        }

        if let Some(stop) = &request.stop {
            params["stop"] = json!(stop);
        }

        if request.stream {
            params["stream"] = json!(true);
        }

        // Add metadata if any
        for (key, value) in &request.metadata {
            params[key] = value.clone();
        }

        // Call provider
        debug!(
            "Calling provider '{}' with method '{}'",
            self.provider_id, self.capability
        );
        let result = self.call_provider(&self.capability, params).await?;

        let elapsed = start.elapsed();

        // Parse universal response
        let text = result
            .get("text")
            .or_else(|| result.get("content"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| PrimalError::ParsingError("No text in response".to_string()))?
            .to_string();

        let model = result
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        // Parse usage if available
        let usage = result.get("usage").and_then(|u| {
            Some(TokenUsage {
                prompt_tokens: u.get("prompt_tokens")?.as_u64()? as u32,
                completion_tokens: u.get("completion_tokens")?.as_u64()? as u32,
                total_tokens: u.get("total_tokens")?.as_u64()? as u32,
            })
        });

        let stop_reason = result
            .get("stop_reason")
            .or_else(|| result.get("finish_reason"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let cost_usd = result.get("cost_usd").and_then(|v| v.as_f64());

        Ok(UniversalAiResponse {
            text,
            provider_id: self.provider_id.clone(),
            model,
            usage,
            stop_reason,
            latency_ms: Some(elapsed.as_millis() as u64),
            cost_usd,
            metadata: HashMap::new(),
        })
    }

    async fn is_available(&self) -> bool {
        // Try to connect to socket
        match UnixStream::connect(&self.socket).await {
            Ok(_) => true,
            Err(e) => {
                debug!("Provider '{}' not available: {}", self.provider_id, e);
                false
            }
        }
    }

    fn capabilities(&self) -> Vec<String> {
        self.metadata.capabilities.clone()
    }

    fn metadata(&self) -> ProviderMetadata {
        self.metadata.clone()
    }

    fn provider_id(&self) -> &str {
        &self.provider_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_extract_metadata() {
        let mut metadata_map = HashMap::new();
        metadata_map.insert("provider_type".to_string(), "local".to_string());
        metadata_map.insert(
            "models".to_string(),
            r#"["model-1", "model-2"]"#.to_string(),
        );
        metadata_map.insert("cost_tier".to_string(), "free".to_string());

        let provider = CapabilityProvider {
            id: "test-provider".to_string(),
            capabilities: vec!["ai.complete".to_string()],
            socket: PathBuf::from("/tmp/test.sock"),
            metadata: metadata_map,
            discovered_via: "test".to_string(),
        };

        let metadata = UniversalAiAdapter::extract_metadata(&provider, "ai.complete");

        assert_eq!(metadata.name, "test-provider");
        assert_eq!(metadata.provider_type, ProviderType::Local);
        assert_eq!(metadata.models.len(), 2);
        assert_eq!(metadata.cost_tier, Some(CostTier::Free));
    }

    #[tokio::test]
    async fn test_adapter_is_available_no_socket() {
        let provider = CapabilityProvider {
            id: "test-provider".to_string(),
            capabilities: vec!["ai.complete".to_string()],
            socket: PathBuf::from("/tmp/nonexistent-test.sock"),
            metadata: HashMap::new(),
            discovered_via: "test".to_string(),
        };

        let adapter =
            UniversalAiAdapter::from_capability_provider(provider, "ai.complete".to_string())
                .await
                .unwrap();

        // Should return false, not panic
        let available = adapter.is_available().await;
        assert!(!available);
    }
}
