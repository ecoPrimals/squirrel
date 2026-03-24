// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! # Universal Adapter V2 - Infant Primal Pattern
#![expect(dead_code, reason = "Adapter V2 infrastructure awaiting activation")]
//!
//! **Philosophy**: Like an infant, Squirrel starts with ZERO hardcoded knowledge
//! and discovers its world through universal patterns.
//!
//! Following capability-based discovery patterns:
//! - Orchestration: Agnostic service discovery (env: SERVICE_MESH_PORT)
//! - Security: Universal infant discovery (env: SECURITY_SERVICE_PORT)
//!
//! ## Core Principle
//!
//! ```text
//! ❌ OLD: N² Hardcoded Connections
//! Squirrel → ComputeClient → "http://localhost:8500"
//! Squirrel → SecurityClient → "https://localhost:8443"
//! Squirrel → OrchestrationClient → "http://localhost:9090"
//!
//! ✅ NEW: O(1) Universal Adapter
//! Squirrel → UniversalAdapterV2 → discover("compute") → ANY compute provider
//! Squirrel → UniversalAdapterV2 → discover("security") → ANY security provider
//! ```

use crate::discovery::types::{DiscoveredService, DiscoveryError, DiscoveryResult};
use crate::discovery::{PrimalSelfKnowledge, RuntimeDiscoveryEngine};
use crate::error::PrimalError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

// ============================================================================
// UNIVERSAL ADAPTER - Infant Primal Core
// ============================================================================

/// Universal adapter V2 for capability-based primal communication
///
/// This adapter enables O(1) knowledge instead of N² hardcoded connections.
/// Each primal only needs to know itself and uses this adapter to discover
/// everything else at runtime.
///
/// ## Example
///
/// ```rust,ignore
/// // Infant primal awakens with ZERO knowledge
/// let adapter = UniversalAdapterV2::awaken().await?;
///
/// // Discover compute capability (could be Toadstool, AWS Lambda, etc.)
/// let compute = adapter.connect_capability("compute").await?;
///
/// // Discover security capability (any provider exposing security)
/// let security = adapter.connect_capability("security").await?;
///
/// // NO hardcoded knowledge of specific primals!
/// ```
#[derive(Clone)]
pub struct UniversalAdapterV2 {
    /// Self-knowledge (what am I?)
    self_knowledge: Arc<PrimalSelfKnowledge>,

    /// Runtime discovery engine
    discovery: Arc<RuntimeDiscoveryEngine>,

    /// Protocol negotiator for multi-protocol support
    protocol_negotiator: Arc<ProtocolNegotiator>,

    /// Connection pool (capability → active connection)
    connections: Arc<RwLock<HashMap<String, ActiveConnection>>>,

    /// Adapter configuration
    config: AdapterConfig,
}

/// Configuration for universal adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    /// Connection timeout (ms)
    pub connection_timeout_ms: u64,

    /// Maximum retry attempts
    pub max_retries: u32,

    /// Connection pooling enabled
    pub enable_pooling: bool,

    /// Cache TTL for discovered services (seconds)
    pub discovery_cache_ttl_secs: u64,
}

impl Default for AdapterConfig {
    fn default() -> Self {
        Self {
            connection_timeout_ms: 5000,
            max_retries: 3,
            enable_pooling: true,
            discovery_cache_ttl_secs: 300, // 5 minutes
        }
    }
}

/// Active connection to a capability provider
#[derive(Debug, Clone)]
struct ActiveConnection {
    /// The discovered service
    service: DiscoveredService,

    /// Protocol being used
    protocol: Protocol,

    /// Connection metadata
    metadata: ConnectionMetadata,
}

/// Connection metadata for monitoring
#[derive(Debug, Clone)]
struct ConnectionMetadata {
    /// Number of successful requests
    successful_requests: u64,

    /// Number of failed requests
    failed_requests: u64,

    /// Average response time (ms)
    avg_response_time_ms: f64,

    /// Last used timestamp
    last_used: std::time::SystemTime,
}

/// Protocol negotiator for multi-protocol support
#[derive(Debug, Clone)]
pub struct ProtocolNegotiator {
    /// Preferred protocol order
    protocol_preference: Vec<Protocol>,
}

/// Supported protocols (in preference order)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// tarpc binary RPC (fastest, preferred)
    Tarpc,

    /// JSON-RPC over Unix socket
    JsonRpc,

    /// HTTPS REST API (fallback)
    Https,

    /// HTTP (unsecured, testing only)
    Http,
}

impl UniversalAdapterV2 {
    /// Awaken as an infant primal with ZERO hardcoded knowledge
    ///
    /// This is the entry point for infant primal pattern. The primal:
    /// 1. Discovers its own identity
    /// 2. Prepares to discover the world
    /// 3. Starts with NO knowledge of other primals
    ///
    /// # Errors
    ///
    /// Returns error if self-discovery fails
    pub async fn awaken() -> DiscoveryResult<Self> {
        info!("👶 Awakening as infant primal with ZERO hardcoded knowledge...");

        // Discover self-identity (NO hardcoding!)
        let self_knowledge = PrimalSelfKnowledge::discover_self()
            .map_err(|e| DiscoveryError::ParseError(format!("Self-discovery failed: {e}")))?;

        info!(
            "✅ Self-knowledge acquired: {}",
            self_knowledge.identity().name
        );

        // Create discovery engine
        let discovery = Arc::new(RuntimeDiscoveryEngine::new());

        // Create protocol negotiator
        let protocol_negotiator = Arc::new(ProtocolNegotiator::new());

        Ok(Self {
            self_knowledge: Arc::new(self_knowledge),
            discovery,
            protocol_negotiator,
            connections: Arc::new(RwLock::new(HashMap::new())),
            config: AdapterConfig::default(),
        })
    }

    /// Awaken with custom configuration
    pub async fn awaken_with_config(config: AdapterConfig) -> DiscoveryResult<Self> {
        let mut adapter = Self::awaken().await?;
        adapter.config = config;
        Ok(adapter)
    }

    /// Connect to a capability (NOT a specific primal!)
    ///
    /// This is the core method that replaces all hardcoded client connections.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // ❌ OLD: Hardcoded
    /// let toadstool = ToadstoolClient::connect("http://localhost:8500")?;
    ///
    /// // ✅ NEW: Capability-based
    /// let compute = adapter.connect_capability("compute").await?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if capability cannot be discovered or connection fails
    pub async fn connect_capability(&self, capability: &str) -> DiscoveryResult<UniversalClient> {
        info!("🔍 Discovering capability: {}", capability);

        // Check connection pool first
        if self.config.enable_pooling
            && let Some(active_conn) = self.get_pooled_connection(capability).await
        {
            debug!("✅ Using pooled connection for '{}'", capability);
            return Ok(UniversalClient::from_connection(active_conn));
        }

        // Discover service provider for this capability
        let service = self.discovery.discover_capability(capability).await?;

        info!(
            "✅ Discovered '{}' provider: {} at {}",
            capability, service.name, service.endpoint
        );

        // Negotiate protocol
        let protocol = self.protocol_negotiator.negotiate(&service).await?;

        debug!("🔌 Using protocol: {:?}", protocol);

        // Create connection
        let connection = ActiveConnection {
            service: service.clone(),
            protocol,
            metadata: ConnectionMetadata {
                successful_requests: 0,
                failed_requests: 0,
                avg_response_time_ms: 0.0,
                last_used: std::time::SystemTime::now(),
            },
        };

        // Pool the connection
        if self.config.enable_pooling {
            self.pool_connection(capability.to_string(), connection.clone())
                .await;
        }

        Ok(UniversalClient::from_connection(connection))
    }

    /// Get self-knowledge (who am I?)
    #[must_use]
    pub fn identity(&self) -> &PrimalSelfKnowledge {
        &self.self_knowledge
    }

    /// Get pooled connection if available and fresh
    async fn get_pooled_connection(&self, capability: &str) -> Option<ActiveConnection> {
        let connections = self.connections.read().await;
        connections.get(capability).and_then(|conn| {
            // Check if service is still fresh
            if conn.service.is_fresh(std::time::Duration::from_secs(
                self.config.discovery_cache_ttl_secs,
            )) {
                Some(conn.clone())
            } else {
                None
            }
        })
    }

    /// Pool a connection for reuse
    async fn pool_connection(&self, capability: String, connection: ActiveConnection) {
        let mut connections = self.connections.write().await;
        connections.insert(capability, connection);
    }

    /// Clear connection pool (force rediscovery)
    pub async fn clear_pool(&self) {
        self.connections.write().await.clear();
        info!("🗑️  Connection pool cleared");
    }
}

// ============================================================================
// PROTOCOL NEGOTIATOR
// ============================================================================

impl ProtocolNegotiator {
    /// Create new protocol negotiator with default preferences
    #[must_use]
    pub fn new() -> Self {
        Self {
            protocol_preference: vec![
                Protocol::Tarpc,   // Fastest, binary
                Protocol::JsonRpc, // Local IPC
                Protocol::Https,   // Secure fallback
                Protocol::Http,    // Testing only
            ],
        }
    }

    /// Negotiate protocol with a discovered service
    ///
    /// Tries protocols in preference order and returns the first supported one.
    async fn negotiate(&self, service: &DiscoveredService) -> DiscoveryResult<Protocol> {
        debug!("🤝 Negotiating protocol with {}", service.name);

        // Check service's supported protocols
        let supported_protocols = self.parse_supported_protocols(&service.endpoint);

        // Find first matching protocol in our preference order
        for preferred in &self.protocol_preference {
            if supported_protocols.contains(preferred) {
                debug!("✅ Negotiated protocol: {:?}", preferred);
                return Ok(*preferred);
            }
        }

        // Default to HTTP if endpoint is http://
        if service.endpoint.starts_with("http://") {
            Ok(Protocol::Http)
        } else if service.endpoint.starts_with("https://") {
            Ok(Protocol::Https)
        } else {
            Err(DiscoveryError::MechanismFailed {
                mechanism: "protocol_negotiation".to_string(),
                reason: "No compatible protocol found".to_string(),
            })
        }
    }

    /// Parse supported protocols from endpoint URL
    fn parse_supported_protocols(&self, endpoint: &str) -> Vec<Protocol> {
        let mut protocols = Vec::new();

        if endpoint.starts_with("tarpc://") {
            protocols.push(Protocol::Tarpc);
        }
        if endpoint.starts_with("jsonrpc+unix://")
            || std::path::Path::new(endpoint)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
        {
            protocols.push(Protocol::JsonRpc);
        }
        if endpoint.starts_with("https://") {
            protocols.push(Protocol::Https);
        }
        if endpoint.starts_with("http://") {
            protocols.push(Protocol::Http);
        }

        protocols
    }
}

impl Default for ProtocolNegotiator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// UNIVERSAL CLIENT
// ============================================================================

/// Universal client that can communicate over any protocol
///
/// This abstracts away protocol details and provides a uniform interface.
pub struct UniversalClient {
    connection: ActiveConnection,
}

impl UniversalClient {
    /// Create from active connection
    const fn from_connection(connection: ActiveConnection) -> Self {
        Self { connection }
    }

    /// Extract Unix socket path from the discovered service endpoint.
    /// Returns error if endpoint is not a Unix socket (e.g. HTTP).
    fn extract_unix_socket_path(&self) -> Result<std::path::PathBuf, PrimalError> {
        let endpoint = &self.connection.service.endpoint;
        if let Some(path) = endpoint.strip_prefix("unix://") {
            Ok(std::path::PathBuf::from(path))
        } else if let Some(path) = endpoint.strip_prefix("jsonrpc+unix://") {
            Ok(std::path::PathBuf::from(path))
        } else if std::path::Path::new(endpoint)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
        {
            Ok(std::path::PathBuf::from(endpoint))
        } else {
            Err(PrimalError::NotSupported(format!(
                "Capability '{}' at {} does not support JSON-RPC over Unix socket. \
                 HTTP/HTTPS endpoints require delegation to a primal with 'http.proxy' capability.",
                self.connection.service.name, endpoint
            )))
        }
    }

    /// Send JSON-RPC request over Unix socket and return the result value.
    async fn send_jsonrpc_over_unix(
        &self,
        socket_path: &std::path::Path,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::UnixStream;

        let mut stream = UnixStream::connect(socket_path).await.map_err(|e| {
            PrimalError::NetworkError(format!(
                "Failed to connect to Unix socket {}: {e}",
                socket_path.display()
            ))
        })?;

        let request_bytes = serde_json::to_vec(request)
            .map_err(|e| PrimalError::InvalidInput(format!("Failed to serialize: {e}")))?;

        stream
            .write_all(&request_bytes)
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to write: {e}")))?;
        stream
            .write_all(b"\n")
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to write delimiter: {e}")))?;

        let mut response_bytes = Vec::new();
        stream
            .read_to_end(&mut response_bytes)
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to read: {e}")))?;

        let json_rpc_response: serde_json::Value = serde_json::from_slice(&response_bytes)
            .map_err(|e| {
                PrimalError::SerializationError(format!("Failed to deserialize response: {e}"))
            })?;

        universal_patterns::extract_rpc_result(&json_rpc_response)
            .map_err(|rpc_err| PrimalError::RemoteError(rpc_err.to_string()))
    }

    /// Get the endpoint this client is connected to
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.connection.service.endpoint
    }

    /// Get the protocol being used
    #[must_use]
    pub const fn protocol(&self) -> Protocol {
        self.connection.protocol
    }

    /// Get service name
    #[must_use]
    pub fn service_name(&self) -> &str {
        &self.connection.service.name
    }

    /// Execute a capability request
    ///
    /// This is the universal method for invoking any capability.
    /// Routes to the discovered primal via JSON-RPC over Unix socket.
    /// Uses the adapter's discovered capabilities—no hardcoded primal knowledge.
    ///
    /// # Errors
    ///
    /// Returns error if capability endpoint is not a Unix socket, request fails,
    /// or the capability is not found.
    pub async fn execute_capability<T, R>(&self, request: T) -> Result<R, PrimalError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        tracing::debug!(
            "Executing capability on service '{}' at {} (protocol: {:?})",
            self.connection.service.name,
            self.connection.service.endpoint,
            self.connection.protocol
        );

        let socket_path = self.extract_unix_socket_path()?;

        let params = serde_json::to_value(&request)
            .map_err(|e| PrimalError::InvalidInput(format!("Failed to serialize request: {e}")))?;

        let json_rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": uuid::Uuid::new_v4().to_string(),
            "method": "execute",
            "params": params,
        });

        let response = self
            .send_jsonrpc_over_unix(&socket_path, &json_rpc_request)
            .await?;

        serde_json::from_value(response).map_err(|e| {
            PrimalError::SerializationError(format!("Failed to deserialize response: {e}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use universal_constants::network::{DEFAULT_LOCALHOST, get_service_port, http_url};

    #[tokio::test]
    async fn test_awaken_infant_primal() {
        let adapter = UniversalAdapterV2::awaken().await.expect("should succeed");
        let identity = adapter.identity().identity();

        // Should have discovered self-identity
        assert!(!identity.name.is_empty());
        assert!(!identity.capabilities.is_empty());
    }

    #[test]
    fn test_connect_capability_from_env() {
        let compute_url = http_url(DEFAULT_LOCALHOST, get_service_port("compute"), "");
        temp_env::with_var("COMPUTE_ENDPOINT", Some(compute_url.as_str()), || {
            let rt = tokio::runtime::Runtime::new().expect("should succeed");
            rt.block_on(async {
                let adapter = UniversalAdapterV2::awaken().await.expect("should succeed");
                let client = adapter.connect_capability("compute").await;

                assert!(client.is_ok());
                if let Ok(client) = client {
                    assert_eq!(client.endpoint(), compute_url);
                }
            });
        });
    }

    #[test]
    fn test_connection_pooling() {
        let storage_url = http_url(DEFAULT_LOCALHOST, get_service_port("storage"), "");
        temp_env::with_var("STORAGE_ENDPOINT", Some(storage_url.as_str()), || {
            let rt = tokio::runtime::Runtime::new().expect("should succeed");
            rt.block_on(async {
                let adapter = UniversalAdapterV2::awaken().await.expect("should succeed");

                let _client1 = adapter
                    .connect_capability("storage")
                    .await
                    .expect("should succeed");
                let _client2 = adapter
                    .connect_capability("storage")
                    .await
                    .expect("should succeed");

                let connections = adapter.connections.read().await;
                assert!(connections.contains_key("storage"));
            });
        });
    }
}
