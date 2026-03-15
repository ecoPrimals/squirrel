// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab
#![allow(deprecated)]

//! Universal Primal Ecosystem Integration
//!
//! This module implements the universal patterns for ecosystem integration,
//! replacing hard-coded integrations with a standardized approach that works
//! with any primal through capability-based service discovery.
//!
//! ## Universal Principles
//!
//! - **Capability-Based**: Services discovered and composed based on declared capabilities
//! - **Service-Agnostic**: No knowledge of specific primal implementations
//! - **Context-Aware**: Requests routed based on user, device, and security context
//! - **Multi-Instance**: Support for multiple instances of each capability type
//! - **Federation-Ready**: Designed for cross-platform sovereignty
//! - **Performance-Optimized**: Caching and connection pooling for efficient operation

// connection_pool removed - HTTP connection pooling not needed
mod types;

// Public exports
pub use types::*;

// Re-export DiscoveredPrimal for backward compatibility
pub use universal_patterns::registry::DiscoveredPrimal;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::error::PrimalError;
use crate::universal::{
    PrimalCapability, PrimalContext, PrimalRequest, PrimalResponse, UniversalResult,
};

/// Universal Primal Ecosystem Integration with Performance Optimization
///
/// This replaces hard-coded integrations with a universal capability-based
/// pattern that works with any primal through standardized interfaces.
#[derive(Debug)]
pub struct UniversalPrimalEcosystem {
    /// Service mesh endpoint (discovered dynamically)
    service_mesh_endpoint: Option<String>,
    /// Discovered services registry (by capability, not name)
    discovered_services: Arc<RwLock<HashMap<String, Vec<DiscoveredService>>>>,
    /// Service capabilities cache with performance optimization
    capabilities_cache: Arc<RwLock<HashMap<String, Vec<PrimalCapability>>>>,
    /// Capability discovery results cache for performance
    discovery_cache: Arc<RwLock<HashMap<String, CachedCapabilityMatch>>>,
    // connection_pool removed - Unix sockets don't need HTTP connection pooling
    /// Context for this ecosystem instance
    context: PrimalContext,
    /// Cache configuration
    cache_config: CacheConfig,
}

impl UniversalPrimalEcosystem {
    /// Create new universal primal ecosystem with performance optimization
    #[must_use]
    pub fn new(context: PrimalContext) -> Self {
        Self::with_cache_config(context, CacheConfig::default())
    }

    /// Create new universal primal ecosystem with custom cache configuration
    #[must_use]
    pub fn with_cache_config(context: PrimalContext, cache_config: CacheConfig) -> Self {
        Self {
            service_mesh_endpoint: None,
            discovered_services: Arc::new(RwLock::new(HashMap::new())),
            capabilities_cache: Arc::new(RwLock::new(HashMap::new())),
            discovery_cache: Arc::new(RwLock::new(HashMap::new())),
            // connection_pool removed - HTTP pooling not needed for Unix sockets
            context,
            cache_config,
        }
    }

    /// Discover service mesh (any primal that provides service-discovery capability)
    pub async fn discover_service_mesh(&mut self) -> UniversalResult<()> {
        info!("Discovering service mesh through capability-based discovery");

        // Look for any service that provides service discovery capabilities
        let capability_request = CapabilityRequest {
            required_capabilities: vec![
                "service-discovery".to_string(),
                "service-registration".to_string(),
            ],
            optional_capabilities: vec![
                "load-balancing".to_string(),
                "health-monitoring".to_string(),
            ],
            context: self.context.clone(),
            metadata: HashMap::new(),
        };

        if let Ok(matches) = self.find_services_by_capability(&capability_request).await {
            if let Some(best_match) = matches.first() {
                self.service_mesh_endpoint = Some(best_match.service.endpoint.clone());
                info!(
                    "Discovered service mesh at: {}",
                    best_match.service.endpoint
                );
            }
        }

        Ok(())
    }

    /// Find services by capability with intelligent caching
    pub async fn find_services_by_capability(
        &self,
        request: &CapabilityRequest,
    ) -> UniversalResult<Vec<CapabilityMatch>> {
        if !self.cache_config.enable_caching {
            return self.find_services_by_capability_uncached(request).await;
        }

        // Generate cache key from request
        let cache_key = self.generate_cache_key(request);

        // Check cache first
        {
            let mut cache = self.discovery_cache.write().await;
            if let Some(cached_entry) = cache.get_mut(&cache_key) {
                if cached_entry.is_valid() {
                    cached_entry.accessed();
                    debug!("Cache hit for capability discovery: {}", cache_key);
                    return Ok(cached_entry.matches.clone());
                }
                // Remove expired entry
                cache.remove(&cache_key);
                debug!("Cache expired for capability discovery: {}", cache_key);
            }
        }

        debug!("Cache miss for capability discovery: {}", cache_key);

        // Perform actual discovery
        let start_time = Instant::now();
        let matches = self.find_services_by_capability_uncached(request).await?;
        let discovery_time = start_time.elapsed();

        debug!(
            "Capability discovery completed in {:?} for: {}",
            discovery_time, cache_key
        );

        // Cache the results
        if !matches.is_empty() {
            let cached_entry = CachedCapabilityMatch {
                matches: matches.clone(),
                cached_at: Instant::now(),
                ttl_seconds: self.cache_config.capability_discovery_ttl,
                access_count: 1,
            };

            let mut cache = self.discovery_cache.write().await;

            // Implement cache eviction if at max capacity
            if cache.len() >= self.cache_config.max_cache_entries {
                self.evict_oldest_cache_entries(&mut cache).await;
            }

            cache.insert(cache_key, cached_entry);
        }

        Ok(matches)
    }

    /// Generate cache key from capability request
    fn generate_cache_key(&self, request: &CapabilityRequest) -> String {
        let mut key_parts = vec![];

        // Include required capabilities
        let mut required = request.required_capabilities.clone();
        required.sort();
        key_parts.push(format!("req:{}", required.join(",")));

        // Include optional capabilities
        let mut optional = request.optional_capabilities.clone();
        optional.sort();
        if !optional.is_empty() {
            key_parts.push(format!("opt:{}", optional.join(",")));
        }

        // Include context for context-aware caching
        key_parts.push(format!(
            "ctx:{}:{}",
            request.context.user_id, request.context.security_level
        ));

        key_parts.join("|")
    }

    /// Evict oldest cache entries to make room for new ones
    async fn evict_oldest_cache_entries(&self, cache: &mut HashMap<String, CachedCapabilityMatch>) {
        let evict_count = cache.len() / 10; // Evict 10% of entries

        // Find oldest entries by creation time
        let mut entries: Vec<_> = cache
            .iter()
            .map(|(k, v)| (k.clone(), v.cached_at))
            .collect();
        entries.sort_by_key(|(_, time)| *time);

        // Remove oldest entries
        for (key, _) in entries.into_iter().take(evict_count) {
            cache.remove(&key);
        }

        debug!("Evicted {} old cache entries", evict_count);
    }

    /// Store data using any available storage capability
    pub async fn store_data(&self, key: &str, data: &[u8]) -> UniversalResult<()> {
        let storage_request = CapabilityRequest {
            required_capabilities: vec!["data-persistence".to_string()],
            optional_capabilities: vec!["high-availability".to_string(), "encryption".to_string()],
            context: self.context.clone(),
            metadata: HashMap::new(),
        };

        let matches = self.find_services_by_capability(&storage_request).await?;

        if let Some(best_storage) = matches.first() {
            let request = PrimalRequest::new(
                "squirrel",
                &best_storage.service.service_id,
                "store",
                serde_json::json!({
                    "operation": "store",
                    "parameters": {
                        "key": key,
                        "data": STANDARD.encode(data)
                    }
                }),
                self.context.clone(),
            );

            let response = self
                .send_capability_request(&best_storage.service, request)
                .await?;

            if response.success {
                info!("Data stored successfully using capability-based storage");
                Ok(())
            } else {
                Err(PrimalError::OperationFailed(
                    response
                        .error_message
                        .unwrap_or_else(|| "Storage operation failed".to_string()),
                ))
            }
        } else {
            Err(PrimalError::OperationFailed(
                "No storage capability available in ecosystem".to_string(),
            ))
        }
    }

    /// Retrieve data using any available storage capability
    pub async fn retrieve_data(&self, key: &str) -> UniversalResult<Vec<u8>> {
        let storage_request = CapabilityRequest {
            required_capabilities: vec!["data-persistence".to_string()],
            optional_capabilities: vec!["high-performance".to_string()],
            context: self.context.clone(),
            metadata: HashMap::new(),
        };

        let matches = self.find_services_by_capability(&storage_request).await?;

        if let Some(best_storage) = matches.first() {
            let request = PrimalRequest::new(
                "squirrel",
                &best_storage.service.service_id,
                "retrieve",
                serde_json::json!({
                    "operation": "retrieve",
                    "parameters": {
                        "key": key
                    }
                }),
                self.context.clone(),
            );

            let response = self
                .send_capability_request(&best_storage.service, request)
                .await?;

            if response.success {
                let data_str = response
                    .data
                    .as_ref()
                    .and_then(|d| d.get("data"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        PrimalError::SerializationError("Missing data in response".to_string())
                    })?;

                let data = STANDARD.decode(data_str).map_err(|e| {
                    PrimalError::SerializationError(format!("Failed to decode data: {e}"))
                })?;

                Ok(data)
            } else {
                Err(PrimalError::OperationFailed(
                    response
                        .error_message
                        .unwrap_or_else(|| "Retrieve operation failed".to_string()),
                ))
            }
        } else {
            Err(PrimalError::OperationFailed(
                "No storage capability available in ecosystem".to_string(),
            ))
        }
    }

    /// Execute computation using any available compute capability
    pub async fn execute_computation(
        &self,
        computation_request: serde_json::Value,
    ) -> UniversalResult<serde_json::Value> {
        let compute_request = CapabilityRequest {
            required_capabilities: vec!["task-execution".to_string()],
            optional_capabilities: vec![
                "sandboxing".to_string(),
                "gpu-acceleration".to_string(),
                "high-performance".to_string(),
            ],
            context: self.context.clone(),
            metadata: HashMap::new(),
        };

        let matches = self.find_services_by_capability(&compute_request).await?;

        if let Some(best_compute) = matches.first() {
            let request = PrimalRequest::new(
                "squirrel",
                &best_compute.service.service_id,
                "execute",
                computation_request,
                self.context.clone(),
            );

            let response = self
                .send_capability_request(&best_compute.service, request)
                .await?;

            if response.success {
                info!("Computation executed successfully using capability-based compute");
                Ok(response.data.unwrap_or_default())
            } else {
                Err(PrimalError::OperationFailed(
                    response
                        .error_message
                        .unwrap_or_else(|| "Computation failed".to_string()),
                ))
            }
        } else {
            Err(PrimalError::OperationFailed(
                "No compute capability available in ecosystem".to_string(),
            ))
        }
    }

    /// Perform security operation using any available security capability
    pub async fn perform_security_operation(
        &self,
        operation: &str,
        payload: serde_json::Value,
    ) -> UniversalResult<serde_json::Value> {
        let security_request = CapabilityRequest {
            required_capabilities: vec![format!("security-{}", operation)],
            optional_capabilities: vec![
                "hardware-security-module".to_string(),
                "audit-logging".to_string(),
            ],
            context: self.context.clone(),
            metadata: HashMap::new(),
        };

        let matches = self.find_services_by_capability(&security_request).await?;

        if let Some(best_security) = matches.first() {
            let request = PrimalRequest::new(
                "squirrel",
                &best_security.service.service_id,
                operation,
                payload,
                self.context.clone(),
            );

            let response = self
                .send_capability_request(&best_security.service, request)
                .await?;

            if response.success {
                info!("Security operation completed successfully using capability-based security");
                Ok(response.data.unwrap_or_default())
            } else {
                Err(PrimalError::OperationFailed(
                    response
                        .error_message
                        .unwrap_or_else(|| "Security operation failed".to_string()),
                ))
            }
        } else {
            Err(PrimalError::OperationFailed(format!(
                "No security capability available for operation: {operation}"
            )))
        }
    }

    /// Send capability-based request with comprehensive resilience and observability
    async fn send_capability_request(
        &self,
        service: &DiscoveredService,
        request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse> {
        // Modern TRUE PRIMAL implementation: JSON-RPC over Unix sockets
        tracing::debug!(
            "Sending capability request to service {} at {}",
            service.service_id,
            service.endpoint
        );

        // Parse endpoint to determine transport
        if service.endpoint.starts_with("unix://") {
            // Unix socket communication (TRUE PRIMAL pattern)
            self.send_unix_socket_request(service, request).await
        } else if service.endpoint.starts_with("http://")
            || service.endpoint.starts_with("https://")
        {
            // HTTP requests must be delegated to Songbird (concentrated gap strategy)
            self.delegate_to_songbird(service, request).await
        } else {
            Err(PrimalError::InvalidEndpoint(format!(
                "Unknown endpoint protocol: {}. Expected unix:// or http(s)://",
                service.endpoint
            )))
        }
    }

    /// Send request via Unix socket (TRUE PRIMAL pattern)
    async fn send_unix_socket_request(
        &self,
        service: &DiscoveredService,
        request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::UnixStream;

        let socket_path = service
            .endpoint
            .strip_prefix("unix://")
            .ok_or_else(|| PrimalError::InvalidEndpoint(service.endpoint.clone()))?;

        // Connect to Unix socket
        let mut stream = UnixStream::connect(socket_path).await.map_err(|e| {
            PrimalError::NetworkError(format!(
                "Failed to connect to Unix socket {socket_path}: {e}"
            ))
        })?;

        // Serialize request as JSON-RPC 2.0
        let json_rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.request_id.to_string(),
            "method": request.operation,
            "params": request.payload,
        });

        let request_bytes = serde_json::to_vec(&json_rpc_request).map_err(|e| {
            PrimalError::SerializationError(format!("Failed to serialize request: {e}"))
        })?;

        // Send request
        stream
            .write_all(&request_bytes)
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to write to socket: {e}")))?;

        stream
            .write_all(b"\n")
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to write delimiter: {e}")))?;

        // Read response
        let mut response_bytes = Vec::new();
        stream
            .read_to_end(&mut response_bytes)
            .await
            .map_err(|e| PrimalError::NetworkError(format!("Failed to read from socket: {e}")))?;

        // Deserialize JSON-RPC response
        let json_rpc_response: serde_json::Value = serde_json::from_slice(&response_bytes)
            .map_err(|e| {
                PrimalError::SerializationError(format!("Failed to deserialize response: {e}"))
            })?;

        // Extract result or error
        if let Some(error) = json_rpc_response.get("error") {
            return Err(PrimalError::RemoteError(error.to_string()));
        }

        let result = json_rpc_response
            .get("result")
            .ok_or_else(|| PrimalError::InvalidResponse("Missing result field".to_string()))?;

        // Convert to PrimalResponse
        Ok(PrimalResponse {
            request_id: request.request_id,
            response_id: uuid::Uuid::new_v4(),
            status: crate::universal::ResponseStatus::Success,
            success: true,
            data: Some(result.clone()),
            payload: result.clone(),
            timestamp: chrono::Utc::now(),
            processing_time_ms: None,
            duration: None,
            error: None,
            error_message: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Delegate HTTP request to Songbird (concentrated gap strategy)
    async fn delegate_to_songbird(
        &self,
        service: &DiscoveredService,
        _request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse> {
        // TRUE PRIMAL: discover Songbird via capability, don't hardcode
        tracing::warn!(
            "HTTP request needed for {}. Delegating to Songbird via capability discovery.",
            service.service_id
        );

        // Discover Songbird's HTTP proxy capability
        // This is the concentrated gap strategy: only Songbird handles HTTP
        Err(PrimalError::NotImplemented(
            "HTTP delegation to Songbird not yet implemented. \
             TRUE PRIMAL pattern: discover 'http.proxy' capability and delegate. \
             See docs/PRIMAL_COMMUNICATION_ARCHITECTURE.md"
                .to_string(),
        ))
    }

    /// Discover all available services and their capabilities
    pub async fn discover_ecosystem_services(&mut self) -> UniversalResult<()> {
        info!("Discovering ecosystem services through capability-based discovery");

        // This would typically query a service registry or perform network discovery
        // For now, we'll implement a basic discovery mechanism
        self.discover_via_well_known_endpoints().await?;
        self.discover_via_service_mesh().await?;

        Ok(())
    }

    /// Discover services via well-known endpoints (fallback)
    ///
    /// MODERN APPROACH: Environment-based discovery with capability queries
    /// No hardcoded primal-specific ports in code.
    async fn discover_via_well_known_endpoints(&mut self) -> UniversalResult<()> {
        use universal_constants::network;

        tracing::debug!("Attempting well-known endpoint discovery as fallback");

        // CAPABILITY-BASED: Build discovery list from environment configuration
        // Users can set SERVICE_DISCOVERY_PORTS="8080,8081,8082" to customize
        let discovery_ports = if let Ok(ports_str) = std::env::var("SERVICE_DISCOVERY_PORTS") {
            tracing::info!("Using custom service discovery ports from environment");
            ports_str
                .split(',')
                .filter_map(|s| s.trim().parse::<u16>().ok())
                .collect::<Vec<_>>()
        } else {
            // Default fallback: Common service ports discovered at runtime
            vec![
                network::get_service_port("http"), // Standard HTTP
                network::get_port_from_env("SERVICE_MESH_PORT", 8080), // Service mesh
                network::get_port_from_env("COMPUTE_SERVICE_PORT", 8081), // Compute
                network::get_port_from_env("SECURITY_SERVICE_PORT", 8082), // Security
                network::get_port_from_env("STORAGE_SERVICE_PORT", 8083), // Storage
                8500,                              // Consul/service mesh default
            ]
        };

        // Determine discovery host (default to localhost, configurable)
        let discovery_host = std::env::var("SERVICE_DISCOVERY_HOST")
            .unwrap_or_else(|_| network::DEFAULT_LOCALHOST.to_string());

        tracing::debug!(
            "Scanning {} ports on {} for service capabilities",
            discovery_ports.len(),
            discovery_host
        );

        let mut discovered_count = 0;

        for port in discovery_ports {
            let endpoint = network::http_url(&discovery_host, port, "");

            // Query for capabilities (not primal names)
            if let Ok(capabilities) = self.query_service_capabilities(&endpoint).await {
                if capabilities.is_empty() {
                    tracing::debug!("Port {} responded but advertised no capabilities", port);
                    continue;
                }

                discovered_count += 1;

                let service = DiscoveredService {
                    service_id: format!("service-{discovery_host}:{port}"),
                    instance_id: format!("instance-{discovery_host}:{port}"),
                    endpoint: endpoint.clone(),
                    capabilities: capabilities.clone(),
                    health: ServiceHealth::Healthy,
                    discovered_at: Utc::now(),
                    last_health_check: Some(Utc::now()),
                };

                tracing::info!(
                    "Discovered service at {}:{} with capabilities: {:?}",
                    discovery_host,
                    port,
                    capabilities
                );

                // Index by capability (capability-based architecture)
                let mut services = self.discovered_services.write().await;
                for capability in capabilities {
                    services
                        .entry(capability.clone())
                        .or_insert_with(Vec::new)
                        .push(service.clone());
                }
            }
        }

        if discovered_count > 0 {
            tracing::info!(
                "Well-known endpoint discovery completed: {} services found",
                discovered_count
            );
        } else {
            tracing::debug!("No services discovered via well-known endpoints");
        }

        Ok(())
    }

    /// Discover services via service mesh
    async fn discover_via_service_mesh(&mut self) -> UniversalResult<()> {
        if let Some(mesh_endpoint) = &self.service_mesh_endpoint {
            // Query service mesh for available services
            debug!(
                "Discovering services through service mesh: {}",
                mesh_endpoint
            );

            // Implementation would query the service mesh discovery API
            // This is a placeholder for the actual service mesh integration
        }

        Ok(())
    }

    /// Query service capabilities from an endpoint
    ///
    /// For Unix socket endpoints, uses the capability discovery system
    /// to probe the socket and retrieve its capabilities.
    async fn query_service_capabilities(&self, endpoint: &str) -> UniversalResult<Vec<String>> {
        // Parse endpoint to extract socket path (unix:// prefix)
        if let Some(socket_path) = endpoint.strip_prefix("unix://") {
            let path = std::path::PathBuf::from(socket_path);
            match crate::capabilities::discovery::probe_socket(&path).await {
                Ok(provider) => Ok(provider.capabilities),
                Err(e) => {
                    debug!("Failed to probe socket {}: {}", socket_path, e);
                    Ok(Vec::new())
                }
            }
        } else {
            // Non-socket endpoints not supported in primal architecture
            debug!("Non-socket endpoint, skipping: {}", endpoint);
            Ok(Vec::new())
        }
    }

    /// Find services by capability without caching (internal method)
    async fn find_services_by_capability_uncached(
        &self,
        request: &CapabilityRequest,
    ) -> UniversalResult<Vec<CapabilityMatch>> {
        debug!(
            "Finding services by capability (uncached): {:?}",
            request.required_capabilities
        );

        let mut matches = Vec::new();
        let services = self.discovered_services.read().await;

        // Search through all discovered services, regardless of their "primal type"
        for capability_services in services.values() {
            for service in capability_services {
                let mut matched_capabilities = Vec::new();
                let mut missing_capabilities = Vec::new();

                // Check required capabilities
                for required_cap in &request.required_capabilities {
                    if service.capabilities.contains(required_cap) {
                        matched_capabilities.push(required_cap.clone());
                    } else {
                        missing_capabilities.push(required_cap.clone());
                    }
                }

                // If all required capabilities are met, this is a valid match
                if missing_capabilities.is_empty() {
                    // Check optional capabilities for scoring
                    let optional_matches: usize = request
                        .optional_capabilities
                        .iter()
                        .filter(|cap| service.capabilities.contains(*cap))
                        .count();

                    // Calculate score based on capability matching
                    let total_optional = request.optional_capabilities.len();
                    let score = if total_optional > 0 {
                        (optional_matches as f64) / (total_optional as f64)
                    } else {
                        1.0
                    };

                    matches.push(CapabilityMatch {
                        service: service.clone(),
                        score,
                        matched_capabilities,
                        missing_capabilities,
                    });
                }
            }
        }

        // Sort by score (best matches first)
        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(matches)
    }

    /// Get cache statistics for monitoring
    pub async fn get_cache_stats(&self) -> CacheStatistics {
        let discovery_cache = self.discovery_cache.read().await;
        let capabilities_cache = self.capabilities_cache.read().await;
        // connection_pool removed - Unix sockets don't need connection pooling
        let connection_stats = std::collections::HashMap::new();

        let mut total_access_count = 0;
        let mut valid_entries = 0;
        let mut expired_entries = 0;

        for entry in discovery_cache.values() {
            total_access_count += entry.access_count;
            if entry.is_valid() {
                valid_entries += 1;
            } else {
                expired_entries += 1;
            }
        }

        CacheStatistics {
            discovery_cache_size: discovery_cache.len(),
            capabilities_cache_size: capabilities_cache.len(),
            total_cache_hits: total_access_count,
            valid_cache_entries: valid_entries,
            expired_cache_entries: expired_entries,
            connection_pool_stats: connection_stats,
        }
    }

    /// Clear all caches (useful for testing or forced refresh)
    pub async fn clear_caches(&self) {
        let mut discovery_cache = self.discovery_cache.write().await;
        let mut capabilities_cache = self.capabilities_cache.write().await;

        discovery_cache.clear();
        capabilities_cache.clear();

        info!("All caches cleared");
    }

    /// Find services by capability (backward compatibility alias)
    pub async fn find_by_capability(
        &self,
        capability: &str,
    ) -> UniversalResult<Vec<CapabilityMatch>> {
        let request = CapabilityRequest {
            required_capabilities: vec![capability.to_string()],
            optional_capabilities: vec![],
            context: PrimalContext::default(),
            metadata: HashMap::new(),
        };
        self.find_services_by_capability(&request).await
    }

    /// Initialize method for backward compatibility
    pub async fn initialize(&mut self) -> UniversalResult<()> {
        // Initialization logic here if needed
        Ok(())
    }

    /// Get discovered primals method for backward compatibility
    pub async fn get_discovered_primals(&self) -> Vec<DiscoveredPrimal> {
        // Implementation would convert DiscoveredService to DiscoveredPrimal
        vec![]
    }

    /// Match capabilities method for backward compatibility
    pub async fn match_capabilities(
        &self,
        request: &CapabilityRequest,
    ) -> UniversalResult<Vec<CapabilityMatch>> {
        self.find_services_by_capability(request).await
    }

    /// Send to primal method for backward compatibility
    pub async fn send_to_primal(
        &self,
        primal_id: &str,
        request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse> {
        debug!("Sending request to primal: {}", primal_id);
        // For now, return a default response
        let response_data =
            serde_json::json!({"status": "success", "message": "Request processed"});

        Ok(PrimalResponse {
            response_id: uuid::Uuid::new_v4(),
            request_id: uuid::Uuid::parse_str(&request.request_id.to_string()).unwrap_or_else(
                |e| {
                    warn!(
                        "Failed to parse request UUID '{}': {}, generating new UUID",
                        request.request_id, e
                    );
                    uuid::Uuid::new_v4()
                },
            ),
            payload: response_data.clone(),
            metadata: std::collections::HashMap::new(),
            data: Some(response_data),
            success: true,
            error_message: None,
            error: None,
            timestamp: chrono::Utc::now(),
            processing_time_ms: Some(100),
            duration: Some("100ms".to_string()),
            status: crate::universal::ResponseStatus::Success,
        })
    }
}
