// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Capability-Based Crypto Provider - TRUE PRIMAL Architecture
//!
//! This module replaces the hardcoded `BearDogClient` with capability-based discovery.
//! Instead of knowing about "`BearDog`", we discover "crypto.signing" capability at runtime.
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
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, error, info};
use universal_constants::network::resolve_capability_unix_socket;

/// Primary env for tiered resolution of the `crypto.sign` / signing socket (see [`resolve_capability_unix_socket`]).
const CRYPTO_SIGN_CAPABILITY_SOCKET_ENV: &str = "CRYPTO_CAPABILITY_SOCKET";
/// Basename stem for `BearDog` under `$XDG_RUNTIME_DIR/biomeos/` (e.g. `beardog.sock`).
const BEARDOG_BIOMEOS_SOCKET_STEM: &str = "beardog";

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|p| p == &path) {
        paths.push(path);
    }
}

/// Candidate Unix socket paths for crypto signing (security primal / `BearDog`), ordered by precedence.
///
/// Order:
/// 1. `SECURITY_SOCKET` — primary override for the security primal socket
/// 2. `BEARDOG_SOCKET` — legacy compat
/// 3. [`resolve_capability_unix_socket`] for capability `crypto.sign` (tiered env + standard `XDG_RUNTIME_DIR/biomeos/beardog.sock`)
/// 4. `/tmp/beardog.sock` — last resort
#[must_use]
fn candidate_crypto_signing_socket_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(p) = std::env::var("SECURITY_SOCKET")
        && !p.is_empty()
    {
        push_unique_path(&mut paths, PathBuf::from(p));
    }
    if let Ok(p) = std::env::var("BEARDOG_SOCKET")
        && !p.is_empty()
    {
        push_unique_path(&mut paths, PathBuf::from(p));
    }

    push_unique_path(
        &mut paths,
        resolve_capability_unix_socket(
            CRYPTO_SIGN_CAPABILITY_SOCKET_ENV,
            BEARDOG_BIOMEOS_SOCKET_STEM,
        ),
    );
    push_unique_path(&mut paths, PathBuf::from("/tmp/beardog.sock"));

    paths
}

/// Capability-based crypto provider (replaces hardcoded `BearDogClient`)
#[derive(Clone)]
pub struct CapabilityCryptoProvider {
    /// Discovered endpoint for crypto.signing capability
    endpoint: Option<Arc<str>>,

    /// Discovery timeout
    discovery_timeout: std::time::Duration,
}

impl CapabilityCryptoProvider {
    /// Create new capability-based crypto provider
    #[must_use]
    pub const fn new() -> Self {
        Self {
            endpoint: None,
            discovery_timeout: std::time::Duration::from_millis(500),
        }
    }

    /// Discover crypto.signing capability
    ///
    /// Uses environment-first discovery:
    /// 1. Check `CRYPTO_SIGNING_ENDPOINT` env var
    /// 2. Check `CRYPTO_ENDPOINT` env var
    /// 3. Try [`candidate_crypto_signing_socket_paths`] (`SECURITY_SOCKET`, `BEARDOG_SOCKET`,
    ///    [`resolve_capability_unix_socket`] for `crypto.sign` / `beardog.sock`, then `/tmp/beardog.sock`)
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

        // Strategy 2: capability-based paths (SECURITY_SOCKET, BEARDOG_SOCKET,
        // universal-constants tiered resolution for crypto.sign → `/tmp/beardog.sock` last)
        for path in candidate_crypto_signing_socket_paths() {
            let path_str = path.to_string_lossy();
            debug!("Trying candidate crypto signing socket path: {}", path_str);
            if tokio::fs::metadata(&path).await.is_ok() {
                // Socket exists, verify it provides crypto.signing
                if matches!(
                    self.verify_capability(&path_str, "crypto.signing").await,
                    Ok(true)
                ) {
                    info!("✅ Discovered crypto.signing at: {}", path_str);
                    let endpoint_arc: Arc<str> = path_str.as_ref().into();
                    self.endpoint = Some(Arc::clone(&endpoint_arc));
                    return Ok(endpoint_arc);
                }
            }
        }

        // Strategy 3: Could integrate full discovery engine here
        // For now, fail with helpful error

        error!("❌ Cannot discover crypto.signing capability");
        error!(
            "   Set CRYPTO_SIGNING_ENDPOINT, CRYPTO_ENDPOINT, SECURITY_SOCKET, or BEARDOG_SOCKET"
        );
        error!("   Example: export SECURITY_SOCKET=\"$XDG_RUNTIME_DIR/biomeos/beardog.sock\"");

        Err(anyhow::anyhow!(
            "Crypto capability not found. Set CRYPTO_SIGNING_ENDPOINT, CRYPTO_ENDPOINT, or SECURITY_SOCKET."
        ))
    }

    /// Verify that an endpoint provides a specific capability
    async fn verify_capability(&self, endpoint: &str, capability: &str) -> Result<bool> {
        // Quick capability check via JSON-RPC
        let request = json!({
            "jsonrpc": "2.0",
            "method": "capability.list",
            "id": 1
        });

        match self.call_json_rpc(endpoint, request).await {
            Ok(response) => {
                // Check if response contains our capability
                if let Some(capabilities) =
                    response.get("result").and_then(|r| r.get("capabilities"))
                    && let Some(caps_array) = capabilities.as_array()
                {
                    return Ok(caps_array.iter().any(|c| {
                        c.as_str()
                            .is_some_and(|s| s == capability || s.starts_with(capability))
                    }));
                }
                Ok(false)
            }
            Err(_) => Ok(false), // Connection failed, not available
        }
    }

    /// Sign data using discovered crypto provider
    ///
    /// This replaces `BearDogClient::ed25519_sign()`
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if endpoint discovery, JSON-RPC, or signature decoding fails.
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
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if endpoint discovery, JSON-RPC, or response parsing fails.
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
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| anyhow::anyhow!("Invalid response: missing valid field"))?;

        debug!(
            "✅ Verified signature via capability crypto provider: {}",
            valid
        );
        Ok(valid)
    }

    /// Verify signature using `key_id` (provider manages key lookup)
    ///
    /// This is useful for JWT and other scenarios where the provider manages keys
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if endpoint discovery, JSON-RPC, or response parsing fails.
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
            .and_then(serde_json::Value::as_bool)
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

/// Configuration for capability-based crypto provider (backward compatibility).
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
    /// Create from config (for compatibility with old `BearDogClientConfig`)
    #[must_use]
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
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    #[test]
    fn test_provider_new() {
        let _provider = CapabilityCryptoProvider::new();
    }

    #[test]
    fn test_provider_default() {
        let _provider = CapabilityCryptoProvider::default();
    }

    #[test]
    fn test_config_default() {
        let config = CapabilityCryptoConfig::default();
        assert_eq!(config.discovery_timeout_ms, Some(500));
    }

    #[test]
    fn test_config_serialize_deserialize() {
        let config = CapabilityCryptoConfig {
            endpoint: Some("/tmp/crypto.sock".to_string()),
            discovery_timeout_ms: Some(1000),
        };
        let json = serde_json::to_string(&config).expect("should succeed");
        let restored: CapabilityCryptoConfig = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(restored.endpoint, config.endpoint);
        assert_eq!(restored.discovery_timeout_ms, config.discovery_timeout_ms);
    }

    #[test]
    fn test_from_config_with_endpoint() {
        let config = CapabilityCryptoConfig {
            endpoint: Some("/run/user/1000/crypto.sock".to_string()),
            discovery_timeout_ms: None,
        };
        let _provider = CapabilityCryptoProvider::from_config(config);
    }

    #[test]
    fn test_from_config_with_timeout() {
        let config = CapabilityCryptoConfig {
            endpoint: None,
            discovery_timeout_ms: Some(2000),
        };
        let _provider = CapabilityCryptoProvider::from_config(config);
    }

    #[test]
    fn test_from_config_full() {
        let config = CapabilityCryptoConfig {
            endpoint: Some("/var/run/crypto.sock".to_string()),
            discovery_timeout_ms: Some(1500),
        };
        let _provider = CapabilityCryptoProvider::from_config(config);
    }

    #[test]
    fn test_discovery_from_env() {
        temp_env::with_var(
            "CRYPTO_SIGNING_ENDPOINT",
            Some("/tmp/test-crypto.sock"),
            || {
                let rt = tokio::runtime::Runtime::new().expect("should succeed");
                rt.block_on(async {
                    let mut provider = CapabilityCryptoProvider::new();
                    let result = provider.sign_ed25519(b"test").await;
                    assert!(result.is_err());
                    let err_str = result.unwrap_err().to_string();
                    assert!(
                        err_str.contains("connect")
                            || err_str.contains("timeout")
                            || err_str.contains("Crypto")
                            || err_str.contains("Failed")
                    );
                });
            },
        );
    }

    #[test]
    fn test_discovery_from_crypto_endpoint_env() {
        temp_env::with_vars(
            [
                ("CRYPTO_SIGNING_ENDPOINT", None::<&str>),
                ("CRYPTO_ENDPOINT", Some("/tmp/crypto-alt.sock")),
            ],
            || {
                let rt = tokio::runtime::Runtime::new().expect("should succeed");
                rt.block_on(async {
                    let mut provider = CapabilityCryptoProvider::new();
                    let result = provider.sign_ed25519(b"test").await;
                    assert!(result.is_err());
                });
            },
        );
    }

    #[test]
    fn test_sign_ed25519_no_endpoint_fails() {
        temp_env::with_vars(
            [
                ("CRYPTO_SIGNING_ENDPOINT", None::<&str>),
                ("CRYPTO_ENDPOINT", None::<&str>),
            ],
            || {
                let rt = tokio::runtime::Runtime::new().expect("should succeed");
                rt.block_on(async {
                    let mut provider = CapabilityCryptoProvider::new();
                    let result = provider.sign_ed25519(b"data").await;
                    assert!(result.is_err());
                    let err_str = result.unwrap_err().to_string();
                    assert!(
                        err_str.contains("Crypto capability not found")
                            || err_str.contains("CRYPTO")
                    );
                });
            },
        );
    }

    #[test]
    fn test_mock_socket_sign_ed25519() {
        let rt = tokio::runtime::Runtime::new().expect("should succeed");
        let dir = tempfile::tempdir().expect("should succeed");
        let socket_path = dir.path().join("crypto.sock");
        let path_str = socket_path.to_string_lossy().to_string();

        let result = temp_env::with_var("CRYPTO_SIGNING_ENDPOINT", Some(path_str.as_str()), || {
            rt.block_on(async {
                let listener = UnixListener::bind(&socket_path).expect("should succeed");

                let server_handle = tokio::spawn(async move {
                    let (stream, _) = listener.accept().await.expect("should succeed");
                    let mut reader = BufReader::new(stream);
                    let mut line = String::new();
                    reader.read_line(&mut line).await.expect("should succeed");
                    let req: serde_json::Value =
                        serde_json::from_str(&line).expect("should succeed");
                    assert_eq!(req["method"], "crypto.sign");
                    let sig_b64 = base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        b"mock-signature-64-bytes!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!",
                    );
                    let response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "result": { "signature": sig_b64 }
                    });
                    let mut stream = reader.into_inner();
                    stream
                        .write_all(response.to_string().as_bytes())
                        .await
                        .expect("should succeed");
                    stream.write_all(b"\n").await.expect("should succeed");
                });

                let mut provider = CapabilityCryptoProvider::new();
                let sig = provider.sign_ed25519(b"hello").await;
                let _ = server_handle.await;
                sig
            })
        });

        let signature = result.expect("should succeed");
        assert!(!signature.is_empty());
    }

    #[test]
    fn test_mock_socket_verify_ed25519() {
        let rt = tokio::runtime::Runtime::new().expect("should succeed");
        let dir = tempfile::tempdir().expect("should succeed");
        let socket_path = dir.path().join("crypto-verify.sock");
        let path_str = socket_path.to_string_lossy().to_string();

        let result = temp_env::with_var("CRYPTO_SIGNING_ENDPOINT", Some(path_str.as_str()), || {
            rt.block_on(async {
                let listener = UnixListener::bind(&socket_path).expect("should succeed");

                let server_handle = tokio::spawn(async move {
                    let (stream, _) = listener.accept().await.expect("should succeed");
                    let mut reader = BufReader::new(stream);
                    let mut line = String::new();
                    reader.read_line(&mut line).await.expect("should succeed");
                    let req: serde_json::Value =
                        serde_json::from_str(&line).expect("should succeed");
                    assert_eq!(req["method"], "crypto.verify");
                    let response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "result": { "valid": true }
                    });
                    let mut stream = reader.into_inner();
                    stream
                        .write_all(response.to_string().as_bytes())
                        .await
                        .expect("should succeed");
                    stream.write_all(b"\n").await.expect("should succeed");
                });

                let mut provider = CapabilityCryptoProvider::new();
                let valid = provider.verify_ed25519(b"data", b"sig", b"pubkey").await;
                let _ = server_handle.await;
                valid
            })
        });

        assert!(result.expect("should succeed"));
    }

    #[test]
    fn test_mock_socket_verify_ed25519_with_key_id() {
        let rt = tokio::runtime::Runtime::new().expect("should succeed");
        let dir = tempfile::tempdir().expect("should succeed");
        let socket_path = dir.path().join("crypto-keyid.sock");
        let path_str = socket_path.to_string_lossy().to_string();

        let result = temp_env::with_var("CRYPTO_SIGNING_ENDPOINT", Some(path_str.as_str()), || {
            rt.block_on(async {
                let listener = UnixListener::bind(&socket_path).expect("should succeed");

                let server_handle = tokio::spawn(async move {
                    let (stream, _) = listener.accept().await.expect("should succeed");
                    let mut reader = BufReader::new(stream);
                    let mut line = String::new();
                    reader.read_line(&mut line).await.expect("should succeed");
                    let req: serde_json::Value =
                        serde_json::from_str(&line).expect("should succeed");
                    assert_eq!(req["method"], "crypto.verify");
                    assert_eq!(req["params"]["key_id"], "jwt-key-1");
                    let response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "result": { "valid": false }
                    });
                    let mut stream = reader.into_inner();
                    stream
                        .write_all(response.to_string().as_bytes())
                        .await
                        .expect("should succeed");
                    stream.write_all(b"\n").await.expect("should succeed");
                });

                let mut provider = CapabilityCryptoProvider::new();
                let valid = provider
                    .verify_ed25519_with_key_id(b"data", b"sig", "jwt-key-1")
                    .await;
                let _ = server_handle.await;
                valid
            })
        });

        assert!(!result.expect("should succeed"));
    }
}
