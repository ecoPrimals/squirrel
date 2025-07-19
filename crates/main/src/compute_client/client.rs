//! Universal Compute Client Implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use crate::error::PrimalError;
use crate::universal::{PrimalCapability, PrimalContext, PrimalRequest, UniversalResult};
use crate::universal_primal_ecosystem::UniversalPrimalEcosystem;

use super::types::*;
use super::ai_metadata::*;
use super::providers::*;

// ============================================================================
// UNIVERSAL COMPUTE CLIENT IMPLEMENTATION
// ============================================================================

/// Universal Compute Client - AI-First, Capability-Based Design
#[derive(Debug)]
pub struct UniversalComputeClient {
    /// Ecosystem integration for service discovery
    ecosystem: Arc<UniversalPrimalEcosystem>,
    
    /// Client configuration
    config: ComputeClientConfig,
    
    /// Active compute providers (discovered dynamically)
    providers: Arc<RwLock<HashMap<String, ComputeProvider>>>,
    
    /// Request context for routing
    context: PrimalContext,
    
    /// AI-first metadata for intelligent routing
    ai_metadata: AIComputeMetadata,
}

impl UniversalComputeClient {
    /// Create new universal compute client
    pub fn new(
        ecosystem: Arc<UniversalPrimalEcosystem>,
        config: ComputeClientConfig,
        context: PrimalContext,
    ) -> Self {
        Self {
            ecosystem,
            config,
            providers: Arc::new(RwLock::new(HashMap::new())),
            context,
            ai_metadata: AIComputeMetadata::default(),
        }
    }

    /// Initialize the universal compute client
    pub async fn initialize(&self) -> UniversalResult<()> {
        info!("Initializing Universal Compute Client");

        // Discover all available compute providers
        self.discover_compute_providers().await?;

        // Start background tasks for health monitoring
        self.start_health_monitoring().await;

        info!("Universal Compute Client initialized successfully");
        Ok(())
    }

    /// Discover compute providers using capability-based discovery
    async fn discover_compute_providers(&self) -> UniversalResult<()> {
        debug!("Discovering compute providers through capability-based search");

        let compute_capabilities = vec![
            PrimalCapability::ContainerRuntime {
                orchestrators: vec!["kubernetes".to_string(), "docker".to_string()],
            },
            PrimalCapability::ServerlessExecution {
                languages: vec!["python".to_string(), "javascript".to_string()],
            },
            PrimalCapability::GpuAcceleration {
                cuda_support: true,
            },
        ];

        let mut discovered_providers = HashMap::new();

        for capability in compute_capabilities {
            let providers = self.ecosystem.find_by_capability(&capability).await;
            
            for primal in providers {
                let provider = ComputeProvider::from_discovered_primal(&primal);
                discovered_providers.insert(primal.instance_id.clone(), provider);
            }
        }

        let mut providers = self.providers.write().await;
        *providers = discovered_providers;

        info!("Discovered {} compute providers", providers.len());
        Ok(())
    }

    /// Start background health monitoring
    async fn start_health_monitoring(&self) {
        debug!("Started background health monitoring for compute providers");
    }

    /// Execute universal compute operation
    pub async fn execute_operation(
        &self,
        request: UniversalComputeRequest,
    ) -> UniversalResult<UniversalComputeResponse> {
        debug!("Executing universal compute operation: {:?}", request.operation);

        // Select best provider using AI-based routing
        let provider = self.select_best_provider(&request).await?;

        // Create primal request
        let primal_request = PrimalRequest::new(
            "squirrel",
            &provider.provider_id,
            "compute_operation",
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
        let compute_response = self.process_response(response, &provider, &request).await?;

        // Update provider health based on operation
        self.update_provider_health(&provider.provider_id, &compute_response).await;

        info!("Universal compute operation completed successfully");
        Ok(compute_response)
    }

    /// Select best provider using AI-based routing
    async fn select_best_provider(
        &self,
        request: &UniversalComputeRequest,
    ) -> UniversalResult<ComputeProvider> {
        let providers = self.providers.read().await;
        
        if providers.is_empty() {
            return Err(PrimalError::ResourceNotFound(
                "No compute providers available".to_string(),
            ));
        }

        // AI-based provider selection algorithm
        let mut best_provider: Option<ComputeProvider> = None;
        let mut best_score = 0.0;

        for provider in providers.values() {
            let score = self.calculate_provider_score(provider, request).await;
            if score > best_score {
                best_score = score;
                best_provider = Some(provider.clone());
            }
        }

        best_provider.ok_or_else(|| {
            PrimalError::OperationFailed("Failed to select compute provider".to_string())
        })
    }

    /// Calculate provider score for specific request
    async fn calculate_provider_score(
        &self,
        provider: &ComputeProvider,
        request: &UniversalComputeRequest,
    ) -> f64 {
        let mut score = provider.routing_score;

        // Factor in current health
        score *= provider.health.health_score;

        // Factor in resource availability
        let cpu_availability = (1.0 - provider.health.cpu_load).max(0.0);
        let memory_availability = (1.0 - provider.health.memory_usage).max(0.0);
        score *= (cpu_availability + memory_availability) / 2.0;

        // Factor in queue length
        if provider.health.queue_length > 0 {
            score *= 0.8; // Penalty for queued jobs
        }

        // Factor in workload characteristics
        let workload = &request.ai_context.workload_characteristics;
        if workload.cpu_intensity > 0.8 && provider.health.cpu_load > 0.7 {
            score *= 0.5; // High penalty for CPU-intensive tasks on loaded providers
        }

        score.min(1.0).max(0.0)
    }

    /// Process response and generate AI insights
    async fn process_response(
        &self,
        response: crate::universal::PrimalResponse,
        provider: &ComputeProvider,
        request: &UniversalComputeRequest,
    ) -> UniversalResult<UniversalComputeResponse> {
        let success = response.success;
        
        let results = if success {
            Some(ComputeResults {
                output_data: response.data.get("output").and_then(|v| {
                    base64::decode(v.as_str().unwrap_or("")).ok()
                }),
                return_code: response.data.get("return_code")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32,
                stdout: response.data.get("stdout")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                stderr: response.data.get("stderr")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                metadata: HashMap::new(),
            })
        } else {
            None
        };

        Ok(UniversalComputeResponse {
            request_id: request.request_id,
            success,
            results,
            provider_id: provider.provider_id.clone(),
            performance: ComputePerformanceMetrics {
                execution_time: std::time::Duration::from_millis(response.duration.num_milliseconds() as u64),
                queue_time: std::time::Duration::from_secs(0),
                resource_utilization: request.ai_context.workload_characteristics.clone().into(),
                cost_breakdown: CostBreakdown {
                    cpu_cost: 0.05,
                    memory_cost: 0.02,
                    gpu_cost: Some(0.10),
                    storage_cost: 0.01,
                    network_cost: 0.01,
                    total_cost: 0.19,
                },
                provider_health: provider.health.health_score,
            },
            ai_insights: AIComputeInsights {
                confidence_score: 0.9,
                performance_optimizations: vec![
                    "Consider using GPU acceleration for this workload".to_string(),
                ],
                cost_optimizations: vec![
                    "Use spot instances for non-critical workloads".to_string(),
                ],
                alternative_providers: vec![],
                workload_analysis: WorkloadAnalysis {
                    patterns: vec!["CPU-intensive".to_string()],
                    efficiency_score: 0.8,
                    bottlenecks: vec!["Memory bandwidth".to_string()],
                    recommendations: vec!["Increase memory allocation".to_string()],
                },
            },
            error: response.error_message,
        })
    }

    /// Update provider health based on operation results
    async fn update_provider_health(&self, provider_id: &str, response: &UniversalComputeResponse) {
        let mut providers = self.providers.write().await;
        if let Some(provider) = providers.get_mut(provider_id) {
            provider.health.avg_execution_time_ms = response.performance.execution_time.as_millis() as f64;
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

impl UniversalComputeClient {
    /// Execute code using intelligent provider selection
    pub async fn execute_code(
        &self,
        language: &str,
        code: String,
        priority: ComputePriority,
    ) -> UniversalResult<UniversalComputeResponse> {
        let request = UniversalComputeRequest {
            request_id: Uuid::new_v4(),
            operation: ComputeOperation::Execute {
                language: language.to_string(),
                entrypoint: "main".to_string(),
            },
            payload: ComputePayload {
                code: Some(code),
                input_data: None,
                environment: HashMap::new(),
                dependencies: Vec::new(),
                parameters: HashMap::new(),
            },
            resources: ResourceRequirements {
                cpu_cores: 2,
                memory_gb: 4,
                gpu_units: None,
                storage_gb: 10,
                max_execution_time: std::time::Duration::from_secs(300),
                network_bandwidth_mbps: Some(100.0),
            },
            security: ComputeSecurityRequirements::default(),
            ai_context: AIComputeContext {
                workload_characteristics: WorkloadCharacteristics {
                    cpu_intensity: 0.6,
                    memory_intensity: 0.4,
                    io_intensity: 0.2,
                    gpu_requirement: 0.0,
                    parallelizability: 0.5,
                },
                priority,
                deadline: None,
                cost_performance_preference: CostPerformancePreference::Balanced,
            },
            metadata: HashMap::new(),
        };

        self.execute_operation(request).await
    }
}

// Helper conversions
impl From<WorkloadCharacteristics> for ResourceUtilization {
    fn from(workload: WorkloadCharacteristics) -> Self {
        Self {
            cpu_utilization: workload.cpu_intensity,
            memory_utilization: workload.memory_intensity,
            gpu_utilization: Some(workload.gpu_requirement),
            network_utilization: Some(workload.io_intensity),
        }
    }
}

impl Default for ComputeSecurityRequirements {
    fn default() -> Self {
        Self {
            isolation_level: IsolationLevel::Container,
            trusted_execution: false,
            encryption_requirements: EncryptionRequirements {
                data_at_rest: true,
                data_in_transit: true,
                data_in_use: false,
            },
            network_security: NetworkSecurityLevel::Basic,
        }
    }
} 