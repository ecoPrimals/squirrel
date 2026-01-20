//! Neural API HTTP Client Wrapper for Squirrel
//!
//! This module provides a bridge between Squirrel's existing capability_http
//! interface and the new neural-api-client for TRUE PRIMAL routing.
//!
//! # Migration Strategy
//!
//! Phase 1: Keep capability_http as-is (existing code)
//! Phase 2: Add neural_http as alternative (this module)
//! Phase 3: Migrate callers to neural_http
//! Phase 4: Deprecate capability_http
//!
//! # TRUE PRIMAL Pattern
//!
//! - Zero knowledge of Songbird or BearDog
//! - Runtime discovery via family_id
//! - No reqwest, no ring, 100% Pure Rust

use anyhow::{Context, Result};
use neural_api_client::{HttpResponse as NeuralHttpResponse, NeuralApiClient};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP request wrapper (compatible with existing code)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

/// HTTP response wrapper (compatible with existing code)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl From<NeuralHttpResponse> for HttpResponse {
    fn from(response: NeuralHttpResponse) -> Self {
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
/// This client uses the Neural API for routing instead of direct HTTP.
/// Squirrel doesn't know about Songbird or BearDog - it just asks for
/// "http proxy capability" and Neural API routes it.
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
///     url: "https://api.anthropic.com/v1/messages".to_string(),
///     headers: vec![
///         ("x-api-key".to_string(), "sk-...".to_string()),
///     ],
///     body: Some(r#"{"model": "claude-3-opus-20240229"}"#.to_string()),
/// };
///
/// let response = client.request(request).await?;
/// println!("Status: {}", response.status);
/// # Ok(())
/// # }
/// ```
pub struct NeuralHttpClient {
    neural_client: NeuralApiClient,
}

impl NeuralHttpClient {
    /// Create client by discovering Neural API socket
    ///
    /// # TRUE PRIMAL Pattern
    ///
    /// Socket path is discovered at runtime based on family_id.
    /// No hardcoded paths, no primal names!
    pub fn discover(family_id: &str) -> Result<Self> {
        let neural_client =
            NeuralApiClient::discover(family_id).context("Failed to discover Neural API")?;

        Ok(Self { neural_client })
    }

    /// Create client with explicit socket path (for testing)
    pub fn new(socket_path: impl Into<std::path::PathBuf>) -> Result<Self> {
        let neural_client = NeuralApiClient::new(socket_path)?;
        Ok(Self { neural_client })
    }

    /// Make HTTP request via Neural API routing
    ///
    /// This delegates to Neural API, which:
    /// 1. Discovers Tower Atomic (Songbird + BearDog)
    /// 2. Routes request to Songbird
    /// 3. Songbird uses BearDog for crypto/TLS
    /// 4. Returns response
    ///
    /// **Squirrel knows NONE of this!** Just asks for HTTP capability.
    pub async fn request(&self, request: HttpRequest) -> Result<HttpResponse> {
        // Convert headers from Vec<(String, String)> to HashMap
        let headers: HashMap<String, String> = request.headers.into_iter().collect();

        // Parse body as JSON (if present)
        let body = if let Some(body_str) = request.body {
            Some(serde_json::from_str(&body_str).context("Failed to parse body as JSON")?)
        } else {
            None
        };

        // Call Neural API proxy_http
        let response = self
            .neural_client
            .proxy_http(&request.method, &request.url, Some(headers), body)
            .await
            .context("Neural API HTTP proxy failed")?;

        // Convert response
        Ok(response.into())
    }

    /// Convenience: POST JSON
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
    pub async fn get(&self, url: &str, headers: Vec<(String, String)>) -> Result<HttpResponse> {
        self.request(HttpRequest {
            method: "GET".to_string(),
            url: url.to_string(),
            headers,
            body: None,
        })
        .await
    }

    /// Get Neural API routing metrics (for debugging)
    pub async fn get_metrics(&self) -> Result<neural_api_client::RoutingMetrics> {
        self.neural_client
            .get_metrics()
            .await
            .context("Failed to get routing metrics")
    }

    /// Discover capability information (for debugging)
    pub async fn discover_capability(
        &self,
        capability: &str,
    ) -> Result<neural_api_client::CapabilityInfo> {
        self.neural_client
            .discover_capability(capability)
            .await
            .context("Failed to discover capability")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_discovery() {
        // Should not panic during construction
        // (actual connection will fail if Neural API not running)
        let _result = NeuralHttpClient::discover("test");
        // OK if this returns Err - Neural API might not be running in tests
    }

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
    fn test_response_conversion() {
        let neural_response = NeuralHttpResponse {
            status: 200,
            headers: HashMap::from([("content-type".to_string(), "application/json".to_string())]),
            body: r#"{"success": true}"#.to_string(),
        };

        let response: HttpResponse = neural_response.into();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("success"));
    }
}
