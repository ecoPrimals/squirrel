//! # HTTPS Fallback Server
//!
//! Following Songbird pattern for HTTPS as compatibility layer.
//! Provides RESTful API when tarpc/JSON-RPC unavailable.
//!
//! ## Design
//!
//! - Feature-gated (optional)
//! - TLS support (optional)
//! - RESTful endpoints
//! - Compatible with any HTTP client
//!
//! ## Endpoints
//!
//! - `POST /api/v1/query` - AI query
//! - `GET /api/v1/providers` - List providers
//! - `GET /api/v1/health` - Health check
//! - `GET /api/v1/capabilities` - List capabilities

use crate::error::PrimalError;
use serde::{Deserialize, Serialize};

// Re-export for protocol router
#[derive(Debug, Deserialize, Serialize)]
pub struct QueryRequest {
    pub prompt: String,
    pub provider: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct QueryResponse {
    pub response: String,
    pub provider: String,
    pub model: String,
}
use std::net::SocketAddr;
use std::sync::Arc;
use warp::{Filter, Reply};

/// HTTPS fallback server configuration
#[derive(Debug, Clone)]
pub struct HttpsFallbackConfig {
    /// Bind address
    pub bind_addr: SocketAddr,
    /// Enable TLS
    pub tls_enabled: bool,
    /// TLS certificate path (if enabled)
    pub cert_path: Option<String>,
    /// TLS key path (if enabled)
    pub key_path: Option<String>,
}

impl Default for HttpsFallbackConfig {
    fn default() -> Self {
        use std::env;

        let port = env::var("HTTPS_FALLBACK_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(9010);

        let bind_addr = format!("0.0.0.0:{}", port)
            .parse()
            .expect("Invalid bind address");

        Self {
            bind_addr,
            tls_enabled: env::var("HTTPS_TLS_ENABLED").is_ok(),
            cert_path: env::var("HTTPS_CERT_PATH").ok(),
            key_path: env::var("HTTPS_KEY_PATH").ok(),
        }
    }
}

/// HTTPS fallback server (Songbird pattern)
pub struct HttpsFallbackServer {
    config: HttpsFallbackConfig,
}

impl HttpsFallbackServer {
    /// Create new HTTPS fallback server
    pub fn new(config: HttpsFallbackConfig) -> Self {
        Self { config }
    }

    /// Start HTTPS server
    pub async fn start(self: Arc<Self>) -> Result<(), PrimalError> {
        tracing::info!(
            "Starting HTTPS fallback server on {}",
            self.config.bind_addr
        );

        let routes = self.build_routes();

        // Note: TLS support requires additional crates (warp-tls or similar)
        // For now, run HTTP only and document TLS as TODO
        if self.config.tls_enabled {
            tracing::warn!("TLS requested but not yet implemented - running HTTP");
            tracing::warn!("TODO: Add warp TLS support (requires additional dependencies)");
        }

        tracing::info!("Starting HTTP server (TLS support pending)");
        warp::serve(routes).run(self.config.bind_addr).await;

        Ok(())
    }

    /// Build warp routes
    fn build_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let query = warp::path!("api" / "v1" / "query")
            .and(warp::post())
            .and(warp::body::json())
            .and_then(handle_query);

        let providers = warp::path!("api" / "v1" / "providers")
            .and(warp::get())
            .and_then(handle_list_providers);

        let health = warp::path!("api" / "v1" / "health")
            .and(warp::get())
            .and_then(handle_health);

        let capabilities = warp::path!("api" / "v1" / "capabilities")
            .and(warp::get())
            .and_then(handle_capabilities);

        // CORS for browser access
        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "POST"])
            .allow_headers(vec!["Content-Type"]);

        query
            .or(providers)
            .or(health)
            .or(capabilities)
            .with(cors)
            .with(warp::trace::request())
    }
}

/// Request/Response types (internal, private versions)

#[derive(Debug, Serialize)]
struct ProviderInfo {
    id: String,
    name: String,
    available: bool,
    models: Vec<String>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime_secs: u64,
}

#[derive(Debug, Serialize)]
struct CapabilitiesResponse {
    capabilities: Vec<String>,
    protocols: Vec<String>,
}

/// Handler functions

async fn handle_query(request: QueryRequest) -> Result<impl Reply, warp::Rejection> {
    tracing::debug!("HTTPS query request: {}", request.prompt);

    // Call actual query handler
    match handle_https_request("query_ai", serde_json::to_value(&request).unwrap()).await {
        Ok(result) => {
            // Parse response directly as JSON value, then convert
            let response = QueryResponse {
                response: result["response"].as_str().unwrap_or("").to_string(),
                provider: result["provider"].as_str().unwrap_or("unknown").to_string(),
                model: result["model"].as_str().unwrap_or("unknown").to_string(),
            };
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            tracing::error!("Query failed: {}", e);
            Ok(warp::reply::json(&QueryResponse {
                response: format!("Error: {}", e),
                provider: "error".to_string(),
                model: "error".to_string(),
            }))
        }
    }
}

async fn handle_list_providers() -> Result<impl Reply, warp::Rejection> {
    tracing::debug!("HTTPS list providers request");

    match handle_https_request("list_providers", serde_json::json!({})).await {
        Ok(result) => Ok(warp::reply::json(&result)),
        Err(e) => {
            tracing::error!("List providers failed: {}", e);
            Ok(warp::reply::json(&Vec::<ProviderInfo>::new()))
        }
    }
}

async fn handle_health() -> Result<impl Reply, warp::Rejection> {
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: 0, // TODO: Track actual uptime
    };
    Ok(warp::reply::json(&response))
}

async fn handle_capabilities() -> Result<impl Reply, warp::Rejection> {
    let response = CapabilitiesResponse {
        capabilities: vec![
            "ai.query".to_string(),
            "ai.routing".to_string(),
            "mcp.protocol".to_string(),
        ],
        protocols: vec![
            "https".to_string(),
            "json-rpc".to_string(),
            #[cfg(feature = "tarpc-rpc")]
            "tarpc".to_string(),
        ],
    };
    Ok(warp::reply::json(&response))
}

/// Handle HTTPS request (called by protocol router)
pub async fn handle_https_request(
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, PrimalError> {
    tracing::debug!("Handling HTTPS request: method={}", method);

    match method {
        "query_ai" => {
            // Parse request
            let request: QueryRequest = serde_json::from_value(params)
                .map_err(|e| PrimalError::InvalidInput(e.to_string()))?;

            // Call AI router (if available)
            // For now, return a structured response
            Ok(serde_json::json!({
                "response": format!("Response to: {}", request.prompt),
                "provider": request.provider.unwrap_or_else(|| "default".to_string()),
                "model": request.model.unwrap_or_else(|| "default".to_string()),
            }))
        }
        "list_providers" => {
            // Return available providers
            Ok(serde_json::json!([
                {
                    "id": "openai",
                    "name": "OpenAI",
                    "available": true,
                    "models": ["gpt-4", "gpt-3.5-turbo"]
                }
            ]))
        }
        _ => Err(PrimalError::InvalidInput(format!(
            "Unknown method: {}",
            method
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = HttpsFallbackConfig::default();
        assert_eq!(config.bind_addr.port(), 9010);
        assert!(!config.tls_enabled);
    }

    #[tokio::test]
    async fn test_handle_health() {
        let result = handle_health().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_capabilities() {
        let result = handle_capabilities().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_https_request_query() {
        let params = serde_json::json!({
            "prompt": "Hello",
            "provider": "openai",
            "model": "gpt-4"
        });

        let result = handle_https_request("query_ai", params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_https_request_providers() {
        let result = handle_https_request("list_providers", serde_json::json!({})).await;
        assert!(result.is_ok());
    }
}
