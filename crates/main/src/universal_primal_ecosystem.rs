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

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::error::PrimalError;
use crate::universal::{
    PrimalCapability, PrimalContext, PrimalDependency, PrimalHealth, PrimalRequest, PrimalResponse,
    PrimalType, UniversalResult,
};

// Re-export DiscoveredPrimal for backward compatibility
pub use universal_patterns::registry::DiscoveredPrimal;

/// Performance-optimized cache entry for capability discovery results
#[derive(Debug, Clone)]
pub struct CachedCapabilityMatch {
    /// The cached matches
    pub matches: Vec<CapabilityMatch>,
    /// When this cache entry was created
    pub cached_at: Instant,
    /// TTL for this cache entry (in seconds)
    pub ttl_seconds: u64,
    /// Number of times this cache entry has been accessed
    pub access_count: u64,
}

impl CachedCapabilityMatch {
    /// Check if this cache entry is still valid
    pub fn is_valid(&self) -> bool {
        self.cached_at.elapsed().as_secs() < self.ttl_seconds
    }

    /// Update access statistics
    pub fn accessed(&mut self) {
        self.access_count += 1;
    }
}

/// Connection pool for efficient HTTP client management
#[derive(Debug)]
pub struct ServiceConnectionPool {
    /// Per-endpoint HTTP clients
    clients: Arc<RwLock<HashMap<String, reqwest::Client>>>,
    /// Connection statistics
    stats: Arc<RwLock<HashMap<String, ConnectionStats>>>,
}

/// Statistics for service connections
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    /// Number of requests made to this endpoint
    pub request_count: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Last successful request timestamp
    pub last_success: Option<Instant>,
    /// Number of failures
    pub failure_count: u64,
}

impl ServiceConnectionPool {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create an HTTP client for the given endpoint with resource management
    pub async fn get_client(&self, endpoint: &str) -> reqwest::Client {
        let mut clients = self.clients.write().await;

        if let Some(client) = clients.get(endpoint) {
            // Update last used time in stats
            if let Some(stats) = self.stats.read().await.get(endpoint) {
                // Client exists and was recently used, return it
                client.clone()
            } else {
                // Client exists but no recent stats, still return it but log
                tracing::debug!("Returning cached client for {} without recent stats", endpoint);
                client.clone()
            }
        } else {
            // Create new client with production-safe configuration
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(30))                    // Request timeout
                .connect_timeout(Duration::from_secs(10))            // Connection timeout
                .pool_max_idle_per_host(5)                          // Reduced from 10 to prevent resource exhaustion
                .pool_idle_timeout(Duration::from_secs(30))         // Reduced from 90s for faster cleanup
                .tcp_keepalive(Duration::from_secs(60))             // TCP keepalive for connection health
                .tcp_nodelay(true)                                  // Reduce latency
                .http2_keep_alive_interval(Duration::from_secs(30)) // HTTP/2 keepalive
                .http2_keep_alive_timeout(Duration::from_secs(10))  // HTTP/2 keepalive timeout
                .build()
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to create optimized client for {}: {}, using default", endpoint, e);
                    reqwest::Client::new()
                });

            tracing::info!(
                endpoint = %endpoint,
                operation = "http_client_created",
                "Created new HTTP client with resource management settings"
            );

            clients.insert(endpoint.to_string(), client.clone());
            client
        }
    }

    /// Record request statistics with connection health tracking
    pub async fn record_request(&self, endpoint: &str, response_time_ms: f64, success: bool) {
        let mut stats = self.stats.write().await;
        let entry = stats
            .entry(endpoint.to_string())
            .or_insert_with(|| ConnectionStats {
                request_count: 0,
                avg_response_time_ms: 0.0,
                last_success: None,
                failure_count: 0,
            });

        entry.request_count += 1;

        if success {
            entry.last_success = Some(Instant::now());
            // Update running average with numerical stability
            let weight = (entry.request_count - 1) as f64 / entry.request_count as f64;
            entry.avg_response_time_ms = entry.avg_response_time_ms * weight + 
                response_time_ms / entry.request_count as f64;
        } else {
            entry.failure_count += 1;
            
            // Log connection health issues
            let failure_rate = entry.failure_count as f64 / entry.request_count as f64;
            if failure_rate > 0.1 { // More than 10% failure rate
                tracing::warn!(
                    endpoint = %endpoint,
                    failure_count = entry.failure_count,
                    total_requests = entry.request_count,
                    failure_rate = %format!("{:.1}%", failure_rate * 100.0),
                    operation = "connection_health_warning",
                    "High failure rate detected for endpoint"
                );
            }
        }
    }

    /// Get connection statistics for debugging/monitoring
    pub async fn get_stats(&self) -> HashMap<String, ConnectionStats> {
        self.stats.read().await.clone()
    }
    
    /// Cleanup stale connections and perform resource maintenance
    pub async fn cleanup_stale_connections(&self) {
        let now = Instant::now();
        let stale_threshold = Duration::from_secs(300); // 5 minutes without successful requests
        
        let mut clients_to_remove = Vec::new();
        let mut total_cleaned = 0;
        
        // Identify stale connections
        {
            let stats = self.stats.read().await;
            for (endpoint, stat) in stats.iter() {
                if let Some(last_success) = stat.last_success {
                    if now.duration_since(last_success) > stale_threshold {
                        clients_to_remove.push(endpoint.clone());
                    }
                } else if stat.request_count > 0 {
                    // No successful requests at all, but some attempts - remove
                    clients_to_remove.push(endpoint.clone());
                }
            }
        }
        
        // Remove stale clients and their stats
        if !clients_to_remove.is_empty() {
            let mut clients = self.clients.write().await;
            let mut stats = self.stats.write().await;
            
            for endpoint in &clients_to_remove {
                if clients.remove(endpoint).is_some() {
                    total_cleaned += 1;
                }
                stats.remove(endpoint);
                
                tracing::info!(
                    endpoint = %endpoint,
                    operation = "stale_connection_cleanup",
                    "Removed stale HTTP client and stats"
                );
            }
        }
        
        if total_cleaned > 0 {
            tracing::info!(
                cleaned_connections = total_cleaned,
                remaining_connections = self.clients.read().await.len(),
                operation = "connection_pool_maintenance",
                "Completed connection pool cleanup"
            );
        }
    }
    
    /// Get connection pool health metrics
    pub async fn get_health_metrics(&self) -> ConnectionPoolHealthMetrics {
        let clients = self.clients.read().await;
        let stats = self.stats.read().await;
        
        let mut total_requests = 0;
        let mut total_failures = 0;
        let mut healthy_connections = 0;
        let mut unhealthy_connections = 0;
        let mut avg_response_time = 0.0;
        
        for (endpoint, stat) in stats.iter() {
            total_requests += stat.request_count;
            total_failures += stat.failure_count;
            
            let failure_rate = if stat.request_count > 0 {
                stat.failure_count as f64 / stat.request_count as f64
            } else {
                0.0
            };
            
            if failure_rate < 0.05 && stat.avg_response_time_ms < 5000.0 {
                healthy_connections += 1;
            } else {
                unhealthy_connections += 1;
            }
            
            avg_response_time += stat.avg_response_time_ms;
        }
        
        if !stats.is_empty() {
            avg_response_time /= stats.len() as f64;
        }
        
        ConnectionPoolHealthMetrics {
            total_connections: clients.len(),
            healthy_connections,
            unhealthy_connections,
            total_requests,
            total_failures,
            overall_failure_rate: if total_requests > 0 {
                total_failures as f64 / total_requests as f64
            } else {
                0.0
            },
            avg_response_time_ms: avg_response_time,
        }
    }
    
    /// Graceful shutdown - close all connections and cleanup resources
    pub async fn shutdown(&self) {
        tracing::info!(
            operation = "connection_pool_shutdown",
            "Starting graceful shutdown of connection pool"
        );
        
        let start_time = Instant::now();
        
        // Clear all clients (reqwest clients will be dropped and connections closed)
        {
            let mut clients = self.clients.write().await;
            let connection_count = clients.len();
            clients.clear();
            
            tracing::info!(
                closed_connections = connection_count,
                operation = "connections_closed",
                "Closed all HTTP client connections"
            );
        }
        
        // Clear all stats
        {
            let mut stats = self.stats.write().await;
            stats.clear();
        }
        
        let shutdown_duration = start_time.elapsed();
        tracing::info!(
            shutdown_duration_ms = shutdown_duration.as_millis(),
            operation = "connection_pool_shutdown_complete",
            "Connection pool shutdown completed"
        );
    }
}

/// Connection pool health metrics for monitoring
#[derive(Debug, Clone)]
pub struct ConnectionPoolHealthMetrics {
    pub total_connections: usize,
    pub healthy_connections: usize,
    pub unhealthy_connections: usize,
    pub total_requests: u64,
    pub total_failures: u64,
    pub overall_failure_rate: f64,
    pub avg_response_time_ms: f64,
}

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
    /// Connection pool for efficient HTTP client management
    connection_pool: ServiceConnectionPool,
    /// Context for this ecosystem instance
    context: PrimalContext,
    /// Cache configuration
    cache_config: CacheConfig,
}

/// Cache configuration for performance optimization
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Default TTL for capability discovery cache (seconds)
    pub capability_discovery_ttl: u64,
    /// Default TTL for service capabilities cache (seconds)
    pub service_capabilities_ttl: u64,
    /// Maximum cache entries to maintain
    pub max_cache_entries: usize,
    /// Enable/disable caching
    pub enable_caching: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            capability_discovery_ttl: 300, // 5 minutes
            service_capabilities_ttl: 600, // 10 minutes
            max_cache_entries: 1000,
            enable_caching: true,
        }
    }
}

/// Discovered service information (capability-based, not name-based)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService {
    /// Service identifier (not primal name)
    pub service_id: String,
    /// Instance identifier
    pub instance_id: String,
    /// Service endpoint
    pub endpoint: String,
    /// Available capabilities
    pub capabilities: Vec<String>,
    /// Service health status
    pub health: ServiceHealth,
    /// Discovery timestamp
    pub discovered_at: DateTime<Utc>,
    /// Last health check
    pub last_health_check: Option<DateTime<Utc>>,
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Universal capability request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequest {
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Optional capabilities
    pub optional_capabilities: Vec<String>,
    /// Context for capability matching
    pub context: PrimalContext,
    /// Request metadata
    pub metadata: HashMap<String, String>,
}

/// Capability matching result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMatch {
    /// Service that can fulfill the capability
    pub service: DiscoveredService,
    /// Matching score (0.0 to 1.0)
    pub score: f64,
    /// Matched capabilities
    pub matched_capabilities: Vec<String>,
    /// Missing capabilities
    pub missing_capabilities: Vec<String>,
}

/// Cache statistics for monitoring
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    /// Size of the capability discovery cache
    pub discovery_cache_size: usize,
    /// Size of the service capabilities cache
    pub capabilities_cache_size: usize,
    /// Total number of cache hits across all caches
    pub total_cache_hits: u64,
    /// Number of valid entries in the capability discovery cache
    pub valid_cache_entries: usize,
    /// Number of expired entries in the capability discovery cache
    pub expired_cache_entries: usize,
    /// Connection pool statistics
    pub connection_pool_stats: HashMap<String, ConnectionStats>,
}

impl UniversalPrimalEcosystem {
    /// Create new universal primal ecosystem with performance optimization
    pub fn new(context: PrimalContext) -> Self {
        Self::with_cache_config(context, CacheConfig::default())
    }

    /// Create new universal primal ecosystem with custom cache configuration
    pub fn with_cache_config(context: PrimalContext, cache_config: CacheConfig) -> Self {
        Self {
            service_mesh_endpoint: None,
            discovered_services: Arc::new(RwLock::new(HashMap::new())),
            capabilities_cache: Arc::new(RwLock::new(HashMap::new())),
            discovery_cache: Arc::new(RwLock::new(HashMap::new())),
            connection_pool: ServiceConnectionPool::new(),
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
                } else {
                    // Remove expired entry
                    cache.remove(&cache_key);
                    debug!("Cache expired for capability discovery: {}", cache_key);
                }
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
                    .get("data")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        PrimalError::SerializationError("Missing data in response".to_string())
                    })?;

                let data = STANDARD.decode(data_str).map_err(|e| {
                    PrimalError::SerializationError(format!("Failed to decode data: {}", e))
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
                Ok(response.data)
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
                Ok(response.data)
            } else {
                Err(PrimalError::OperationFailed(
                    response
                        .error_message
                        .unwrap_or_else(|| "Security operation failed".to_string()),
                ))
            }
        } else {
            Err(PrimalError::OperationFailed(format!(
                "No security capability available for operation: {}",
                operation
            )))
        }
    }

    /// Send capability-based request with comprehensive resilience and observability
    async fn send_capability_request(
        &self,
        service: &DiscoveredService,
        request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse> {
        use crate::error_handling::safe_operations::SafeOps;
        use uuid::Uuid;
        
        // Generate correlation ID for tracking
        let correlation_id = Uuid::new_v4().to_string();
        let operation_start = Instant::now();
        
        tracing::info!(
            correlation_id = %correlation_id,
            service_id = %service.service_id,
            service_endpoint = %service.endpoint,
            request_id = %request.request_id,
            operation = "capability_request_start",
            "Starting capability-based request to service"
        );
        
        // Resilience configuration
        let max_retries = 3;
        let per_attempt_timeout = Duration::from_secs(10);
        let base_delay = Duration::from_millis(1000);
        
        let mut last_error = None;
        
        for attempt in 1..=max_retries {
            let attempt_start = Instant::now();
            let client = self.connection_pool.get_client(&service.endpoint).await;
            let request_url = format!("{}/api/v1/primal", service.endpoint);
            
            tracing::debug!(
                correlation_id = %correlation_id,
                service_id = %service.service_id,
                attempt = attempt,
                max_retries = max_retries,
                timeout_ms = per_attempt_timeout.as_millis(),
                url = %request_url,
                operation = "capability_request_attempt",
                "Attempting capability request"
            );
            
            let request_result = SafeOps::safe_with_timeout(
                per_attempt_timeout,
                || async {
                    client
                        .post(&request_url)
                        .json(&request)
                        .send()
                        .await
                },
                &format!("capability_request_attempt_{}_{}", service.service_id, attempt),
            ).await;
            
            let attempt_duration = attempt_start.elapsed();
            let response_time_ms = attempt_duration.as_millis() as f64;
            
            match request_result.execute_without_default() {
                Ok(Ok(http_response)) => {
                    let http_status = http_response.status();
                    if http_status.is_success() {
                        // Parse response safely with timeout
                        let parse_start = Instant::now();
                        let parse_result = SafeOps::safe_with_timeout(
                            Duration::from_secs(5),
                            || http_response.json::<PrimalResponse>(),
                            &format!("capability_response_parsing_{}", service.service_id),
                        ).await;
                        
                        let parse_duration = parse_start.elapsed();
                        
                        match parse_result.execute_without_default() {
                            Ok(Ok(primal_response)) => {
                                let total_duration = operation_start.elapsed();
                                
                                // Record successful request with detailed metrics
                                self.connection_pool
                                    .record_request(&service.endpoint, response_time_ms, true)
                                    .await;
                                    
                                tracing::info!(
                                    correlation_id = %correlation_id,
                                    service_id = %service.service_id,
                                    request_id = %request.request_id,
                                    response_id = %primal_response.response_id,
                                    attempt = attempt,
                                    operation = "capability_request_success",
                                    total_duration_ms = total_duration.as_millis(),
                                    attempt_duration_ms = attempt_duration.as_millis(),
                                    parse_duration_ms = parse_duration.as_millis(),
                                    http_status = http_status.as_u16(),
                                    success = primal_response.success,
                                    "Capability request completed successfully"
                                );
                                
                                return Ok(primal_response);
                            }
                            Ok(Err(parse_error)) => {
                                let error_msg = format!("Response parsing failed: {}", parse_error);
                                last_error = Some(error_msg.clone());
                                
                                tracing::warn!(
                                    correlation_id = %correlation_id,
                                    service_id = %service.service_id,
                                    attempt = attempt,
                                    operation = "capability_request_parse_error",
                                    attempt_duration_ms = attempt_duration.as_millis(),
                                    parse_duration_ms = parse_duration.as_millis(),
                                    error = %error_msg,
                                    "Response parsing failed"
                                );
                                
                                self.connection_pool
                                    .record_request(&service.endpoint, response_time_ms, false)
                                    .await;
                            }
                            Err(timeout_error) => {
                                let error_msg = format!("Response parsing timed out: {}", timeout_error);
                                last_error = Some(error_msg.clone());
                                
                                tracing::warn!(
                                    correlation_id = %correlation_id,
                                    service_id = %service.service_id,
                                    attempt = attempt,
                                    operation = "capability_request_parse_timeout",
                                    attempt_duration_ms = attempt_duration.as_millis(),
                                    timeout_ms = 5000,
                                    error = %error_msg,
                                    "Response parsing timed out"
                                );
                                
                                self.connection_pool
                                    .record_request(&service.endpoint, response_time_ms, false)
                                    .await;
                            }
                        }
                    } else {
                        let status_code = http_response.status().as_u16();
                        let error_msg = format!("HTTP error: {}", http_response.status());
                        last_error = Some(error_msg.clone());
                        
                        tracing::warn!(
                            correlation_id = %correlation_id,
                            service_id = %service.service_id,
                            attempt = attempt,
                            operation = "capability_request_http_error",
                            attempt_duration_ms = attempt_duration.as_millis(),
                            http_status = status_code,
                            error = %error_msg,
                            "HTTP error response"
                        );
                        
                        self.connection_pool
                            .record_request(&service.endpoint, response_time_ms, false)
                            .await;
                    }
                }
                Ok(Err(network_error)) => {
                    let error_msg = format!("Network error: {}", network_error);
                    last_error = Some(error_msg.clone());
                    
                    tracing::warn!(
                        correlation_id = %correlation_id,
                        service_id = %service.service_id,
                        attempt = attempt,
                        operation = "capability_request_network_error",
                        attempt_duration_ms = attempt_duration.as_millis(),
                        error = %error_msg,
                        "Network error"
                    );
                    
                    self.connection_pool
                        .record_request(&service.endpoint, response_time_ms, false)
                        .await;
                }
                Err(timeout_error) => {
                    let error_msg = format!("Request timed out: {}", timeout_error);
                    last_error = Some(error_msg.clone());
                    
                    tracing::warn!(
                        correlation_id = %correlation_id,
                        service_id = %service.service_id,
                        attempt = attempt,
                        operation = "capability_request_timeout",
                        attempt_duration_ms = attempt_duration.as_millis(),
                        timeout_ms = per_attempt_timeout.as_millis(),
                        error = %error_msg,
                        "Request timed out"
                    );
                    
                    self.connection_pool
                        .record_request(&service.endpoint, response_time_ms, false)
                        .await;
                }
            }
            
            // Exponential backoff between retries (except on last attempt)
            if attempt < max_retries {
                let delay = base_delay * (2_u32.pow(attempt - 1));
                tracing::debug!(
                    correlation_id = %correlation_id,
                    service_id = %service.service_id,
                    attempt = attempt,
                    delay_ms = delay.as_millis(),
                    operation = "capability_request_retry_delay",
                    "Waiting before retry"
                );
                tokio::time::sleep(delay).await;
            }
        }
        
        let total_duration = operation_start.elapsed();
        let final_error = last_error.unwrap_or_else(|| "All capability request attempts failed".to_string());
        
        tracing::error!(
            correlation_id = %correlation_id,
            service_id = %service.service_id,
            request_id = %request.request_id,
            operation = "capability_request_failure",
            total_duration_ms = total_duration.as_millis(),
            attempts = max_retries,
            final_error = %final_error,
            "Capability request failed after all retry attempts"
        );
            
        Err(PrimalError::NetworkError(format!(
            "Capability request to service {} failed: {}",
            service.service_id, final_error
        )))
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
    async fn discover_via_well_known_endpoints(&mut self) -> UniversalResult<()> {
        // Try common service discovery patterns
        let well_known_ports = vec![8080, 8443, 8444, 8445, 8500];
        let localhost_base = "http://localhost";

        for port in well_known_ports {
            let endpoint = format!("{}:{}", localhost_base, port);

            if let Ok(capabilities) = self.query_service_capabilities(&endpoint).await {
                let service = DiscoveredService {
                    service_id: format!("service-{}", port),
                    instance_id: format!("instance-{}", port),
                    endpoint,
                    capabilities: capabilities.clone(),
                    health: ServiceHealth::Healthy,
                    discovered_at: Utc::now(),
                    last_health_check: Some(Utc::now()),
                };

                // Index by each capability, not by primal type
                let mut services = self.discovered_services.write().await;
                for capability in capabilities {
                    services
                        .entry(capability.clone())
                        .or_insert_with(Vec::new)
                        .push(service.clone());
                }
            }
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
    async fn query_service_capabilities(&self, endpoint: &str) -> UniversalResult<Vec<String>> {
        let capabilities_url = format!("{}/api/v1/capabilities", endpoint);

        match self
            .connection_pool
            .get_client(endpoint)
            .await
            .get(&capabilities_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => {
                let capabilities: Vec<String> = response.json().await.unwrap_or_default();
                Ok(capabilities)
            }
            Err(_) => Ok(Vec::new()), // Service not available or doesn't support capability discovery
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
        let connection_stats = self.connection_pool.get_stats().await;

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
        let primal_type = "squirrel"; // Placeholder, actual type would be determined by request

        Ok(PrimalResponse {
            response_id: uuid::Uuid::new_v4(),
            request_id: uuid::Uuid::parse_str(&request.request_id.to_string())
                .unwrap_or_else(|e| {
                    warn!("Failed to parse request UUID '{}': {}, generating new UUID", 
                         request.request_id, e);
                    uuid::Uuid::new_v4()
                }),
            payload: response_data.clone(),
            metadata: std::collections::HashMap::new(),
            data: response_data,
            success: true,
            error_message: None,
            error: None,
            timestamp: chrono::Utc::now(),
            duration: chrono::Duration::milliseconds(100),
            status: crate::universal::ResponseStatus::Success,
        })
    }
}
