// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! IPC-backed secret store — delegates to the security provider's `secrets.*` JSON-RPC.
//!
//! [`SecurityProviderSecretStore`] implements [`SecretStore`] by connecting to
//! the security capability provider over Unix socket or TCP and calling
//! `secrets.store`, `secrets.retrieve`, `secrets.list`, and `secrets.delete`.
//!
//! Discovery follows the standard tiered resolution:
//! 1. `SECURITY_ENDPOINT` env var (full URL)
//! 2. `BEARDOG_ENDPOINT` env var (deprecated fallback)
//! 3. Socket discovery via `resolve_capability_unix_socket`
//!
//! When the security provider is unreachable, all operations return graceful errors —
//! callers should fall back to env vars or the platform cache.

use super::secret_store::SecretStore;
use crate::error::{MCPError, Result};
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, warn};
use universal_patterns::transport::{TransportEndpoint, connect_transport_with_timeout};

const IPC_TIMEOUT: Duration = Duration::from_secs(5);

/// Secret store that delegates to the security provider's `secrets.*` JSON-RPC interface.
///
/// This is the **production** secret store — the security provider is the credential
/// authority. When the security provider is not running, operations fail gracefully
/// and callers should fall back to env vars or the platform cache.
#[derive(Debug, Clone)]
pub struct SecurityProviderSecretStore {
    endpoint: TransportEndpoint,
}

impl SecurityProviderSecretStore {
    /// Create a store targeting a specific endpoint.
    #[must_use]
    pub const fn new(endpoint: TransportEndpoint) -> Self {
        Self { endpoint }
    }

    /// Auto-discover the security provider endpoint.
    ///
    /// Resolution: `SECURITY_ENDPOINT` → `BEARDOG_ENDPOINT` → socket discovery.
    #[must_use]
    pub fn discover() -> Self {
        let endpoint = resolve_security_endpoint();
        debug!(?endpoint, "SecurityProviderSecretStore endpoint resolved");
        Self { endpoint }
    }

    /// Send a JSON-RPC request to the security provider and parse the response.
    async fn rpc_call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        let request_bytes = {
            let mut buf = serde_json::to_vec(&request).map_err(|e| {
                MCPError::Internal(format!("Failed to serialize JSON-RPC request: {e}"))
            })?;
            buf.push(b'\n');
            buf
        };

        let mut stream = connect_transport_with_timeout(&self.endpoint, IPC_TIMEOUT)
            .await
            .map_err(|e| {
                MCPError::Internal(format!(
                    "Failed to connect to security provider at {:?}: {e}",
                    self.endpoint
                ))
            })?;

        // BTSP handshake when strict mode is active (sporeGate LIVE, eastGate next)
        if let Err(e) = super::btsp_client::maybe_client_handshake(&mut stream).await {
            warn!(error = %e, "BTSP client handshake failed — falling back to plain JSON-RPC");
        }

        stream
            .write_all(&request_bytes)
            .await
            .map_err(|e| MCPError::Internal(format!("Failed to send to security provider: {e}")))?;
        stream.flush().await.map_err(|e| {
            MCPError::Internal(format!("Failed to flush to security provider: {e}"))
        })?;

        let mut reader = BufReader::new(stream);
        let mut response_line = String::new();

        tokio::time::timeout(IPC_TIMEOUT, reader.read_line(&mut response_line))
            .await
            .map_err(|_| MCPError::Internal("Security provider response timed out".to_string()))?
            .map_err(|e| {
                MCPError::Internal(format!("Failed to read from security provider: {e}"))
            })?;

        let response: serde_json::Value =
            serde_json::from_str(response_line.trim()).map_err(|e| {
                MCPError::Internal(format!(
                    "Invalid JSON-RPC response from security provider: {e}"
                ))
            })?;

        if let Some(error) = response.get("error") {
            let msg = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error");
            return Err(MCPError::Internal(format!(
                "Security provider error on {method}: {msg}"
            )));
        }

        response.get("result").cloned().ok_or_else(|| {
            MCPError::Internal(format!("Security provider returned no result for {method}"))
        })
    }
}

impl SecretStore for SecurityProviderSecretStore {
    async fn get(&self, name: &str) -> Result<Option<Vec<u8>>> {
        match self
            .rpc_call("secrets.retrieve", serde_json::json!({"name": name}))
            .await
        {
            Ok(result) => {
                let value = result
                    .get("value")
                    .and_then(|v| v.as_str())
                    .map(|s| s.as_bytes().to_vec());
                Ok(value)
            }
            Err(e) => {
                warn!(
                    name,
                    error = %e,
                    "secrets.retrieve failed — security provider may be unavailable"
                );
                Err(e)
            }
        }
    }

    async fn set(&self, name: &str, value: Vec<u8>) -> Result<()> {
        let value_str = String::from_utf8(value).map_err(|e| {
            MCPError::Internal(format!("Secret value must be valid UTF-8 for IPC: {e}"))
        })?;

        self.rpc_call(
            "secrets.store",
            serde_json::json!({"name": name, "value": value_str}),
        )
        .await?;
        Ok(())
    }

    async fn delete(&self, name: &str) -> Result<bool> {
        let result = self
            .rpc_call("secrets.delete", serde_json::json!({"name": name}))
            .await?;

        Ok(result
            .get("deleted")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false))
    }

    async fn list_keys(&self) -> Result<Vec<String>> {
        let result = self.rpc_call("secrets.list", serde_json::json!({})).await?;

        let keys = result
            .get("secrets")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(keys)
    }
}

/// Resolve the security provider endpoint via tiered discovery.
fn resolve_security_endpoint() -> TransportEndpoint {
    if let Ok(url) = std::env::var("SECURITY_ENDPOINT")
        && !url.is_empty()
        && let Some(ep) = parse_endpoint_url(&url)
    {
        return ep;
    }

    if let Ok(url) = std::env::var("BEARDOG_ENDPOINT")
        && !url.is_empty()
    {
        warn!("BEARDOG_ENDPOINT is deprecated — use SECURITY_ENDPOINT");
        if let Some(ep) = parse_endpoint_url(&url) {
            return ep;
        }
    }

    let socket_path =
        universal_constants::network::resolve_capability_unix_socket("SECURITY_SOCKET", "security");
    TransportEndpoint::uds(socket_path.to_string_lossy())
}

/// Parse a URL-like string into a `TransportEndpoint`.
fn parse_endpoint_url(url: &str) -> Option<TransportEndpoint> {
    if let Some(path) = url.strip_prefix("unix://") {
        return Some(TransportEndpoint::uds(path));
    }
    if url.starts_with('/')
        || Path::new(url)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
    {
        return Some(TransportEndpoint::uds(url));
    }
    if let Some(rest) = url.strip_prefix("tcp://")
        && let Some((host, port_str)) = rest.rsplit_once(':')
        && let Ok(port) = port_str.parse::<u16>()
    {
        return Some(TransportEndpoint::tcp(host, port));
    }
    if let Some((host, port_str)) = url.rsplit_once(':')
        && let Ok(port) = port_str.parse::<u16>()
    {
        return Some(TransportEndpoint::tcp(host, port));
    }
    None
}

// ---------------------------------------------------------------------------
// Convenience: resolve a secret with env var fallback
// ---------------------------------------------------------------------------

/// Try to retrieve a secret from the security provider, falling back to an
/// environment variable if the security provider is unavailable.
///
/// This is the recommended pattern for AI API keys and other runtime secrets:
/// ```ignore
/// let key = resolve_secret_or_env("openai_api_key", "OPENAI_API_KEY").await;
/// ```
pub async fn resolve_secret_or_env(
    store: &impl SecretStore,
    secret_name: &str,
    env_var: &str,
) -> Option<String> {
    match store.get(secret_name).await {
        Ok(Some(bytes)) => {
            if let Ok(s) = String::from_utf8(bytes) {
                debug!(secret_name, "Secret resolved via security provider");
                return Some(s);
            }
        }
        Ok(None) => {
            debug!(secret_name, "Secret not found in security provider");
        }
        Err(e) => {
            debug!(
                secret_name,
                error = %e,
                "Security provider unavailable, falling back to env"
            );
        }
    }

    std::env::var(env_var).ok().inspect(|_| {
        debug!(env_var, "Secret resolved via environment variable");
    })
}

#[cfg(test)]
mod tests {
    use super::super::secret_store::InMemorySecretStore;
    use super::*;

    #[test]
    fn parse_endpoint_url_uds() {
        let ep = parse_endpoint_url("unix:///run/beardog.sock");
        assert_eq!(ep, Some(TransportEndpoint::uds("/run/beardog.sock")));

        let ep = parse_endpoint_url("/tmp/beardog.sock");
        assert_eq!(ep, Some(TransportEndpoint::uds("/tmp/beardog.sock")));
    }

    #[test]
    fn parse_endpoint_url_tcp() {
        let ep = parse_endpoint_url("tcp://192.168.1.1:7700");
        assert_eq!(ep, Some(TransportEndpoint::tcp("192.168.1.1", 7700)));

        let ep = parse_endpoint_url("localhost:7700");
        assert_eq!(ep, Some(TransportEndpoint::tcp("localhost", 7700)));
    }

    #[test]
    fn parse_endpoint_url_invalid() {
        assert!(parse_endpoint_url("").is_none());
        assert!(parse_endpoint_url("not-a-url").is_none());
    }

    #[test]
    fn discover_uses_default_socket() {
        temp_env::with_vars_unset(
            ["SECURITY_ENDPOINT", "BEARDOG_ENDPOINT", "SECURITY_SOCKET"],
            || {
                let store = SecurityProviderSecretStore::discover();
                match &store.endpoint {
                    TransportEndpoint::Uds { path } => {
                        assert!(
                            path.contains("security"),
                            "should resolve to security provider socket: {path}"
                        );
                    }
                    other => panic!("Expected UDS endpoint, got {other:?}"),
                }
            },
        );
    }

    #[test]
    fn discover_prefers_security_endpoint_env() {
        temp_env::with_vars(
            [
                ("SECURITY_ENDPOINT", Some("tcp://10.0.0.1:7700")),
                ("BEARDOG_ENDPOINT", Some("tcp://10.0.0.2:7700")),
            ],
            || {
                let store = SecurityProviderSecretStore::discover();
                assert_eq!(store.endpoint, TransportEndpoint::tcp("10.0.0.1", 7700));
            },
        );
    }

    #[tokio::test]
    async fn resolve_secret_or_env_from_store() {
        let store = InMemorySecretStore::new();
        store.set("my_key", b"from_store".to_vec()).await.unwrap();

        let result = resolve_secret_or_env(&store, "my_key", "NONEXISTENT_ENV_VAR_XYZ").await;
        assert_eq!(result, Some("from_store".to_string()));
    }

    #[test]
    fn resolve_secret_or_env_falls_back_to_env() {
        let store = InMemorySecretStore::new();

        temp_env::with_var("TEST_FALLBACK_KEY_XYZ", Some("from_env"), || {
            let rt = tokio::runtime::Runtime::new().expect("runtime");
            let result = rt.block_on(resolve_secret_or_env(
                &store,
                "missing_key",
                "TEST_FALLBACK_KEY_XYZ",
            ));
            assert_eq!(result, Some("from_env".to_string()));
        });
    }

    #[tokio::test]
    async fn resolve_secret_or_env_returns_none_when_both_missing() {
        let store = InMemorySecretStore::new();
        let result = resolve_secret_or_env(&store, "nope", "NONEXISTENT_ENV_XYZ_123").await;
        assert!(result.is_none());
    }
}
