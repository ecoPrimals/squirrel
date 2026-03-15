// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Security Provider Client for Crypto Operations (Pure Rust!)
//!
//! This module provides a Unix socket client for delegating cryptographic
//! operations to the security provider primal discovered via capability.
//!
//! **Architecture**: TRUE PRIMAL principle - delegate crypto to the specialist!
//! - Squirrel = AI/MCP specialist
//! - Security provider primal = Crypto specialist (discovered at runtime)
//! - Communication = Unix sockets (Zero-HTTP!)
//!
//! **Capability-Based Discovery**: Auth discovers the security provider via
//! capability (e.g., "crypto.signing"), not by hardcoded primal name.
//!
//! **API**: JSON-RPC 2.0 over Unix sockets
//! **Crypto**: Ed25519 (EdDSA) for JWT signing/verification
//! **Pure Rust**: 100% (no ring, no C deps!)

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};
use tracing::{debug, info, warn};

/// Security provider client configuration
#[derive(Debug, Clone)]
pub struct BearDogClientConfig {
    /// Path to security provider's crypto Unix socket
    pub socket_path: String,

    /// Timeout for socket operations (default: 5 seconds)
    pub timeout_secs: u64,

    /// Number of connection retries (default: 3)
    pub max_retries: usize,

    /// Delay between retries in milliseconds (default: 100ms)
    pub retry_delay_ms: u64,
}

impl Default for BearDogClientConfig {
    fn default() -> Self {
        Self {
            socket_path: "/var/run/beardog/crypto.sock".to_string(),
            timeout_secs: 5,
            max_retries: 3,
            retry_delay_ms: 100,
        }
    }
}

/// Security provider client for delegating crypto operations
///
/// # Examples
///
/// ```no_run
/// use squirrel_mcp_auth::security_provider_client::{BearDogClient, BearDogClientConfig};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let config = BearDogClientConfig::default();
///     let client = BearDogClient::new(config)?;
///
///     let data = b"Hello, ecoPrimals!";
///     let signature = client.ed25519_sign(data, "my-key-id").await?;
///     let is_valid = client.ed25519_verify(data, &signature, "my-key-id").await?;
///
///     assert!(is_valid);
///     Ok(())
/// }
/// ```
pub struct BearDogClient {
    config: BearDogClientConfig,
    request_id: std::sync::atomic::AtomicU64,
}

impl BearDogClient {
    /// Create a new security provider client
    pub fn new(config: BearDogClientConfig) -> Result<Self> {
        // Validate socket path exists (in production)
        if !Path::new(&config.socket_path).exists() {
            warn!(
                "Security provider socket not found at: {} (may not be running yet)",
                config.socket_path
            );
        }

        info!(
            "Security provider client configured: {}",
            config.socket_path
        );

        Ok(Self {
            config,
            request_id: std::sync::atomic::AtomicU64::new(1),
        })
    }

    /// Sign data using Ed25519 (via security provider)
    ///
    /// # Arguments
    /// * `data` - Data to sign
    /// * `key_id` - Key ID to use for signing
    ///
    /// # Returns
    /// Raw Ed25519 signature (64 bytes)
    pub async fn ed25519_sign(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>> {
        debug!(
            "Ed25519 sign request: key_id={}, data_len={}",
            key_id,
            data.len()
        );

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_request_id(),
            method: "crypto.ed25519.sign".to_string(),
            params: serde_json::json!({
                "data": BASE64.encode(data),
                "key_id": key_id
            }),
        };

        let response = self.send_request(request).await?;

        let signature_b64 = response["result"]["signature"]
            .as_str()
            .context("Missing 'signature' in security provider response")?;

        let signature = BASE64
            .decode(signature_b64)
            .context("Failed to decode signature from security provider")?;

        debug!("Ed25519 sign successful: signature_len={}", signature.len());

        Ok(signature)
    }

    /// Verify Ed25519 signature (via security provider)
    ///
    /// # Arguments
    /// * `data` - Original data that was signed
    /// * `signature` - Ed25519 signature to verify (64 bytes)
    /// * `key_id` - Key ID to use for verification
    ///
    /// # Returns
    /// `true` if signature is valid, `false` otherwise
    pub async fn ed25519_verify(
        &self,
        data: &[u8],
        signature: &[u8],
        key_id: &str,
    ) -> Result<bool> {
        debug!(
            "Ed25519 verify request: key_id={}, data_len={}, sig_len={}",
            key_id,
            data.len(),
            signature.len()
        );

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_request_id(),
            method: "crypto.ed25519.verify".to_string(),
            params: serde_json::json!({
                "data": BASE64.encode(data),
                "signature": BASE64.encode(signature),
                "key_id": key_id
            }),
        };

        let response = self.send_request(request).await?;

        let valid = response["result"]["valid"]
            .as_bool()
            .context("Missing 'valid' field in security provider response")?;

        debug!("Ed25519 verify result: valid={}", valid);

        Ok(valid)
    }

    /// Send JSON-RPC request to security provider with retry logic
    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonValue> {
        let mut last_error = None;

        for attempt in 1..=self.config.max_retries {
            match self.send_request_once(&request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    warn!(
                        "Security provider request failed (attempt {}/{}): {}",
                        attempt, self.config.max_retries, e
                    );
                    last_error = Some(e);

                    if attempt < self.config.max_retries {
                        tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!(
                "Security provider request failed after {} retries",
                self.config.max_retries
            )
        }))
    }

    /// Send a single JSON-RPC request to security provider
    async fn send_request_once(&self, request: &JsonRpcRequest) -> Result<JsonValue> {
        // Connect to Unix socket with timeout
        let stream = timeout(
            Duration::from_secs(self.config.timeout_secs),
            UnixStream::connect(&self.config.socket_path),
        )
        .await
        .context("Timeout connecting to security provider socket")?
        .context("Failed to connect to security provider socket")?;

        let (read_half, mut write_half) = stream.into_split();

        // Serialize request
        let request_json =
            serde_json::to_string(&request).context("Failed to serialize JSON-RPC request")?;

        // Send request (with newline delimiter)
        timeout(Duration::from_secs(self.config.timeout_secs), async {
            write_half.write_all(request_json.as_bytes()).await?;
            write_half.write_all(b"\n").await?;
            write_half.flush().await?;
            Ok::<(), std::io::Error>(())
        })
        .await
        .context("Timeout sending request to security provider")?
        .context("Failed to send request to security provider")?;

        // Read response (newline-delimited JSON)
        let mut reader = BufReader::new(read_half);
        let mut response_line = String::new();

        timeout(
            Duration::from_secs(self.config.timeout_secs),
            reader.read_line(&mut response_line),
        )
        .await
        .context("Timeout reading response from security provider")?
        .context("Failed to read response from security provider")?;

        // Parse response
        let response: JsonRpcResponse = serde_json::from_str(&response_line)
            .context("Failed to parse JSON-RPC response from security provider")?;

        // Check for JSON-RPC error
        if let Some(error) = response.error {
            return Err(anyhow::anyhow!(
                "Security provider error: {} (code: {})",
                error.message,
                error.code
            ));
        }

        response
            .result
            .context("Security provider response missing 'result' field")
    }

    /// Get next request ID (monotonically increasing)
    fn next_request_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: JsonValue,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[allow(dead_code)]
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<JsonValue>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beardog_client_creation() {
        let config = BearDogClientConfig::default();
        let client = BearDogClient::new(config);

        assert!(client.is_ok());
    }

    #[test]
    fn test_beardog_client_custom_config() {
        let config = BearDogClientConfig {
            socket_path: "/tmp/test-beardog.sock".to_string(),
            timeout_secs: 10,
            max_retries: 5,
            retry_delay_ms: 200,
        };

        let client = BearDogClient::new(config).unwrap();
        assert_eq!(client.config.socket_path, "/tmp/test-beardog.sock");
        assert_eq!(client.config.timeout_secs, 10);
        assert_eq!(client.config.max_retries, 5);
        assert_eq!(client.config.retry_delay_ms, 200);
    }

    #[test]
    fn test_request_id_increments() {
        let config = BearDogClientConfig::default();
        let client = BearDogClient::new(config).unwrap();

        let id1 = client.next_request_id();
        let id2 = client.next_request_id();
        let id3 = client.next_request_id();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    // Integration tests (require security provider running) are in integration tests
}
