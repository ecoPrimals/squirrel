// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors
#![expect(deprecated, reason = "Backward compatibility during migration")]

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

mod cache;
mod discovery;
mod ipc;

// Public exports
pub use types::*;

// Re-export DiscoveredPrimal for backward compatibility
pub use universal_patterns::registry::DiscoveredPrimal;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(test)]
use tokio::sync::Mutex;
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
    /// When set, the next `send_to_primal` returns this response (unit tests only).
    #[cfg(test)]
    test_send_to_primal_response: Arc<Mutex<Option<PrimalResponse>>>,
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
            #[cfg(test)]
            test_send_to_primal_response: Arc::new(Mutex::new(None)),
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

        if let Ok(matches) = self.find_services_by_capability(&capability_request).await
            && let Some(best_match) = matches.first()
        {
            self.service_mesh_endpoint = Some(best_match.service.endpoint.clone());
            info!(
                "Discovered service mesh at: {}",
                best_match.service.endpoint
            );
        }

        Ok(())
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
        #[cfg(test)]
        {
            let mut guard = self.test_send_to_primal_response.lock().await;
            if let Some(resp) = guard.take() {
                return Ok(resp);
            }
        }
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

#[cfg(test)]
impl UniversalPrimalEcosystem {
    /// Register a discovered service under each of its advertised capability keys (unit tests only).
    pub(crate) async fn test_only_register_service(&self, service: DiscoveredService) {
        let mut map = self.discovered_services.write().await;
        for cap in &service.capabilities {
            map.entry(cap.clone()).or_default().push(service.clone());
        }
    }

    /// Override the next `send_to_primal` result (unit tests only).
    pub async fn test_only_set_next_primal_response(&self, response: PrimalResponse) {
        *self.test_send_to_primal_response.lock().await = Some(response);
    }
}

#[cfg(test)]
#[path = "ecosystem_tests.rs"]
mod ecosystem_integration_tests;
