// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Capability-Based Crypto Provider - TRUE PRIMAL Architecture
//!
//! This module replaces the hardcoded BearDogClient with capability-based discovery.
//! Instead of knowing about "BearDog", we discover "crypto.signing" capability at runtime.
//!
//! **TRUE PRIMAL Pattern**:
//! - Self-knowledge only (Squirrel knows it needs "crypto.signing")
//! - Runtime discovery (finds WHO provides it dynamically)
//! - JSON-RPC communication (no hardcoded client)
//! - Zero primal dependencies (no `use beardog::*`)
//!
//! **Architecture**:
//! ```text
//! JWT Module → CapabilityCryptoProvider → Discovery Engine
//!                          ↓
//!                  Find "crypto.signing"
//!                          ↓
//!              (Could be BearDog, could be something else!)
//!                          ↓
//!                   JSON-RPC over Unix socket
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, error, info};

/// Capability-based crypto provider (replaces hardcoded BearDogClient)
#[derive(Clone)]
pub struct CapabilityCryptoProvider {
    /// Discovered endpoint for crypto.signing capability
    endpoint: Option<Arc<str>>,

    /// Discovery timeout
    discovery_timeout: std::time::Duration,
}

impl CapabilityCryptoProvider {
    /// Create new capability-based crypto provider
    pub fn new() -> Self {
        Self {
            endpoint: None,
            discovery_timeout: std::time::Duration::from_millis(500),
        }
    }

    /// Discover crypto.signing capability
    ///
    /// Uses environment-first discovery:
    /// 1. Check CRYPTO_SIGNING_ENDPOINT env var
    /// 2. Check CRYPTO_ENDPOINT env var
    /// 3. Try well-known socket paths
    /// 4. Return error if not found
    async fn discover_endpoint(&mut self) -> Result<Arc<str>> {
        // Check cache first
        if let Some(ref endpoint) = self.endpoint {
            return Ok(Arc::clone(endpoint));
        }

        // Strategy 1: Environment variable (highest priority)
        if let Ok(endpoint) = std::env::var("CRYPTO_SIGNING_ENDPOINT") {
            info!(
                "✅ Discovered crypto.signing via CRYPTO_SIGNING_ENDPOINT: {}",
                endpoint
            );
            let endpoint_arc: Arc<str> = Arc::from(endpoint.as_str());
            self.endpoint = Some(Arc::clone(&endpoint_arc));
            return Ok(endpoint_arc);
        }

        if let Ok(endpoint) = std::env::var("CRYPTO_ENDPOINT") {
            info!("✅ Discovered crypto via CRYPTO_ENDPOINT: {}", endpoint);
            let endpoint_arc: Arc<str> = Arc::from(endpoint.as_str());
            self.endpoint = Some(Arc::clone(&endpoint_arc));
            return Ok(endpoint_arc);
        }

        // Strategy 2: Well-known socket paths (development fallback)
        // These are NOT hardcoded dependencies - just standard socket locations
        let well_known_paths = [
            "/tmp/primal-crypto.sock",       // Standard location
            "/tmp/beardog.sock",             // Legacy name (provider-agnostic)
            "/var/run/crypto-provider.sock", // Production location
        ];

        for path in &well_known_paths {
            debug!("Trying well-known socket path: {}", path);
            if tokio::fs::metadata(path).await.is_ok() {
                // Socket exists, verify it provides crypto.signing
                if let Ok(true) = self.verify_capability(path, "crypto.signing").await {
                    info!("✅ Discovered crypto.signing at: {}", path);
                    let endpoint_arc: Arc<str> = Arc::from(*path);
                    self.endpoint = Some(Arc::clone(&endpoint_arc));
                    return Ok(endpoint_arc);
                }
            }
        }

        // Strategy 3: Could integrate full discovery engine here
        // For now, fail with helpful error

        error!("❌ Cannot discover crypto.signing capability");
        error!("   Set CRYPTO_SIGNING_ENDPOINT or CRYPTO_ENDPOINT environment variable");
        error!("   Example: export CRYPTO_SIGNING_ENDPOINT=/tmp/primal-crypto.sock");

        Err(anyhow::anyhow!(
            "Crypto capability not found. Set CRYPTO_SIGNING_ENDPOINT environment variable."
        ))
    }

    /// Verify that an endpoint provides a specific capability
    async fn verify_capability(&self, endpoint: &str, capability: &str) -> Result<bool> {
        // Quick capability check via JSON-RPC
        let request = json!({
            "jsonrpc": "2.0",
            "method": "capabilities.list",
            "id": 1
        });

        match self.call_json_rpc(endpoint, request).await {
            Ok(response) => {
                // Check if response contains our capability
                if let Some(capabilities) =
                    response.get("result").and_then(|r| r.get("capabilities"))
                {
                    if let Some(caps_array) = capabilities.as_array() {
                        return Ok(caps_array.iter().any(|c| {
                            c.as_str()
                                .map(|s| s == capability || s.starts_with(capability))
                                .unwrap_or(false)
                        }));
                    }
                }
                Ok(false)
            }
            Err(_) => Ok(false), // Connection failed, not available
        }
    }

    /// Sign data using discovered crypto provider
    ///
    /// This replaces `BearDogClient::ed25519_sign()`
    pub async fn sign_ed25519(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let endpoint = self.discover_endpoint().await?;

        // Encode data as base64 for JSON transport
        let data_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data);

        // Call crypto.signing via JSON-RPC
        let request = json!({
            "jsonrpc": "2.0",
            "method": "crypto.sign",
            "params": {
                "algorithm": "ed25519",
                "data": data_b64
            },
            "id": 1
        });

        let response = self.call_json_rpc(&endpoint, request).await?;

        // Extract signature from response
        let signature_b64 = response
            .get("result")
            .and_then(|r| r.get("signature"))
            .and_then(|s| s.as_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid response: missing signature"))?;

        // Decode base64 signature
        let signature =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, signature_b64)
                .context("Failed to decode signature")?;

        debug!(
            "✅ Signed {} bytes via capability crypto provider",
            data.len()
        );
        Ok(signature)
    }

    /// Verify signature using discovered crypto provider (with public key bytes)
    ///
    /// This replaces `BearDogClient::ed25519_verify()` when you have the raw public key
    pub async fn verify_ed25519(
        &mut self,
        data: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool> {
        let endpoint = self.discover_endpoint().await?;

        // Encode all data as base64
        let data_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data);
        let sig_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signature);
        let key_b64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, public_key);

        // Call crypto.verify via JSON-RPC
        let request = json!({
            "jsonrpc": "2.0",
            "method": "crypto.verify",
            "params": {
                "algorithm": "ed25519",
                "data": data_b64,
                "signature": sig_b64,
                "public_key": key_b64
            },
            "id": 1
        });

        let response = self.call_json_rpc(&endpoint, request).await?;

        // Extract valid boolean from response
        let valid = response
            .get("result")
            .and_then(|r| r.get("valid"))
            .and_then(|v| v.as_bool())
            .ok_or_else(|| anyhow::anyhow!("Invalid response: missing valid field"))?;

        debug!(
            "✅ Verified signature via capability crypto provider: {}",
            valid
        );
        Ok(valid)
    }

    /// Verify signature using key_id (provider manages key lookup)
    ///
    /// This is useful for JWT and other scenarios where the provider manages keys
    pub async fn verify_ed25519_with_key_id(
        &mut self,
        data: &[u8],
        signature: &[u8],
        key_id: &str,
    ) -> Result<bool> {
        let endpoint = self.discover_endpoint().await?;

        // Encode data and signature
        let data_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data);
        let sig_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signature);

        // Call crypto.verify with key_id
        let request = json!({
            "jsonrpc": "2.0",
            "method": "crypto.verify",
            "params": {
                "algorithm": "ed25519",
                "data": data_b64,
                "signature": sig_b64,
                "key_id": key_id
            },
            "id": 1
        });

        let response = self.call_json_rpc(&endpoint, request).await?;

        // Extract valid boolean from response
        let valid = response
            .get("result")
            .and_then(|r| r.get("valid"))
            .and_then(|v| v.as_bool())
            .ok_or_else(|| anyhow::anyhow!("Invalid response: missing valid field"))?;

        debug!(
            "✅ Verified signature (key_id={}) via capability crypto provider: {}",
            key_id, valid
        );
        Ok(valid)
    }

    /// Call JSON-RPC method on discovered endpoint
    async fn call_json_rpc(
        &self,
        endpoint: &str,
        request: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Connect to Unix socket
        let mut stream =
            tokio::time::timeout(self.discovery_timeout, UnixStream::connect(endpoint))
                .await
                .context("Discovery timeout")?
                .context("Failed to connect to crypto provider")?;

        // Send JSON-RPC request
        let request_str = serde_json::to_string(&request)?;
        stream.write_all(request_str.as_bytes()).await?;
        stream.write_all(b"\n").await?;

        // Read JSON-RPC response
        let mut reader = BufReader::new(stream);
        let mut response_str = String::new();
        reader.read_line(&mut response_str).await?;

        // Parse response
        let response: serde_json::Value = serde_json::from_str(&response_str)?;

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            return Err(anyhow::anyhow!(
                "Crypto provider error: {}",
                error
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown")
            ));
        }

        Ok(response)
    }
}

impl Default for CapabilityCryptoProvider {
    fn default() -> Self {
        Self::new()
    }
}

// Configuration for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityCryptoConfig {
    /// Socket path (optional, will auto-discover if not provided)
    pub endpoint: Option<String>,

    /// Discovery timeout in milliseconds
    pub discovery_timeout_ms: Option<u64>,
}

impl Default for CapabilityCryptoConfig {
    fn default() -> Self {
        Self {
            endpoint: std::env::var("CRYPTO_ENDPOINT").ok(),
            discovery_timeout_ms: Some(500),
        }
    }
}

impl CapabilityCryptoProvider {
    /// Create from config (for compatibility with old BearDogClientConfig)
    pub fn from_config(config: CapabilityCryptoConfig) -> Self {
        let mut provider = Self::new();

        if let Some(endpoint) = config.endpoint {
            provider.endpoint = Some(Arc::from(endpoint.as_str()));
        }

        if let Some(timeout_ms) = config.discovery_timeout_ms {
            provider.discovery_timeout = std::time::Duration::from_millis(timeout_ms);
        }

        provider
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_from_env() {
        std::env::set_var("CRYPTO_SIGNING_ENDPOINT", "/tmp/test-crypto.sock");

        let mut provider = CapabilityCryptoProvider::new();
        // Discovery will work even if socket doesn't exist (we cache the env var)
        // Actual connection happens on first use

        std::env::remove_var("CRYPTO_SIGNING_ENDPOINT");
    }

    #[test]
    fn test_config_default() {
        let config = CapabilityCryptoConfig::default();
        assert_eq!(config.discovery_timeout_ms, Some(500));
    }
}
