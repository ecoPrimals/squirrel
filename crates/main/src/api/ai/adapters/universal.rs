// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal AI Adapter - Capability-Based Discovery
#![allow(dead_code)] // Public API surface awaiting consumer activation
//!
//! This adapter works with ANY AI provider via Unix socket communication,
//! discovered through Songbird's capability-based discovery system.
//!
//! ## TRUE PRIMAL Philosophy
//!
//! - **Zero Hardcoding**: No vendor names, discovers providers at runtime
//! - **Capability-Based**: Connects to any primal offering AI capabilities
//! - **Universal**: Works with Toadstool, NestGate, external vendors, etc.
//! - **Infant Pattern**: Only knows itself, discovers others dynamically
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! // Discover AI providers via Songbird
//! let text_gen_providers = songbird
//!     .discover_by_capability("ai:text-generation")
//!     .await?;
//!
//! // Create adapter for each discovered provider
//! for discovery in text_gen_providers {
//!     let adapter = UniversalAiAdapter::from_discovery(
//!         "ai:text-generation",
//!         discovery,
//!     ).await?;
//!     
//!     // Use the adapter
//!     let response = adapter.generate_text(request).await?;
//! }
//! ```

use super::{
    AiProviderAdapter, ImageGenerationRequest, ImageGenerationResponse, QualityTier,
    TextGenerationRequest, TextGenerationResponse,
};
use crate::error::PrimalError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// JSON-RPC request format for Unix socket communication
#[derive(Debug, Clone, Serialize)]
struct JsonRpcRequest<T> {
    jsonrpc: String,
    method: String,
    params: T,
    id: String,
}

/// JSON-RPC response format
#[derive(Debug, Clone, Deserialize)]
struct JsonRpcResponse<T> {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: String,
}

/// JSON-RPC error details
#[derive(Debug, Clone, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

/// Provider metadata from discovery
#[derive(Debug, Clone)]
pub struct ProviderMetadata {
    /// Primal ID (e.g., "toadstool", "nestgate")
    pub primal_id: String,

    /// Human-readable name
    pub name: String,

    /// Is this a local primal? (vs. external cloud service)
    pub is_local: Option<bool>,

    /// Quality tier
    pub quality: Option<String>,

    /// Cost per request (USD)
    pub cost: Option<f64>,

    /// Maximum tokens supported
    pub max_tokens: Option<u32>,

    /// Additional metadata
    pub additional: std::collections::HashMap<String, serde_json::Value>,
}

/// Universal AI Adapter
///
/// Works with any AI provider discovered via Songbird capability-based discovery.
/// Communicates via Unix sockets using JSON-RPC protocol.
pub struct UniversalAiAdapter {
    /// Unix socket path to the provider
    socket_path: PathBuf,

    /// Capability this provider offers (e.g., "ai:text-generation")
    capability: String,

    /// Provider metadata from discovery
    metadata: ProviderMetadata,

    /// Timeout for requests
    timeout: Duration,
}

impl UniversalAiAdapter {
    /// Create adapter from Songbird discovery result
    ///
    /// # Arguments
    ///
    /// * `capability` - The capability this provider offers
    /// * `socket_path` - Path to the Unix socket
    /// * `metadata` - Provider metadata
    ///
    /// # Returns
    ///
    /// New UniversalAiAdapter instance
    pub fn from_discovery(
        capability: &str,
        socket_path: PathBuf,
        metadata: ProviderMetadata,
    ) -> Self {
        Self {
            socket_path,
            capability: capability.to_string(),
            metadata,
            timeout: Duration::from_secs(120), // 2 minute timeout
        }
    }

    /// Create adapter with custom timeout
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Send JSON-RPC request via Unix socket
    async fn send_rpc_request<T, R>(&self, method: &str, params: T) -> Result<R, PrimalError>
    where
        T: Serialize,
        R: serde::de::DeserializeOwned,
    {
        let start = Instant::now();

        // Create JSON-RPC request
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: Uuid::new_v4().to_string(),
        };

        let request_json = serde_json::to_vec(&request)
            .map_err(|e| PrimalError::ParsingError(format!("Failed to serialize request: {e}")))?;

        debug!(
            "Sending RPC request to {}: method={}, size={} bytes",
            self.metadata.name,
            method,
            request_json.len()
        );

        // Connect to Unix socket
        let mut stream = tokio::time::timeout(self.timeout, UnixStream::connect(&self.socket_path))
            .await
            .map_err(|_| {
                PrimalError::NetworkError(format!(
                    "Timeout connecting to {} at {}",
                    self.metadata.name,
                    self.socket_path.display()
                ))
            })?
            .map_err(|e| {
                PrimalError::NetworkError(format!(
                    "Failed to connect to {} at {}: {}",
                    self.metadata.name,
                    self.socket_path.display(),
                    e
                ))
            })?;

        // Send request
        stream
            .write_all(&request_json)
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to send request: {e}")))?;
        stream
            .write_all(b"\n")
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to send newline: {e}")))?;
        stream
            .flush()
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to flush: {e}")))?;

        // Read response
        let mut response_buf = Vec::new();
        let bytes_read = tokio::time::timeout(self.timeout, stream.read_to_end(&mut response_buf))
            .await
            .map_err(|_| {
                PrimalError::NetworkError(format!(
                    "Timeout reading response from {}",
                    self.metadata.name
                ))
            })?
            .map_err(|e| PrimalError::NetworkError(format!("Failed to read response: {e}")))?;

        debug!(
            "Received response from {}: {} bytes in {}ms",
            self.metadata.name,
            bytes_read,
            start.elapsed().as_millis()
        );

        // Parse JSON-RPC response
        let response: JsonRpcResponse<R> = serde_json::from_slice(&response_buf).map_err(|e| {
            PrimalError::ParsingError(format!(
                "Failed to parse JSON-RPC response: {}. Response: {}",
                e,
                String::from_utf8_lossy(&response_buf)
            ))
        })?;

        // Check for RPC error
        if let Some(error) = response.error {
            return Err(PrimalError::OperationFailed(format!(
                "{} RPC error [{}]: {}",
                self.metadata.name, error.code, error.message
            )));
        }

        // Return result
        response.result.ok_or_else(|| {
            PrimalError::OperationFailed(format!(
                "{} returned neither result nor error",
                self.metadata.name
            ))
        })
    }

    /// Health check via Unix socket
    async fn check_health(&self) -> bool {
        match self
            .send_rpc_request::<(), serde_json::Value>("health", ())
            .await
        {
            Ok(_) => true,
            Err(e) => {
                warn!("Health check failed for {}: {}", self.metadata.name, e);
                false
            }
        }
    }
}

#[async_trait]
impl AiProviderAdapter for UniversalAiAdapter {
    fn provider_id(&self) -> &str {
        &self.metadata.primal_id
    }

    fn provider_name(&self) -> &str {
        &self.metadata.name
    }

    fn is_local(&self) -> bool {
        self.metadata.is_local.unwrap_or(true) // Default to local for primals
    }

    fn cost_per_unit(&self) -> Option<f64> {
        self.metadata.cost
    }

    fn avg_latency_ms(&self) -> u64 {
        // Local primals are fast, estimate 100ms
        // External would be slower, estimate 2000ms
        if self.is_local() { 100 } else { 2000 }
    }

    fn quality_tier(&self) -> QualityTier {
        match self.metadata.quality.as_deref() {
            Some("premium" | "high") => QualityTier::Premium,
            Some("low" | "fast") => QualityTier::Fast,
            _ => QualityTier::Standard,
        }
    }

    fn supports_text_generation(&self) -> bool {
        self.capability.contains("text-generation") || self.capability.contains("ai:text")
    }

    fn supports_image_generation(&self) -> bool {
        self.capability.contains("image-generation") || self.capability.contains("ai:image")
    }

    async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        let start = Instant::now();

        debug!(
            "Generating text with {}: prompt_len={}, max_tokens={}",
            self.metadata.name,
            request.prompt.len(),
            request.max_tokens
        );

        let response = self.send_rpc_request("ai.generate_text", request).await?;

        info!(
            "✅ Text generated by {} in {}ms",
            self.metadata.name,
            start.elapsed().as_millis()
        );

        Ok(response)
    }

    async fn generate_image(
        &self,
        request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        let start = Instant::now();

        debug!(
            "Generating image with {}: prompt_len={}, size={}",
            self.metadata.name,
            request.prompt.len(),
            request.size
        );

        let response = self.send_rpc_request("ai.generate_image", request).await?;

        info!(
            "✅ Image generated by {} in {}ms",
            self.metadata.name,
            start.elapsed().as_millis()
        );

        Ok(response)
    }

    async fn is_available(&self) -> bool {
        self.check_health().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_universal_adapter_creation() {
        let metadata = ProviderMetadata {
            primal_id: "toadstool".to_string(),
            name: "Toadstool GPU Inference".to_string(),
            is_local: Some(true),
            quality: Some("high".to_string()),
            cost: Some(0.0),
            max_tokens: Some(4096),
            additional: HashMap::new(),
        };

        let adapter = UniversalAiAdapter::from_discovery(
            "ai:text-generation",
            PathBuf::from("/run/user/1000/toadstool.sock"),
            metadata,
        );

        assert_eq!(adapter.provider_id(), "toadstool");
        assert_eq!(adapter.provider_name(), "Toadstool GPU Inference");
        assert!(adapter.is_local());
        assert_eq!(adapter.cost_per_unit(), Some(0.0));
        assert!(adapter.supports_text_generation());
        assert!(!adapter.supports_image_generation());
    }

    #[test]
    fn test_quality_tier_mapping() {
        let mut metadata = ProviderMetadata {
            primal_id: "test".to_string(),
            name: "Test".to_string(),
            is_local: Some(true),
            quality: Some("premium".to_string()),
            cost: Some(0.001),
            max_tokens: Some(2048),
            additional: HashMap::new(),
        };

        let adapter = UniversalAiAdapter::from_discovery(
            "ai:text-generation",
            PathBuf::from("/tmp/test.sock"),
            metadata.clone(),
        );

        assert_eq!(adapter.quality_tier(), QualityTier::Premium);

        // Test standard quality
        metadata.quality = Some("standard".to_string());
        let adapter = UniversalAiAdapter::from_discovery(
            "ai:text-generation",
            PathBuf::from("/tmp/test.sock"),
            metadata.clone(),
        );
        assert_eq!(adapter.quality_tier(), QualityTier::Standard);

        // Test fast quality
        metadata.quality = Some("fast".to_string());
        let adapter = UniversalAiAdapter::from_discovery(
            "ai:text-generation",
            PathBuf::from("/tmp/test.sock"),
            metadata,
        );
        assert_eq!(adapter.quality_tier(), QualityTier::Fast);
    }

    #[test]
    fn test_capability_detection() {
        let metadata = ProviderMetadata {
            primal_id: "test".to_string(),
            name: "Test".to_string(),
            is_local: Some(true),
            quality: None,
            cost: None,
            max_tokens: None,
            additional: HashMap::new(),
        };

        // Test text generation capability
        let adapter = UniversalAiAdapter::from_discovery(
            "ai:text-generation",
            PathBuf::from("/tmp/test.sock"),
            metadata.clone(),
        );
        assert!(adapter.supports_text_generation());
        assert!(!adapter.supports_image_generation());

        // Test image generation capability
        let adapter = UniversalAiAdapter::from_discovery(
            "ai:image-generation",
            PathBuf::from("/tmp/test.sock"),
            metadata,
        );
        assert!(!adapter.supports_text_generation());
        assert!(adapter.supports_image_generation());
    }

    #[test]
    fn test_custom_timeout() {
        let metadata = ProviderMetadata {
            primal_id: "test".to_string(),
            name: "Test".to_string(),
            is_local: Some(true),
            quality: None,
            cost: None,
            max_tokens: None,
            additional: HashMap::new(),
        };

        let adapter = UniversalAiAdapter::from_discovery(
            "ai:text-generation",
            PathBuf::from("/tmp/test.sock"),
            metadata,
        )
        .with_timeout(Duration::from_secs(30));

        assert_eq!(adapter.timeout, Duration::from_secs(30));
    }
}
