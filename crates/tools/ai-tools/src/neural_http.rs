// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Neural API HTTP Client Wrapper for Squirrel
//!
//! This module provides a bridge between Squirrel's existing `capability_http`
//! interface and squirrel's own IPC client for TRUE PRIMAL routing.
//!
//! # TRUE PRIMAL Pattern
//!
//! - Zero knowledge of Songbird or `BearDog`
//! - Runtime discovery via `family_id`
//! - No reqwest, no ring, 100% Pure Rust
//! - Uses squirrel's autonomous IPC client (primal autonomy)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// Re-export types from the autonomous IPC client (primal autonomy)
pub use universal_patterns::ipc_client::{
    CapabilityInfo, IpcClient, IpcClientError, ProviderInfo, RoutingMetrics,
};

/// HTTP request wrapper (compatible with existing code)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    /// HTTP method (e.g., "GET", "POST").
    pub method: String,
    /// Full request URL.
    pub url: String,
    /// Request headers as key-value pairs.
    pub headers: Vec<(String, String)>,
    /// Optional request body.
    pub body: Option<String>,
}

/// HTTP response wrapper (compatible with existing code)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// HTTP status code (e.g., 200, 404).
    pub status: u16,
    /// Response headers as key-value pairs.
    pub headers: Vec<(String, String)>,
    /// Response body content.
    pub body: String,
}

impl From<universal_patterns::ipc_client::HttpResponse> for HttpResponse {
    fn from(response: universal_patterns::ipc_client::HttpResponse) -> Self {
        Self {
            status: response.status,
            headers: response.headers.into_iter().collect(),
            body: response.body,
        }
    }
}

/// Neural HTTP Client - Pure Rust capability-based routing
///
/// # TRUE PRIMAL Pattern
///
/// This client uses squirrel's autonomous IPC client for routing.
/// Squirrel doesn't know about Songbird or `BearDog` — it just asks for
/// "http proxy capability" and the ecosystem routes it.
///
/// # Example
///
/// ```no_run
/// use squirrel_ai_tools::neural_http::{NeuralHttpClient, HttpRequest};
///
/// # async fn example() -> anyhow::Result<()> {
/// let client = NeuralHttpClient::discover("nat0")?;
///
/// let request = HttpRequest {
///     method: "POST".to_string(),
///     url: "https://api.example.com/v1/messages".to_string(),
///     headers: vec![
///         ("Authorization".to_string(), "Bearer ...".to_string()),
///     ],
///     body: Some(r#"{"model": "example-model"}"#.to_string()),
/// };
///
/// let response = client.request(request).await?;
/// println!("Status: {}", response.status);
/// # Ok(())
/// # }
/// ```
pub struct NeuralHttpClient {
    ipc_client: IpcClient,
}

impl NeuralHttpClient {
    /// Create client by discovering ecosystem socket at runtime
    ///
    /// # TRUE PRIMAL Pattern
    ///
    /// Socket path is discovered at runtime based on `service_id`.
    /// No hardcoded paths, no primal names!
    ///
    /// # Errors
    ///
    /// Returns an error when the ecosystem IPC socket cannot be discovered.
    pub fn discover(service_id: &str) -> Result<Self> {
        let ipc_client =
            IpcClient::discover(service_id).context("failed to discover ecosystem socket")?;
        Ok(Self { ipc_client })
    }

    /// Create client with explicit socket path (for testing)
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            ipc_client: IpcClient::new(socket_path),
        }
    }

    /// Make HTTP request via ecosystem routing
    ///
    /// Delegates to the ecosystem router, which handles TLS, crypto,
    /// and routing — Squirrel knows NONE of those details.
    ///
    /// # Errors
    ///
    /// Returns an error when the ecosystem HTTP proxy or body parsing fails.
    pub async fn request(&self, request: HttpRequest) -> Result<HttpResponse> {
        let headers: HashMap<String, String> = request.headers.into_iter().collect();
        let body = if let Some(body_str) = request.body {
            Some(serde_json::from_str(&body_str).context("failed to parse body as JSON")?)
        } else {
            None
        };

        let response = self
            .ipc_client
            .proxy_http(&request.method, &request.url, Some(headers), body)
            .await
            .context("ecosystem HTTP proxy failed")?;

        Ok(response.into())
    }

    /// Convenience: POST JSON
    ///
    /// # Errors
    ///
    /// Same as [`Self::request`].
    pub async fn post_json(
        &self,
        url: &str,
        headers: Vec<(String, String)>,
        body: &str,
    ) -> Result<HttpResponse> {
        let mut all_headers = headers;
        if !all_headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        {
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

    /// Convenience: GET
    ///
    /// # Errors
    ///
    /// Same as [`Self::request`].
    pub async fn get(&self, url: &str, headers: Vec<(String, String)>) -> Result<HttpResponse> {
        self.request(HttpRequest {
            method: "GET".to_string(),
            url: url.to_string(),
            headers,
            body: None,
        })
        .await
    }

    /// Get routing metrics (for observability)
    ///
    /// # Errors
    ///
    /// Returns an error when the IPC metrics call fails.
    pub async fn get_metrics(&self) -> Result<RoutingMetrics> {
        self.ipc_client
            .get_metrics()
            .await
            .context("failed to get routing metrics")
    }

    /// Discover capability information
    ///
    /// # Errors
    ///
    /// Returns an error when capability discovery over IPC fails.
    pub async fn discover_capability(&self, capability: &str) -> Result<CapabilityInfo> {
        self.ipc_client
            .discover_capability(capability)
            .await
            .context("failed to discover capability")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_request_building() {
        let request = HttpRequest {
            method: "POST".to_string(),
            url: "https://api.example.com/test".to_string(),
            headers: vec![("Authorization".to_string(), "Bearer test".to_string())],
            body: Some(r#"{"test": true}"#.to_string()),
        };

        assert_eq!(request.method, "POST");
        assert_eq!(request.url, "https://api.example.com/test");
        assert_eq!(request.headers.len(), 1);
    }

    #[test]
    fn test_http_request_response_serde_roundtrip() {
        let req = HttpRequest {
            method: "GET".to_string(),
            url: "https://x".to_string(),
            headers: vec![("a".to_string(), "b".to_string())],
            body: None,
        };
        let j = serde_json::to_string(&req).unwrap();
        let back: HttpRequest = serde_json::from_str(&j).unwrap();
        assert_eq!(back.method, "GET");

        let res = HttpResponse {
            status: 201,
            headers: vec![("X-Foo".to_string(), "bar".to_string())],
            body: "{}".to_string(),
        };
        let j2 = serde_json::to_string(&res).unwrap();
        let back2: HttpResponse = serde_json::from_str(&j2).unwrap();
        assert_eq!(back2.status, 201);
    }

    #[test]
    fn test_http_response_from_ipc() {
        let inner = universal_patterns::ipc_client::HttpResponse {
            status: 418,
            headers: HashMap::from([("h".to_string(), "v".to_string())]),
            body: "teapot".to_string(),
        };
        let outer: HttpResponse = inner.into();
        assert_eq!(outer.status, 418);
        assert_eq!(outer.body, "teapot");
        assert!(outer.headers.iter().any(|(k, _)| k == "h"));
    }

    #[test]
    fn test_client_discovery_graceful() {
        // Should not panic — returns Err if socket not found
        let result = NeuralHttpClient::discover("nonexistent-test");
        assert!(result.is_err());
    }

    #[test]
    fn test_neural_client_new_accepts_path() {
        let _ = NeuralHttpClient::new("/tmp/neural-http-test-no-such.sock");
    }

    #[tokio::test]
    async fn request_rejects_invalid_json_body_before_ipc() {
        let client = NeuralHttpClient::new("/tmp/neural-http-test-no-such.sock");
        let err = client
            .request(HttpRequest {
                method: "POST".to_string(),
                url: "https://example.com".to_string(),
                headers: vec![],
                body: Some("not-json".to_string()),
            })
            .await
            .unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("parse") || msg.contains("json") || msg.contains("JSON"),
            "unexpected error: {msg}"
        );
    }
}
