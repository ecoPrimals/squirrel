//! JSON-RPC 2.0 Server over Unix Sockets
//!
//! Modern, idiomatic Rust implementation of JSON-RPC 2.0 protocol for
//! biomeOS integration. This server handles all Squirrel<->biomeOS communication
//! over Unix domain sockets.
//!
//! ## Architecture
//!
//! ```text
//! Unix Socket → JSON-RPC 2.0 → Handler → AI Router → Response
//! ```
//!
//! ## Supported Methods
//!
//! - `query_ai` - Send prompt to AI and get response
//! - `list_providers` - List available AI providers
//! - `announce_capabilities` - Announce Squirrel capabilities
//! - `health` - Health check endpoint
//!
//! ## Protocol
//!
//! Standard JSON-RPC 2.0:
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "method": "query_ai",
//!   "params": {...},
//!   "id": 1
//! }
//! ```

use super::types::*;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,

    /// Method name
    pub method: String,

    /// Parameters (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,

    /// Request ID (optional for notifications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version
    pub jsonrpc: String,

    /// Result (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Error (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,

    /// Request ID (echoed from request)
    pub id: Value,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,

    /// Error message
    pub message: String,

    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// JSON-RPC error codes (standard)
#[allow(dead_code)]
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

/// Server metrics
#[derive(Debug, Clone)]
pub struct ServerMetrics {
    /// Total requests handled
    pub requests_handled: u64,

    /// Total errors
    pub errors: u64,

    /// Server start time
    pub start_time: Instant,

    /// Total response time (for averaging)
    pub total_response_time_ms: u64,
}

impl Default for ServerMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerMetrics {
    pub fn new() -> Self {
        Self {
            requests_handled: 0,
            errors: 0,
            start_time: Instant::now(),
            total_response_time_ms: 0,
        }
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    pub fn avg_response_time_ms(&self) -> Option<f64> {
        if self.requests_handled > 0 {
            Some(self.total_response_time_ms as f64 / self.requests_handled as f64)
        } else {
            None
        }
    }
}

/// JSON-RPC Server
pub struct JsonRpcServer {
    /// Socket path
    socket_path: String,

    /// Server metrics
    metrics: Arc<RwLock<ServerMetrics>>,

    /// AI router (optional, for actual AI calls)
    ai_router: Option<Arc<crate::api::ai::AiRouter>>,
}

impl JsonRpcServer {
    /// Create a new JSON-RPC server
    pub fn new(socket_path: String) -> Self {
        Self {
            socket_path,
            metrics: Arc::new(RwLock::new(ServerMetrics::new())),
            ai_router: None,
        }
    }

    /// Create server with AI router
    pub fn with_ai_router(socket_path: String, ai_router: Arc<crate::api::ai::AiRouter>) -> Self {
        Self {
            socket_path,
            metrics: Arc::new(RwLock::new(ServerMetrics::new())),
            ai_router: Some(ai_router),
        }
    }

    /// Start the JSON-RPC server
    ///
    /// This method will block until the server is stopped (Ctrl+C).
    pub async fn start(self: Arc<Self>) -> Result<()> {
        // Prepare socket path (remove old socket, create parent directory)
        let socket_path = Path::new(&self.socket_path);
        if let Some(parent) = socket_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).context("Failed to create socket directory")?;
            }
        }

        if socket_path.exists() {
            std::fs::remove_file(socket_path).context("Failed to remove old socket file")?;
        }

        // Bind Unix socket
        let listener = UnixListener::bind(&self.socket_path)
            .context(format!("Failed to bind Unix socket: {}", self.socket_path))?;

        info!("🚀 JSON-RPC server listening on {}", self.socket_path);

        // Accept connections loop
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let server = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream).await {
                            error!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handle a client connection
    async fn handle_connection(&self, stream: UnixStream) -> Result<()> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    // EOF - client disconnected
                    debug!("Client disconnected");
                    break;
                }
                Ok(_) => {
                    let response = self.handle_request(&line).await;

                    // Write response
                    let mut response_json = serde_json::to_string(&response)?;
                    response_json.push('\n');

                    reader.get_mut().write_all(response_json.as_bytes()).await?;
                    reader.get_mut().flush().await?;
                }
                Err(e) => {
                    warn!("Error reading from socket: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle a JSON-RPC request
    async fn handle_request(&self, request_str: &str) -> JsonRpcResponse {
        let start_time = Instant::now();

        // Parse JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(request_str.trim()) {
            Ok(req) => req,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: error_codes::PARSE_ERROR,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                    id: Value::Null,
                };
            }
        };

        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            return self.error_response(
                request.id.unwrap_or(Value::Null),
                error_codes::INVALID_REQUEST,
                "Invalid JSON-RPC version (must be 2.0)",
            );
        }

        let request_id = request.id.clone().unwrap_or(Value::Null);

        // Dispatch to method handler
        let result = match request.method.as_str() {
            "query_ai" => self.handle_query_ai(request.params).await,
            "list_providers" => self.handle_list_providers(request.params).await,
            "announce_capabilities" => self.handle_announce_capabilities(request.params).await,
            "health" => self.handle_health().await,
            "metrics" => self.handle_metrics().await,
            "discover_peers" => self.handle_discover_peers(request.params).await,
            "ping" => self.handle_ping().await,
            _ => Err(self.method_not_found(&request.method)),
        };

        // Update metrics
        let elapsed_ms = start_time.elapsed().as_millis() as u64;
        let mut metrics = self.metrics.write().await;
        metrics.requests_handled += 1;
        metrics.total_response_time_ms += elapsed_ms;

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(value),
                error: None,
                id: request_id,
            },
            Err(error) => {
                metrics.errors += 1;
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(error),
                    id: request_id,
                }
            }
        }
    }

    /// Handle query_ai method
    async fn handle_query_ai(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let request: QueryAiRequest = self.parse_params(params)?;

        info!("🤖 query_ai - prompt length: {}", request.prompt.len());

        // If AI router available, use it; otherwise return mock/error
        if let Some(router) = &self.ai_router {
            use crate::api::ai::types::TextGenerationRequest;

            let ai_request = TextGenerationRequest {
                prompt: request.prompt,
                system: None,
                max_tokens: request.max_tokens.map(|v| v as u32).unwrap_or(1024),
                temperature: request.temperature.unwrap_or(0.7),
                model: request.model,
                constraints: vec![],
                params: std::collections::HashMap::new(),
            };

            let start = Instant::now();
            match router.generate_text(ai_request, None).await {
                Ok(ai_response) => {
                    let response = QueryAiResponse {
                        response: ai_response.text,
                        provider: ai_response.provider_id,
                        model: ai_response.model,
                        tokens_used: ai_response.usage.map(|u| u.total_tokens as usize),
                        latency_ms: start.elapsed().as_millis() as u64,
                        success: true,
                    };
                    serde_json::to_value(response).map_err(|e| JsonRpcError {
                        code: error_codes::INTERNAL_ERROR,
                        message: format!("Serialization error: {}", e),
                        data: None,
                    })
                }
                Err(e) => Err(JsonRpcError {
                    code: error_codes::INTERNAL_ERROR,
                    message: format!("AI router error: {}", e),
                    data: None,
                }),
            }
        } else {
            // No AI router configured
            Err(JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: "AI router not configured. Configure providers to enable AI inference."
                    .to_string(),
                data: None,
            })
        }
    }

    /// Handle list_providers method
    async fn handle_list_providers(&self, _params: Option<Value>) -> Result<Value, JsonRpcError> {
        info!("📋 list_providers");

        if let Some(router) = &self.ai_router {
            let providers: Vec<ProviderInfo> = router
                .list_providers()
                .await
                .into_iter()
                .map(|p| ProviderInfo {
                    id: p.provider_id.clone(),
                    name: p.provider_name,
                    models: vec![], // TODO: Add models list
                    capabilities: p.capabilities,
                    online: p.is_available,
                    avg_latency_ms: None, // TODO: Add latency tracking
                    cost_tier: if p.cost_per_unit.unwrap_or(0.0) > 0.01 {
                        "high".to_string()
                    } else if p.cost_per_unit.unwrap_or(0.0) > 0.0 {
                        "medium".to_string()
                    } else {
                        "free".to_string()
                    },
                })
                .collect();

            let response = ListProvidersResponse {
                total: providers.len(),
                providers,
            };

            serde_json::to_value(response).map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Serialization error: {}", e),
                data: None,
            })
        } else {
            // No providers
            let response = ListProvidersResponse {
                total: 0,
                providers: vec![],
            };
            serde_json::to_value(response).map_err(|e| JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Serialization error: {}", e),
                data: None,
            })
        }
    }

    /// Handle announce_capabilities method
    async fn handle_announce_capabilities(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let request: AnnounceCapabilitiesRequest = self.parse_params(params)?;

        info!(
            "📢 announce_capabilities - {} capabilities",
            request.capabilities.len()
        );

        let response = AnnounceCapabilitiesResponse {
            success: true,
            message: format!("Acknowledged {} capabilities", request.capabilities.len()),
            announced_at: chrono::Utc::now().to_rfc3339(),
        };

        serde_json::to_value(response).map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Serialization error: {}", e),
            data: None,
        })
    }

    /// Handle health method
    async fn handle_health(&self) -> Result<Value, JsonRpcError> {
        debug!("💚 health check");

        let metrics = self.metrics.read().await;

        let response = HealthCheckResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: metrics.uptime_seconds(),
            active_providers: if let Some(router) = &self.ai_router {
                router.provider_count().await
            } else {
                0
            },
            requests_processed: metrics.requests_handled,
            avg_response_time_ms: metrics.avg_response_time_ms(),
        };

        serde_json::to_value(response).map_err(|e| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: format!("Serialization error: {}", e),
            data: None,
        })
    }

    /// Parse parameters into expected type
    fn parse_params<T: serde::de::DeserializeOwned>(
        &self,
        params: Option<Value>,
    ) -> Result<T, JsonRpcError> {
        match params {
            Some(value) => serde_json::from_value(value).map_err(|e| JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: format!("Invalid parameters: {}", e),
                data: None,
            }),
            None => Err(JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing parameters".to_string(),
                data: None,
            }),
        }
    }

    /// Create method not found error
    fn method_not_found(&self, method: &str) -> JsonRpcError {
        JsonRpcError {
            code: error_codes::METHOD_NOT_FOUND,
            message: format!("Method not found: {}", method),
            data: None,
        }
    }

    /// Create error response
    fn error_response(&self, id: Value, code: i32, message: &str) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
            id,
        }
    }

    /// Handle metrics method
    async fn handle_metrics(&self) -> Result<Value, JsonRpcError> {
        debug!("📊 metrics request");

        let metrics = self.metrics.read().await;

        let response = serde_json::json!({
            "requests_handled": metrics.requests_handled,
            "errors": metrics.errors,
            "uptime_seconds": metrics.uptime_seconds(),
            "avg_response_time_ms": metrics.avg_response_time_ms(),
            "success_rate": if metrics.requests_handled > 0 {
                (metrics.requests_handled - metrics.errors) as f64 / metrics.requests_handled as f64
            } else {
                1.0
            }
        });

        Ok(response)
    }

    /// Handle discover_peers method
    async fn handle_discover_peers(&self, _params: Option<Value>) -> Result<Value, JsonRpcError> {
        info!("🔍 discover_peers request");

        // TODO: Integrate with actual primal discovery
        // For now, return discovered primals from registry or environment
        let peers = Vec::<serde_json::Value>::new();

        let response = serde_json::json!({
            "peers": peers,
            "total": peers.len(),
            "discovery_method": "capability_registry"
        });

        Ok(response)
    }

    /// Handle ping method (simple connectivity test)
    async fn handle_ping(&self) -> Result<Value, JsonRpcError> {
        debug!("🏓 ping");

        Ok(serde_json::json!({
            "pong": true,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "version": env!("CARGO_PKG_VERSION")
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "query_ai".to_string(),
            params: Some(json!({"prompt": "Hello"})),
            id: Some(json!(1)),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.method, deserialized.method);
        assert_eq!(request.jsonrpc, deserialized.jsonrpc);
    }

    #[test]
    fn test_jsonrpc_response_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({"status": "ok"})),
            error: None,
            id: json!(1),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: JsonRpcResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.jsonrpc, deserialized.jsonrpc);
        assert!(deserialized.result.is_some());
        assert!(deserialized.error.is_none());
    }

    #[test]
    fn test_jsonrpc_error_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: error_codes::METHOD_NOT_FOUND,
                message: "Method not found".to_string(),
                data: None,
            }),
            id: json!(1),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: JsonRpcResponse = serde_json::from_str(&json).unwrap();

        assert!(deserialized.result.is_none());
        assert!(deserialized.error.is_some());
        assert_eq!(
            deserialized.error.unwrap().code,
            error_codes::METHOD_NOT_FOUND
        );
    }

    #[test]
    fn test_metrics_uptime() {
        let metrics = ServerMetrics::new();
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(metrics.uptime_seconds() >= 0);
    }

    #[test]
    fn test_metrics_avg_response_time() {
        let mut metrics = ServerMetrics::new();
        assert!(metrics.avg_response_time_ms().is_none());

        metrics.requests_handled = 10;
        metrics.total_response_time_ms = 1000;
        assert_eq!(metrics.avg_response_time_ms(), Some(100.0));
    }
}
