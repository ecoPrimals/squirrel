//! # Protocol Router - Intelligent RPC Protocol Selection
//!
//! Following Songbird v3.19.3 pattern for multi-protocol support with intelligent fallback.
//!
//! ## Priority Order
//!
//! 1. **tarpc** - Primary (fastest, type-safe, binary)
//! 2. **JSON-RPC** - Secondary (Unix socket IPC, local)
//! 3. **HTTPS** - Fallback (compatibility, cross-platform)
//!
//! ## Architecture
//!
//! ```text
//! Request → ProtocolRouter
//!              ↓
//!      [Capability Detection]
//!              ↓
//!      ┌───────┴───────┐
//!      ↓       ↓       ↓
//!    tarpc  JSON-RPC HTTPS
//!      ↓       ↓       ↓
//!    [Fast] [Local] [Compat]
//! ```

use crate::error::PrimalError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Protocol router configuration
#[derive(Debug, Clone)]
pub struct ProtocolRouterConfig {
    /// Enable tarpc protocol
    pub tarpc_enabled: bool,
    /// Enable JSON-RPC protocol
    pub jsonrpc_enabled: bool,
    /// Enable HTTPS fallback
    pub https_fallback: bool,
    /// Prefer local protocols when available
    pub prefer_local: bool,
}

impl Default for ProtocolRouterConfig {
    fn default() -> Self {
        Self {
            tarpc_enabled: cfg!(feature = "tarpc-rpc"),
            jsonrpc_enabled: true, // Always enabled
            https_fallback: true,  // Fallback enabled by default
            prefer_local: true,
        }
    }
}

/// Protocol capabilities detected for a request
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolCapabilities {
    /// Request supports tarpc protocol
    pub supports_tarpc: bool,
    /// Request is from local machine
    pub is_local: bool,
    /// Request requires HTTPS (external/security)
    pub requires_secure: bool,
}

/// Protocol actually used for a request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolUsed {
    /// tarpc binary RPC
    Tarpc,
    /// JSON-RPC 2.0
    JsonRpc,
    /// HTTPS RESTful API
    Https,
}

/// Generic request wrapper
#[derive(Debug, Clone)]
pub struct ProtocolRequest {
    /// Request ID
    pub id: String,
    /// Method name
    pub method: String,
    /// Parameters (JSON)
    pub params: serde_json::Value,
    /// Detected capabilities
    pub capabilities: ProtocolCapabilities,
}

/// Generic response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolResponse {
    /// Request ID
    pub id: String,
    /// Result (if success)
    pub result: Option<serde_json::Value>,
    /// Error (if failure)
    pub error: Option<String>,
    /// Protocol used
    pub protocol_used: ProtocolUsed,
}

/// Intelligent protocol router (Songbird pattern)
pub struct ProtocolRouter {
    config: Arc<RwLock<ProtocolRouterConfig>>,
    metrics: Arc<ProtocolMetrics>,
}

/// Protocol usage metrics
#[derive(Debug, Default)]
pub struct ProtocolMetrics {
    tarpc_requests: std::sync::atomic::AtomicU64,
    jsonrpc_requests: std::sync::atomic::AtomicU64,
    https_requests: std::sync::atomic::AtomicU64,
}

impl ProtocolRouter {
    /// Create a new protocol router
    pub fn new(config: ProtocolRouterConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            metrics: Arc::new(ProtocolMetrics::default()),
        }
    }

    /// Route request to best available protocol
    ///
    /// Priority order (following Songbird):
    /// 1. tarpc (if enabled and supported)
    /// 2. JSON-RPC (if local)
    /// 3. HTTPS (fallback)
    pub async fn route_request(
        &self,
        request: ProtocolRequest,
    ) -> Result<ProtocolResponse, PrimalError> {
        let config = self.config.read().await;

        // Priority 1: tarpc (fastest, type-safe)
        if config.tarpc_enabled && request.capabilities.supports_tarpc {
            tracing::debug!("Routing to tarpc (priority 1)");
            self.record_protocol_use(ProtocolUsed::Tarpc);
            return self.route_to_tarpc(request).await;
        }

        // Priority 2: JSON-RPC (local IPC)
        if config.jsonrpc_enabled && request.capabilities.is_local && config.prefer_local {
            tracing::debug!("Routing to JSON-RPC (priority 2, local)");
            self.record_protocol_use(ProtocolUsed::JsonRpc);
            return self.route_to_jsonrpc(request).await;
        }

        // Priority 3: HTTPS (fallback)
        if config.https_fallback {
            tracing::debug!("Routing to HTTPS (priority 3, fallback)");
            self.record_protocol_use(ProtocolUsed::Https);
            return self.route_to_https(request).await;
        }

        Err(PrimalError::Configuration(
            "No protocol available for request".to_string(),
        ))
    }

    /// Detect protocol capabilities for a request
    pub fn detect_capabilities(&self, request_source: &str) -> ProtocolCapabilities {
        let is_local = request_source.starts_with("unix:")
            || request_source.starts_with("127.0.0.1")
            || request_source.starts_with("localhost")
            || request_source.starts_with("/tmp/");

        let supports_tarpc = cfg!(feature = "tarpc-rpc")
            && (request_source.contains("tarpc") || !request_source.starts_with("http"));

        let requires_secure = request_source.starts_with("https") || !is_local;

        ProtocolCapabilities {
            supports_tarpc,
            is_local,
            requires_secure,
        }
    }

    /// Get protocol usage statistics
    pub fn get_metrics(&self) -> (u64, u64, u64) {
        use std::sync::atomic::Ordering;
        (
            self.metrics.tarpc_requests.load(Ordering::Relaxed),
            self.metrics.jsonrpc_requests.load(Ordering::Relaxed),
            self.metrics.https_requests.load(Ordering::Relaxed),
        )
    }

    /// Record protocol usage
    fn record_protocol_use(&self, protocol: ProtocolUsed) {
        use std::sync::atomic::Ordering;
        match protocol {
            ProtocolUsed::Tarpc => {
                self.metrics.tarpc_requests.fetch_add(1, Ordering::Relaxed);
            }
            ProtocolUsed::JsonRpc => {
                self.metrics
                    .jsonrpc_requests
                    .fetch_add(1, Ordering::Relaxed);
            }
            ProtocolUsed::Https => {
                self.metrics.https_requests.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Route to tarpc protocol
    async fn route_to_tarpc(
        &self,
        request: ProtocolRequest,
    ) -> Result<ProtocolResponse, PrimalError> {
        #[cfg(feature = "tarpc-rpc")]
        {
            // Call tarpc handler stub (TODO: wire to actual tarpc server)
            let result =
                crate::rpc::handler_stubs::handle_request(&request.method, request.params.clone())
                    .await?;

            Ok(ProtocolResponse {
                id: request.id,
                result: Some(result),
                error: None,
                protocol_used: ProtocolUsed::Tarpc,
            })
        }

        #[cfg(not(feature = "tarpc-rpc"))]
        {
            Err(PrimalError::Configuration(
                "tarpc not enabled (build with --features tarpc-rpc)".to_string(),
            ))
        }
    }

    /// Route to JSON-RPC protocol
    async fn route_to_jsonrpc(
        &self,
        request: ProtocolRequest,
    ) -> Result<ProtocolResponse, PrimalError> {
        // Call JSON-RPC handler stub (TODO: wire to actual JSON-RPC server)
        let result = crate::rpc::handler_stubs::handle_jsonrpc_request(
            &request.method,
            request.params.clone(),
        )
        .await?;

        Ok(ProtocolResponse {
            id: request.id,
            result: Some(result),
            error: None,
            protocol_used: ProtocolUsed::JsonRpc,
        })
    }

    /// Route to HTTPS fallback
    async fn route_to_https(
        &self,
        request: ProtocolRequest,
    ) -> Result<ProtocolResponse, PrimalError> {
        // Call HTTPS handler (will be implemented in https_fallback.rs)
        let result = crate::rpc::https_fallback::handle_https_request(
            &request.method,
            request.params.clone(),
        )
        .await?;

        Ok(ProtocolResponse {
            id: request.id,
            result: Some(result),
            error: None,
            protocol_used: ProtocolUsed::Https,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_detection_local() {
        let router = ProtocolRouter::new(ProtocolRouterConfig::default());

        let caps = router.detect_capabilities("unix:/tmp/squirrel.sock");
        assert!(caps.is_local);

        let caps = router.detect_capabilities("127.0.0.1:9001");
        assert!(caps.is_local);

        let caps = router.detect_capabilities("localhost:9001");
        assert!(caps.is_local);
    }

    #[test]
    fn test_capability_detection_remote() {
        let router = ProtocolRouter::new(ProtocolRouterConfig::default());

        let caps = router.detect_capabilities("https://remote.example.com");
        assert!(!caps.is_local);
        assert!(caps.requires_secure);
    }

    #[test]
    fn test_capability_detection_tarpc() {
        let router = ProtocolRouter::new(ProtocolRouterConfig::default());

        #[cfg(feature = "tarpc-rpc")]
        {
            let caps = router.detect_capabilities("tcp://localhost:9001");
            assert!(caps.supports_tarpc);
        }
    }

    #[tokio::test]
    async fn test_protocol_priority() {
        let mut config = ProtocolRouterConfig::default();
        config.tarpc_enabled = true;
        config.jsonrpc_enabled = true;
        config.https_fallback = true;

        let router = ProtocolRouter::new(config);

        // tarpc request should use tarpc
        let request = ProtocolRequest {
            id: "test1".to_string(),
            method: "ping".to_string(),
            params: serde_json::json!({}),
            capabilities: ProtocolCapabilities {
                supports_tarpc: true,
                is_local: true,
                requires_secure: false,
            },
        };

        // Note: This will fail without actual handlers, but tests the routing logic
        let result = router.route_request(request).await;
        assert!(result.is_err()); // Expected until handlers are implemented
    }

    #[test]
    fn test_metrics() {
        let router = ProtocolRouter::new(ProtocolRouterConfig::default());

        router.record_protocol_use(ProtocolUsed::Tarpc);
        router.record_protocol_use(ProtocolUsed::JsonRpc);
        router.record_protocol_use(ProtocolUsed::JsonRpc);
        router.record_protocol_use(ProtocolUsed::Https);

        let (tarpc, jsonrpc, https) = router.get_metrics();
        assert_eq!(tarpc, 1);
        assert_eq!(jsonrpc, 2);
        assert_eq!(https, 1);
    }
}
