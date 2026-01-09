//! JSON-RPC Server Implementation
//!
//! This module provides the JSON-RPC 2.0 server that listens on a Unix socket.
//! It handles incoming requests and routes them to the appropriate handlers.

use super::{handlers::RpcHandlers, types::*};
use crate::api::ai::AiRouter;
use crate::error::PrimalError;
use std::path::Path;
use std::sync::Arc;
use tokio::net::UnixListener;
use tracing::{error, info, warn};

/// RPC server that listens on Unix socket
pub struct RpcServer {
    /// Unix socket path
    socket_path: String,

    /// Request handlers
    handlers: Arc<RpcHandlers>,
}

impl Clone for RpcServer {
    fn clone(&self) -> Self {
        Self {
            socket_path: self.socket_path.clone(),
            handlers: self.handlers.clone(),
        }
    }
}

impl RpcServer {
    /// Create a new RPC server
    ///
    /// # Arguments
    ///
    /// * `node_id` - Unique identifier for this Squirrel instance
    pub fn new(node_id: &str) -> Self {
        let socket_path = format!("/tmp/squirrel-{}.sock", node_id);

        Self {
            socket_path,
            handlers: Arc::new(RpcHandlers::new()),
        }
    }

    /// Create a new RPC server with AI router
    ///
    /// # Arguments
    ///
    /// * `node_id` - Unique identifier for this Squirrel instance
    /// * `ai_router` - AI router instance
    pub fn with_ai_router(node_id: &str, ai_router: Arc<AiRouter>) -> Self {
        let socket_path = format!("/tmp/squirrel-{}.sock", node_id);

        Self {
            socket_path,
            handlers: Arc::new(RpcHandlers::with_ai_router(ai_router)),
        }
    }

    /// Create a new RPC server with custom socket path
    pub fn with_socket_path(socket_path: String) -> Self {
        Self {
            socket_path,
            handlers: Arc::new(RpcHandlers::new()),
        }
    }

    /// Start the RPC server
    ///
    /// This will:
    /// 1. Remove existing socket file if present
    /// 2. Bind to the Unix socket
    /// 3. Listen for incoming connections
    /// 4. Process JSON-RPC 2.0 requests
    pub async fn start(&self) -> Result<(), PrimalError> {
        // Remove existing socket if present
        if Path::new(&self.socket_path).exists() {
            info!("🧹 Removing existing socket: {}", self.socket_path);
            std::fs::remove_file(&self.socket_path).map_err(|e| {
                PrimalError::NetworkError(format!("Failed to remove socket: {}", e))
            })?;
        }

        // Bind to Unix socket
        let listener = UnixListener::bind(&self.socket_path)
            .map_err(|e| PrimalError::NetworkError(format!("Failed to bind Unix socket: {}", e)))?;

        info!("🚀 JSON-RPC server listening on: {}", self.socket_path);
        info!("📡 Ready for biomeOS integration");

        // Accept connections in a loop
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let handlers = self.handlers.clone();

                    // Spawn a task to handle this connection
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(stream, handlers).await {
                            error!("❌ Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("❌ Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handle a single connection
    async fn handle_connection(
        stream: tokio::net::UnixStream,
        handlers: Arc<RpcHandlers>,
    ) -> Result<(), PrimalError> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();

            // Read one line (JSON-RPC request)
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    // Connection closed
                    break;
                }
                Ok(_) => {
                    // Process the request
                    let response = Self::process_request(&line, &handlers).await;

                    // Write response
                    writer
                        .write_all(response.as_bytes())
                        .await
                        .map_err(|e| PrimalError::NetworkError(format!("Write error: {}", e)))?;

                    writer
                        .write_all(b"\n")
                        .await
                        .map_err(|e| PrimalError::NetworkError(format!("Write error: {}", e)))?;

                    writer
                        .flush()
                        .await
                        .map_err(|e| PrimalError::NetworkError(format!("Flush error: {}", e)))?;
                }
                Err(e) => {
                    error!("❌ Read error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Process a single JSON-RPC request
    async fn process_request(request_str: &str, handlers: &RpcHandlers) -> String {
        // Parse JSON-RPC request
        let request: serde_json::Value = match serde_json::from_str(request_str) {
            Ok(v) => v,
            Err(e) => {
                return Self::error_response(None, -32700, format!("Parse error: {}", e));
            }
        };

        // Validate JSON-RPC 2.0 format
        if request.get("jsonrpc") != Some(&serde_json::Value::String("2.0".to_string())) {
            return Self::error_response(
                request.get("id").cloned(),
                -32600,
                "Invalid Request: jsonrpc version must be 2.0".to_string(),
            );
        }

        let method = match request.get("method").and_then(|v| v.as_str()) {
            Some(m) => m,
            None => {
                return Self::error_response(
                    request.get("id").cloned(),
                    -32600,
                    "Invalid Request: missing method".to_string(),
                );
            }
        };

        let params = request
            .get("params")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let id = request.get("id").cloned();

        // Route to appropriate handler
        let result = match method {
            "query_ai" => Self::handle_query_ai(params, handlers).await,
            "list_providers" => Self::handle_list_providers(params, handlers).await,
            "announce_capabilities" => Self::handle_announce_capabilities(params, handlers).await,
            "health_check" => Self::handle_health_check(params, handlers).await,
            _ => Err(format!("Method not found: {}", method)),
        };

        // Build response
        match result {
            Ok(value) => Self::success_response(id, value),
            Err(error_msg) => Self::error_response(id, -32603, error_msg),
        }
    }

    /// Handle query_ai method
    async fn handle_query_ai(
        params: serde_json::Value,
        handlers: &RpcHandlers,
    ) -> Result<serde_json::Value, String> {
        let request: QueryAiRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {}", e))?;

        let response = handlers
            .handle_query_ai(request)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_value(response).map_err(|e| format!("Serialization error: {}", e))
    }

    /// Handle list_providers method
    async fn handle_list_providers(
        params: serde_json::Value,
        handlers: &RpcHandlers,
    ) -> Result<serde_json::Value, String> {
        let request: ListProvidersRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {}", e))?;

        let response = handlers
            .handle_list_providers(request)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_value(response).map_err(|e| format!("Serialization error: {}", e))
    }

    /// Handle announce_capabilities method
    async fn handle_announce_capabilities(
        params: serde_json::Value,
        handlers: &RpcHandlers,
    ) -> Result<serde_json::Value, String> {
        let request: AnnounceCapabilitiesRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {}", e))?;

        let response = handlers
            .handle_announce_capabilities(request)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_value(response).map_err(|e| format!("Serialization error: {}", e))
    }

    /// Handle health_check method
    async fn handle_health_check(
        _params: serde_json::Value,
        handlers: &RpcHandlers,
    ) -> Result<serde_json::Value, String> {
        let response = handlers
            .handle_health_check(HealthCheckRequest {})
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_value(response).map_err(|e| format!("Serialization error: {}", e))
    }

    /// Build a JSON-RPC success response
    fn success_response(id: Option<serde_json::Value>, result: serde_json::Value) -> String {
        serde_json::json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": id
        })
        .to_string()
    }

    /// Build a JSON-RPC error response
    fn error_response(id: Option<serde_json::Value>, code: i32, message: String) -> String {
        serde_json::json!({
            "jsonrpc": "2.0",
            "error": {
                "code": code,
                "message": message
            },
            "id": id
        })
        .to_string()
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = RpcServer::new("test-node");
        assert_eq!(server.socket_path(), "/tmp/squirrel-test-node.sock");
    }

    #[test]
    fn test_custom_socket_path() {
        let server = RpcServer::with_socket_path("/tmp/custom.sock".to_string());
        assert_eq!(server.socket_path(), "/tmp/custom.sock");
    }

    #[test]
    fn test_success_response() {
        let response = RpcServer::success_response(
            Some(serde_json::Value::Number(1.into())),
            serde_json::json!({"status": "ok"}),
        );

        let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["result"]["status"], "ok");
        assert_eq!(parsed["id"], 1);
    }

    #[test]
    fn test_error_response() {
        let response = RpcServer::error_response(
            Some(serde_json::Value::Number(1.into())),
            -32600,
            "Invalid Request".to_string(),
        );

        let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["error"]["code"], -32600);
        assert_eq!(parsed["error"]["message"], "Invalid Request");
    }
}
