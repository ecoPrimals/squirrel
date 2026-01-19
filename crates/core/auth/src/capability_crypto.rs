//! Capability-Based Crypto Client (TRUE PRIMAL!)
//!
//! **Evolution**: BearDog → Capability Discovery
//! - OLD: Hardcoded "BearDog" (DEV knowledge!)
//! - NEW: Discover "crypto.sign" capability (TRUE PRIMAL!)
//!
//! **Philosophy**: Deploy like an infant - knows nothing, discovers everything!
//! - Squirrel doesn't know "BearDog" exists
//! - Squirrel asks: "Who provides crypto.ed25519.sign?"
//! - Runtime answers: "Service at /var/run/crypto/provider.sock"
//! - Could be BearDog, could be any crypto primal!

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};
use tracing::{debug, info, warn};

/// Crypto client configuration (capability-based!)
#[derive(Debug, Clone)]
pub struct CryptoClientConfig {
    /// Path to crypto capability provider's Unix socket
    /// (Discovered at runtime, NOT hardcoded!)
    pub socket_path: PathBuf,

    /// Timeout for socket operations (default: 5 seconds)
    pub timeout_secs: u64,

    /// Number of connection retries (default: 3)
    pub max_retries: usize,

    /// Delay between retries in milliseconds (default: 100ms)
    pub retry_delay_ms: u64,
}

impl Default for CryptoClientConfig {
    fn default() -> Self {
        Self {
            // Default path from environment or discovery
            socket_path: std::env::var("CRYPTO_CAPABILITY_SOCKET")
                .unwrap_or_else(|_| "/var/run/crypto/provider.sock".to_string())
                .into(),
            timeout_secs: 5,
            max_retries: 3,
            retry_delay_ms: 100,
        }
    }
}

/// Capability-based crypto client (TRUE PRIMAL!)
///
/// **NO hardcoded primal names!**
/// - Discovers "crypto.ed25519.sign" capability at runtime
/// - Connects to whichever primal provides it
/// - Currently: Might be BearDog, might be any crypto primal
/// - Future: Could be multiple providers with failover
///
/// # Examples
///
/// ```no_run
/// use squirrel_mcp_auth::capability_crypto::{CryptoClient, CryptoClientConfig};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Socket path comes from capability discovery (NOT hardcoded!)
///     // let socket = discover_capability("crypto.ed25519.sign").await?.socket_path;
///     
///     let config = CryptoClientConfig {
///         socket_path: "/var/run/crypto/provider.sock".into(),  // From discovery!
///         ..Default::default()
///     };
///     
///     let client = CryptoClient::new(config)?;
///     
///     let data = b"Hello, ecoPrimals!";
///     let signature = client.ed25519_sign(data, "my-key-id").await?;
///     let is_valid = client.ed25519_verify(data, &signature, "my-key-id").await?;
///     
///     assert!(is_valid);
///     Ok(())
/// }
/// ```
pub struct CryptoClient {
    config: CryptoClientConfig,
    request_id: std::sync::atomic::AtomicU64,
}

impl CryptoClient {
    /// Create a new crypto client from capability discovery
    ///
    /// **IMPORTANT**: Socket path should come from capability discovery!
    /// Do NOT hardcode primal names like "beardog-nat0.sock"!
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // ✅ CORRECT: From capability discovery
    /// let capability = discover_capability("crypto.ed25519.sign").await?;
    /// let client = CryptoClient::new(CryptoClientConfig {
    ///     socket_path: capability.socket_path,
    ///     ..Default::default()
    /// })?;
    ///
    /// // ❌ WRONG: Hardcoded primal knowledge
    /// let client = CryptoClient::new(CryptoClientConfig {
    ///     socket_path: "/var/run/beardog/crypto.sock".into(),  // NO!
    ///     ..Default::default()
    /// })?;
    /// ```
    pub fn new(config: CryptoClientConfig) -> Result<Self> {
        info!(
            "Initializing capability-based crypto client: {}",
            config.socket_path.display()
        );

        // Don't validate socket exists (may not be started yet)
        // Discovery system handles availability

        Ok(Self {
            config,
            request_id: std::sync::atomic::AtomicU64::new(1),
        })
    }

    /// Create from environment (for bootstrapping)
    ///
    /// Reads `CRYPTO_CAPABILITY_SOCKET` from environment.
    /// This should be set by capability discovery at startup!
    pub fn from_env() -> Result<Self> {
        let config = CryptoClientConfig::default();
        info!(
            "Crypto client from env: CRYPTO_CAPABILITY_SOCKET={}",
            config.socket_path.display()
        );
        Self::new(config)
    }

    /// Sign data using Ed25519 via discovered crypto capability
    ///
    /// # Arguments
    /// * `data` - Data to sign
    /// * `key_id` - Key ID in crypto provider (primal-specific, not our concern!)
    ///
    /// # Returns
    /// Raw Ed25519 signature (64 bytes)
    pub async fn ed25519_sign(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>> {
        debug!(
            "Ed25519 sign request via capability: key_id={}, data_len={}",
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
            .context("Missing 'signature' in crypto capability response")?;

        let signature = BASE64
            .decode(signature_b64)
            .context("Failed to decode signature from crypto capability")?;

        debug!("Ed25519 sign successful: signature_len={}", signature.len());

        Ok(signature)
    }

    /// Verify Ed25519 signature via discovered crypto capability
    ///
    /// # Arguments
    /// * `data` - Original data that was signed
    /// * `signature` - Ed25519 signature to verify (64 bytes)
    /// * `key_id` - Key ID in crypto provider
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
            "Ed25519 verify request via capability: key_id={}, data_len={}, sig_len={}",
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
            .context("Missing 'valid' field in crypto capability response")?;

        debug!("Ed25519 verify result: valid={}", valid);

        Ok(valid)
    }

    /// Send JSON-RPC request to crypto capability with retry logic
    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonValue> {
        let mut last_error = None;

        for attempt in 1..=self.config.max_retries {
            match self.send_request_once(&request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    warn!(
                        "Crypto capability request failed (attempt {}/{}): {}",
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
                "Crypto capability request failed after {} retries",
                self.config.max_retries
            )
        }))
    }

    /// Send a single JSON-RPC request to crypto capability provider
    async fn send_request_once(&self, request: &JsonRpcRequest) -> Result<JsonValue> {
        // Connect to Unix socket with timeout
        let stream = timeout(
            Duration::from_secs(self.config.timeout_secs),
            UnixStream::connect(&self.config.socket_path),
        )
        .await
        .context("Timeout connecting to crypto capability socket")?
        .context("Failed to connect to crypto capability socket")?;

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
        .context("Timeout sending request to crypto capability")?
        .context("Failed to send request to crypto capability")?;

        // Read response (newline-delimited JSON)
        let mut reader = BufReader::new(read_half);
        let mut response_line = String::new();

        timeout(
            Duration::from_secs(self.config.timeout_secs),
            reader.read_line(&mut response_line),
        )
        .await
        .context("Timeout reading response from crypto capability")?
        .context("Failed to read response from crypto capability")?;

        // Parse response
        let response: JsonRpcResponse = serde_json::from_str(&response_line)
            .context("Failed to parse JSON-RPC response from crypto capability")?;

        // Check for JSON-RPC error
        if let Some(error) = response.error {
            return Err(anyhow::anyhow!(
                "Crypto capability error: {} (code: {})",
                error.message,
                error.code
            ));
        }

        response
            .result
            .context("Crypto capability response missing 'result' field")
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
    fn test_crypto_client_creation() {
        let config = CryptoClientConfig::default();
        let client = CryptoClient::new(config);

        assert!(client.is_ok());
    }

    #[test]
    fn test_crypto_client_from_env() {
        std::env::set_var("CRYPTO_CAPABILITY_SOCKET", "/tmp/test-crypto.sock");

        let client = CryptoClient::from_env();
        assert!(client.is_ok());

        std::env::remove_var("CRYPTO_CAPABILITY_SOCKET");
    }

    #[test]
    fn test_request_id_increments() {
        let config = CryptoClientConfig::default();
        let client = CryptoClient::new(config).unwrap();

        let id1 = client.next_request_id();
        let id2 = client.next_request_id();
        let id3 = client.next_request_id();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    // Integration tests (require crypto capability provider running) are in integration tests
}
