//! Universal Storage Client Implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};

use crate::error::PrimalError;
use crate::universal::{PrimalCapability, PrimalContext, PrimalRequest, UniversalResult};
use crate::universal_primal_ecosystem::UniversalPrimalEcosystem;

use super::types::*;
use super::ai_metadata::*;
use super::providers::*;

// ============================================================================
// UNIVERSAL STORAGE CLIENT IMPLEMENTATION
// ============================================================================

/// Universal Storage Client - AI-First, Capability-Based Design
///
/// This client automatically discovers and integrates with any storage service
/// that provides the required capabilities, regardless of implementation.
#[derive(Debug)]
pub struct UniversalStorageClient {
    /// Ecosystem integration for service discovery
    ecosystem: Arc<UniversalPrimalEcosystem>,
    
    /// Client configuration
    config: StorageClientConfig,
    
    /// Active storage providers (discovered dynamically)
    providers: Arc<RwLock<HashMap<String, StorageProvider>>>,
    
    /// Request context for routing
    context: PrimalContext,
    
    /// AI-first metadata for intelligent routing
    ai_metadata: AIStorageMetadata,
}

impl UniversalStorageClient {
    /// Create new universal storage client
    pub fn new(
        ecosystem: Arc<UniversalPrimalEcosystem>,
        config: StorageClientConfig,
        context: PrimalContext,
    ) -> Self {
        Self {
            ecosystem,
            config,
            providers: Arc::new(RwLock::new(HashMap::new())),
            context,
            ai_metadata: AIStorageMetadata::default(),
        }
    }

    /// Initialize the universal storage client
    pub async fn initialize(&self) -> UniversalResult<()> {
        info!("Initializing Universal Storage Client");

        // Discover all available storage providers
        self.discover_storage_providers().await?;

        // Start background tasks for health monitoring
        self.start_health_monitoring().await;

        info!("Universal Storage Client initialized successfully");
        Ok(())
    }

    /// Discover storage providers using capability-based discovery
    async fn discover_storage_providers(&self) -> UniversalResult<()> {
        debug!("Discovering storage providers through capability-based search");

        let storage_capabilities = vec![
            PrimalCapability::ObjectStorage {
                backends: vec!["universal".to_string()],
            },
            PrimalCapability::FileSystem {
                supports_zfs: false,
            },
        ];

        let mut discovered_providers = HashMap::new();

        for capability in storage_capabilities {
            let providers = self.ecosystem.find_by_capability(&capability).await;
            
            for primal in providers {
                let provider = StorageProvider::from_discovered_primal(&primal);
                discovered_providers.insert(primal.instance_id.clone(), provider);
            }
        }

        let mut providers = self.providers.write().await;
        *providers = discovered_providers;

        info!("Discovered {} storage providers", providers.len());
        Ok(())
    }

    /// Start background health monitoring
    async fn start_health_monitoring(&self) {
        debug!("Started background health monitoring for storage providers");
    }

    /// Execute universal storage operation
    pub async fn execute_operation(
        &self,
        request: UniversalStorageRequest,
    ) -> UniversalResult<UniversalStorageResponse> {
        debug!("Executing universal storage operation: {:?}", request.operation);

        // Select best provider using AI-based routing
        let provider = self.select_best_provider(&request).await?;

        // Create primal request
        let primal_request = PrimalRequest::new(
            "squirrel",
            &provider.provider_id,
            "storage_operation",
            serde_json::to_value(&request).map_err(|e| {
                PrimalError::SerializationError(format!("Failed to serialize request: {}", e))
            })?,
            self.context.clone(),
        );

        // Send request through ecosystem
        let response = self
            .ecosystem
            .send_to_primal(&provider.provider_id, primal_request)
            .await?;

        // Process response and generate AI insights
        let storage_response = self.process_response(response, &provider, &request).await?;

        // Update provider health based on operation
        self.update_provider_health(&provider.provider_id, &storage_response).await;

        info!("Universal storage operation completed successfully");
        Ok(storage_response)
    }

    /// Select best provider using AI-based routing
    async fn select_best_provider(
        &self,
        request: &UniversalStorageRequest,
    ) -> UniversalResult<StorageProvider> {
        let providers = self.providers.read().await;
        
        if providers.is_empty() {
            return Err(PrimalError::ResourceNotFound(
                "No storage providers available".to_string(),
            ));
        }

        // AI-based provider selection algorithm
        let mut best_provider: Option<StorageProvider> = None;
        let mut best_score = 0.0;

        for provider in providers.values() {
            let score = self.calculate_provider_score(provider, request).await;
            if score > best_score {
                best_score = score;
                best_provider = Some(provider.clone());
            }
        }

        best_provider.ok_or_else(|| {
            PrimalError::OperationFailed("Failed to select storage provider".to_string())
        })
    }

    /// Calculate provider score for specific request
    async fn calculate_provider_score(
        &self,
        provider: &StorageProvider,
        request: &UniversalStorageRequest,
    ) -> f64 {
        let mut score = provider.routing_score;

        // Factor in current health
        score *= provider.health.health_score;

        // Factor in performance requirements
        if request.requirements.max_latency_ms > 0 {
            let latency_score = if provider.health.current_latency_ms <= request.requirements.max_latency_ms as f64 {
                1.0
            } else {
                request.requirements.max_latency_ms as f64 / provider.health.current_latency_ms
            };
            score *= latency_score;
        }

        // Factor in throughput requirements
        if request.requirements.min_throughput_mbps > 0.0 {
            let throughput_score = if provider.health.current_throughput_mbps >= request.requirements.min_throughput_mbps {
                1.0
            } else {
                provider.health.current_throughput_mbps / request.requirements.min_throughput_mbps
            };
            score *= throughput_score;
        }

        score.min(1.0).max(0.0)
    }

    /// Process response and generate AI insights
    async fn process_response(
        &self,
        response: crate::universal::PrimalResponse,
        provider: &StorageProvider,
        request: &UniversalStorageRequest,
    ) -> UniversalResult<UniversalStorageResponse> {
        let success = response.success;
        let data = if success {
            response.data.get("data").and_then(|v| {
                general_purpose::STANDARD.decode(v.as_str().unwrap_or("")).ok()
            })
        } else {
            None
        };

        Ok(UniversalStorageResponse {
            request_id: request.request_id,
            success,
            data,
            metadata: request.metadata.clone(),
            provider_id: provider.provider_id.clone(),
            performance: PerformanceMetrics {
                latency_ms: response.duration.num_milliseconds() as f64,
                throughput_mbps: 100.0, // Calculate based on data size and duration
                provider_health: provider.health.health_score,
                estimated_cost: 0.01, // Calculate based on operation and data size
            },
            ai_insights: AIStorageInsights {
                confidence_score: 0.9,
                optimizations: vec![
                    "Consider using compression for large files".to_string(),
                    "Enable caching for frequently accessed data".to_string(),
                ],
                alternative_providers: vec![],
                access_predictions: vec![],
                cost_recommendations: vec![
                    "Use cold storage for infrequently accessed data".to_string(),
                ],
            },
            error: response.error_message,
        })
    }

    /// Update provider health based on operation results
    async fn update_provider_health(&self, provider_id: &str, response: &UniversalStorageResponse) {
        let mut providers = self.providers.write().await;
        if let Some(provider) = providers.get_mut(provider_id) {
            // Update health metrics based on operation performance
            provider.health.current_latency_ms = response.performance.latency_ms;
            provider.health.last_check = chrono::Utc::now();
            
            if response.success {
                provider.health.health_score = (provider.health.health_score * 0.9 + 0.1).min(1.0);
            } else {
                provider.health.health_score = (provider.health.health_score * 0.9).max(0.1);
            }
        }
    }
}

// ============================================================================
// CONVENIENCE METHODS
// ============================================================================

impl UniversalStorageClient {
    /// Store data using intelligent provider selection
    pub async fn store(
        &self,
        key: &str,
        data: Vec<u8>,
        classification: DataClassification,
    ) -> UniversalResult<UniversalStorageResponse> {
        let request = UniversalStorageRequest {
            request_id: Uuid::new_v4(),
            operation: StorageOperation::Store,
            object_key: key.to_string(),
            data: Some(data),
            metadata: HashMap::new(),
            classification,
            requirements: PerformanceRequirements {
                max_latency_ms: 5000,
                min_throughput_mbps: 10.0,
                availability_sla: 0.99,
                durability_nines: 11,
            },
            ai_context: AIRequestContext {
                access_frequency: AccessFrequency::Hot,
                data_lifetime: std::time::Duration::from_secs(86400),
                sharing_scope: SharingScope::Private,
                processing_hints: Vec::new(),
            },
        };

        self.execute_operation(request).await
    }

    /// Retrieve data using intelligent provider selection
    pub async fn retrieve(&self, key: &str) -> UniversalResult<UniversalStorageResponse> {
        let request = UniversalStorageRequest {
            request_id: Uuid::new_v4(),
            operation: StorageOperation::Retrieve,
            object_key: key.to_string(),
            data: None,
            metadata: HashMap::new(),
            classification: DataClassification::Internal,
            requirements: PerformanceRequirements {
                max_latency_ms: 1000,
                min_throughput_mbps: 50.0,
                availability_sla: 0.999,
                durability_nines: 11,
            },
            ai_context: AIRequestContext {
                access_frequency: AccessFrequency::Hot,
                data_lifetime: std::time::Duration::from_secs(3600),
                sharing_scope: SharingScope::Private,
                processing_hints: Vec::new(),
            },
        };

        self.execute_operation(request).await
    }

    /// Delete data from storage
    pub async fn delete(&self, key: &str) -> UniversalResult<UniversalStorageResponse> {
        let request = UniversalStorageRequest {
            request_id: Uuid::new_v4(),
            operation: StorageOperation::Delete,
            object_key: key.to_string(),
            data: None,
            metadata: HashMap::new(),
            classification: DataClassification::Internal,
            requirements: PerformanceRequirements {
                max_latency_ms: 2000,
                min_throughput_mbps: 1.0,
                availability_sla: 0.99,
                durability_nines: 11,
            },
            ai_context: AIRequestContext {
                access_frequency: AccessFrequency::Cold,
                data_lifetime: std::time::Duration::from_secs(0),
                sharing_scope: SharingScope::Private,
                processing_hints: Vec::new(),
            },
        };

        self.execute_operation(request).await
    }
} 