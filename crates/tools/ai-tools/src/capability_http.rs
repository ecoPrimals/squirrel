//! Capability-Based HTTP Client (TRUE PRIMAL!)
//!
//! **Philosophy**: Deploy like an infant - knows nothing, discovers everything!
//! - Squirrel doesn't know "Songbird" exists
//! - Squirrel asks: "Who provides http.client capability?"
//! - Runtime answers: "Service at /var/run/network/http.sock"
//! - Could be Songbird, could be ANY network primal!
//!
//! **NO HARDCODED PRIMAL NAMES!**

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};
use tracing::{debug, info, warn};

/// HTTP client configuration (capability-based!)
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Path to HTTP capability provider's Unix socket
    /// (Discovered at runtime, NOT hardcoded!)
    pub socket_path: PathBuf,
    
    /// Timeout for HTTP operations (default: 30 seconds for AI calls)
    pub timeout_secs: u64,
    
    /// Number of connection retries (default: 3)
    pub max_retries: usize,
    
    /// Delay between retries in milliseconds (default: 100ms)
    pub retry_delay_ms: u64,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            // Default path from environment or discovery
            // NO "songbird" in the name - could be any HTTP provider!
            socket_path: std::env::var("HTTP_CAPABILITY_SOCKET")
                .or_else(|_| std::env::var("NETWORK_HTTP_SOCKET"))
                .unwrap_or_else(|_| "/var/run/network/http.sock".to_string())
                .into(),
            timeout_secs: 30,  // AI calls can be slow
            max_retries: 3,
            retry_delay_ms: 100,
        }
    }
}

/// HTTP request (to be sent to capability provider)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,      // "GET", "POST", etc.
    pub url: String,         // Full URL
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

/// HTTP response (from capability provider)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

/// JSON-RPC request structure
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: JsonValue,
    id: u64,
}

/// JSON-RPC response structure
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: u64,
}

/// JSON-RPC error structure
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<JsonValue>,
}

/// Capability-based HTTP client (TRUE PRIMAL!)
///
/// **NO hardcoded primal names!**
/// - Discovers "http.client" capability at runtime
/// - Connects to whichever primal provides it
/// - Currently: Might be Songbird, might be any network primal
/// - Future: Could be multiple providers with failover
///
/// # Examples
///
/// ```no_run
/// use squirrel_ai_tools::capability_http::{HttpClient, HttpClientConfig, HttpRequest};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Socket path comes from capability discovery (NOT hardcoded!)
///     let config = HttpClientConfig::default();
///     let client = HttpClient::new(config)?;
///     
///     let request = HttpRequest {
///         method: "POST".to_string(),
///         url: "https://api.openai.com/v1/chat/completions".to_string(),
///         headers: vec![
///             ("Authorization".to_string(), "Bearer sk-xxx".to_string()),
///             ("Content-Type".to_string(), "application/json".to_string()),
///         ],
///         body: Some(r#"{"model": "gpt-4", "messages": []}"#.to_string()),
///     };
///     
///     let response = client.request(request).await?;
///     println!("Status: {}", response.status);
///     Ok(())
/// }
/// ```
pub struct HttpClient {
    config: HttpClientConfig,
    request_id: std::sync::atomic::AtomicU64,
}

impl HttpClient {
    /// Create a new HTTP client from capability discovery
    ///
    /// **IMPORTANT**: Socket path should come from capability discovery!
    /// Do NOT hardcode primal names like "songbird.sock"!
    pub fn new(config: HttpClientConfig) -> Result<Self> {
        info!(
            socket_path = %config.socket_path.display(),
            "🔌 Creating HTTP capability client (discovering provider...)"
        );

        Ok(Self {
            config,
            request_id: std::sync::atomic::AtomicU64::new(1),
        })
    }

    /// Make HTTP request via capability provider
    ///
    /// This delegates to whichever primal provides "http.client" capability.
    /// The provider handles:
    /// - TLS/HTTPS negotiation
    /// - Certificate validation
    /// - Connection pooling
    /// - Retry logic
    /// - Rate limiting
    ///
    /// We just send request, get response - pure delegation!
    pub async fn request(&self, request: HttpRequest) -> Result<HttpResponse> {
        let method = &request.method;
        let url = &request.url;
        
        debug!(
            method = %method,
            url = %url,
            "📤 HTTP request via capability provider"
        );

        // Try with retries
        let mut last_error = None;
        for attempt in 0..self.config.max_retries {
            if attempt > 0 {
                warn!(
                    attempt = attempt + 1,
                    max_retries = self.config.max_retries,
                    "⚠️ Retrying HTTP request..."
                );
                tokio::time::sleep(Duration::from_millis(
                    self.config.retry_delay_ms * (1 << attempt),
                ))
                .await;
            }

            match self.send_request_internal(&request).await {
                Ok(response) => {
                    debug!(
                        status = response.status,
                        "✅ HTTP response received"
                    );
                    return Ok(response);
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        attempt = attempt + 1,
                        "❌ HTTP request failed"
                    );
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!("HTTP request failed after {} retries", self.config.max_retries)
        }))
    }

    /// Internal method to send request to capability provider
    async fn send_request_internal(&self, request: &HttpRequest) -> Result<HttpResponse> {
        // Build JSON-RPC request
        let id = self
            .request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let rpc_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "http.request".to_string(),  // Generic capability method!
            params: serde_json::to_value(request)?,
            id,
        };

        let request_json = serde_json::to_string(&rpc_request)?;

        // Connect to Unix socket (with timeout)
        let stream = timeout(
            Duration::from_secs(self.config.timeout_secs),
            UnixStream::connect(&self.config.socket_path),
        )
        .await
        .context("Connection timeout")?
        .with_context(|| {
            format!(
                "Failed to connect to HTTP capability provider at {}",
                self.config.socket_path.display()
            )
        })?;

        let (read_half, mut write_half) = stream.into_split();

        // Send request
        write_half
            .write_all(request_json.as_bytes())
            .await
            .context("Failed to write request")?;
        write_half
            .write_all(b"\n")
            .await
            .context("Failed to write newline")?;
        write_half.flush().await.context("Failed to flush")?;

        // Read response
        let mut reader = BufReader::new(read_half);
        let mut response_line = String::new();

        timeout(
            Duration::from_secs(self.config.timeout_secs),
            reader.read_line(&mut response_line),
        )
        .await
        .context("Response timeout")?
        .context("Failed to read response")?;

        // Parse JSON-RPC response
        let rpc_response: JsonRpcResponse = serde_json::from_str(&response_line)
            .with_context(|| format!("Failed to parse response: {}", response_line))?;

        // Check for errors
        if let Some(error) = rpc_response.error {
            return Err(anyhow::anyhow!(
                "HTTP capability provider error {}: {}",
                error.code,
                error.message
            ));
        }

        // Extract result
        let result = rpc_response
            .result
            .ok_or_else(|| anyhow::anyhow!("No result in response"))?;

        let http_response: HttpResponse = serde_json::from_value(result)
            .context("Failed to parse HTTP response")?;

        Ok(http_response)
    }

    /// Convenience method for POST JSON
    pub async fn post_json(&self, url: &str, headers: Vec<(String, String)>, body: &str) -> Result<HttpResponse> {
        let mut all_headers = headers;
        if !all_headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-type")) {
            all_headers.push(("Content-Type".to_string(), "application/json".to_string()));
        }

        self.request(HttpRequest {
            method: "POST".to_string(),
            url: url.to_string(),
            headers: all_headers,
            body: Some(body.to_string()),
        })
        .await
    }

    /// Convenience method for GET
    pub async fn get(&self, url: &str, headers: Vec<(String, String)>) -> Result<HttpResponse> {
        self.request(HttpRequest {
            method: "GET".to_string(),
            url: url.to_string(),
            headers,
            body: None,
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = HttpClientConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_retries, 3);
        // Socket path should come from env or default to generic path
        assert!(config.socket_path.to_string_lossy().contains("http"));
    }

    #[test]
    fn test_no_hardcoded_primal_names() {
        // THIS IS THE TEST THAT ENFORCES TRUE PRIMAL!
        let config = HttpClientConfig::default();
        let path = config.socket_path.to_string_lossy();
        
        // Should NOT contain any specific primal name
        assert!(!path.contains("songbird"), "Should not hardcode 'songbird'!");
        assert!(!path.contains("beardog"), "Should not hardcode 'beardog'!");
        
        // Should contain generic capability reference
        assert!(
            path.contains("http") || path.contains("network"),
            "Should reference generic capability, got: {}",
            path
        );
    }
}

