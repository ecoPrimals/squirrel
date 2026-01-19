//! # 🌟 Universal Adapter V2 - Infant Primal Pattern
//!
//! **Philosophy**: Like an infant, Squirrel starts with ZERO hardcoded knowledge
//! and discovers its world through universal patterns.
//!
//! Following patterns from:
//! - Songbird: Agnostic service discovery
//! - BearDog: Universal infant discovery
//!
//! ## Core Principle
//!
//! ```text
//! ❌ OLD: N² Hardcoded Connections
//! Squirrel → ToadstoolClient → "http://localhost:8500"
//! Squirrel → BearDogClient → "https://localhost:7443"
//! Squirrel → SongbirdClient → "http://localhost:9090"
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
/// // Discover security capability (could be BearDog, Vault, etc.)
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
            .map_err(|e| DiscoveryError::ParseError(format!("Self-discovery failed: {}", e)))?;

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
        if self.config.enable_pooling {
            if let Some(active_conn) = self.get_pooled_connection(capability).await {
                debug!("✅ Using pooled connection for '{}'", capability);
                return Ok(UniversalClient::from_connection(active_conn));
            }
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
        if endpoint.starts_with("jsonrpc+unix://") || endpoint.ends_with(".sock") {
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
    fn from_connection(connection: ActiveConnection) -> Self {
        Self { connection }
    }

    /// Get the endpoint this client is connected to
    pub fn endpoint(&self) -> &str {
        &self.connection.service.endpoint
    }

    /// Get the protocol being used
    pub fn protocol(&self) -> Protocol {
        self.connection.protocol
    }

    /// Get service name
    pub fn service_name(&self) -> &str {
        &self.connection.service.name
    }

    /// Execute a capability request
    ///
    /// This is the universal method for invoking any capability.
    /// It uses the protocol router to intelligently select the best
    /// transport protocol (tarpc, JSON-RPC, or HTTPS).
    ///
    /// # Errors
    ///
    /// Returns error if request fails or protocol negotiation fails
    pub async fn execute_capability<T, R>(&self, request: T) -> Result<R, PrimalError>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        // protocol_router DELETED - use JSON-RPC or tarpc directly
        // use crate::rpc::protocol_router::{
        //     ProtocolCapabilities, ProtocolRequest, ProtocolRouter, ProtocolRouterConfig,
        // };

        tracing::debug!(
            "Executing capability on service '{}' at {} (protocol: {:?})",
            self.connection.service.name,
            self.connection.service.endpoint,
            self.connection.protocol
        );

        // Serialize request to JSON for protocol-agnostic transport
        let params = serde_json::to_value(&request).map_err(|e| {
            PrimalError::InvalidInput(format!("Failed to serialize request: {}", e))
        })?;

        // Protocol capabilities REMOVED - use Unix sockets directly
        // Modern implementation: JSON-RPC over Unix sockets
        // let capabilities = ProtocolCapabilities {
        //     supports_tarpc: self.connection.protocol == Protocol::Tarpc,
        //     is_local: ...,
        //     requires_secure: ...,
        // };

        // Protocol router REMOVED - modern TRUE PRIMAL uses JSON-RPC over Unix sockets
        let _ = params; // Silence unused warning
        Err(PrimalError::NotSupported(
            "Protocol routing removed. TRUE PRIMAL architecture uses JSON-RPC over Unix sockets. \
             See docs/PRIMAL_COMMUNICATION_ARCHITECTURE.md for modern patterns."
                .to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_awaken_infant_primal() {
        let adapter = UniversalAdapterV2::awaken().await.unwrap();
        let identity = adapter.identity().identity();

        // Should have discovered self-identity
        assert!(!identity.name.is_empty());
        assert!(!identity.capabilities.is_empty());
    }

    #[tokio::test]
    async fn test_connect_capability_from_env() {
        // Set up test environment
        std::env::set_var("COMPUTE_ENDPOINT", "http://localhost:8500");

        let adapter = UniversalAdapterV2::awaken().await.unwrap();
        let client = adapter.connect_capability("compute").await;

        // Should discover from environment
        assert!(client.is_ok());

        if let Ok(client) = client {
            assert_eq!(client.endpoint(), "http://localhost:8500");
        }

        // Cleanup
        std::env::remove_var("COMPUTE_ENDPOINT");
    }

    #[tokio::test]
    async fn test_connection_pooling() {
        std::env::set_var("STORAGE_ENDPOINT", "http://localhost:8080");

        let adapter = UniversalAdapterV2::awaken().await.unwrap();

        // First connection
        let _client1 = adapter.connect_capability("storage").await.unwrap();

        // Second connection should use pool
        let _client2 = adapter.connect_capability("storage").await.unwrap();

        // Verify pool has the connection
        let connections = adapter.connections.read().await;
        assert!(connections.contains_key("storage"));

        std::env::remove_var("STORAGE_ENDPOINT");
    }
}
