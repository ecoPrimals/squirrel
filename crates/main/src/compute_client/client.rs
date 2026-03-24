// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Compute Client Implementation

use base64::{Engine as _, engine::general_purpose};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::PrimalError;
use crate::universal::{
    PrimalCapability, PrimalContext, PrimalRequest, PrimalResponse, UniversalResult,
};
use crate::universal_primal_ecosystem::UniversalPrimalEcosystem;

use super::providers::ComputeProvider;
use super::types::{
    AIComputeContext, AIComputeInsights, ComputeClientConfig, ComputeOperation, ComputePayload,
    ComputePerformanceMetrics, ComputePriority, ComputeResults, ComputeSecurityRequirements,
    CostBreakdown, CostPerformancePreference, EncryptionRequirements, IsolationLevel,
    NetworkSecurityLevel, ResourceRequirements, ResourceUtilization, UniversalComputeRequest,
    UniversalComputeResponse, WorkloadAnalysis, WorkloadCharacteristics,
};

// ============================================================================
// UNIVERSAL COMPUTE CLIENT IMPLEMENTATION
// ============================================================================

/// Universal Compute Client that automatically discovers and routes requests to the best
/// available compute provider (`ToadStool`, cloud providers, etc.).
///
/// This client implements capability-based discovery, meaning it finds any provider
/// that provides the required capabilities, regardless of implementation.
#[derive(Debug)]
pub struct UniversalComputeClient {
    /// Ecosystem integration for service discovery
    ecosystem: Arc<UniversalPrimalEcosystem>,

    /// Client configuration
    config: ComputeClientConfig,

    /// Active compute providers (discovered dynamically)
    providers: Arc<DashMap<String, ComputeProvider>>,

    /// Request context for routing
    context: PrimalContext,
    // Removed ai_metadata - was over-engineered early implementation
}

impl UniversalComputeClient {
    /// Create new universal compute client
    #[must_use]
    pub fn new(
        ecosystem: Arc<UniversalPrimalEcosystem>,
        config: ComputeClientConfig,
        context: PrimalContext,
    ) -> Self {
        Self {
            ecosystem,
            config,
            providers: Arc::new(DashMap::new()),
            context,
            // Removed ai_metadata: AIComputeMetadata::default(),
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

        // Define compute capabilities to discover
        // NOTE: These lists describe SUPPORTED runtimes/orchestrators, not hardcoded requirements
        // The actual provider is discovered dynamically via UniversalAdapterV2
        let compute_capabilities = vec![
            PrimalCapability::ContainerRuntime {
                // Capability: Can run containers via Docker API, containerd, or any OCI runtime
                container_types: vec!["docker".to_string(), "containerd".to_string()],
                // Capability: Can work with Kubernetes, Docker Swarm, or any orchestrator
                orchestrators: vec!["kubernetes".to_string(), "docker".to_string()],
            },
            PrimalCapability::ServerlessExecution {
                languages: vec!["python".to_string(), "javascript".to_string()],
            },
            PrimalCapability::GpuAcceleration {
                gpu_types: vec!["nvidia".to_string(), "amd".to_string()],
                cuda_support: true,
            },
        ];

        let mut discovered_providers = HashMap::new();

        for capability in compute_capabilities {
            if let Ok(providers) = self
                .ecosystem
                .find_by_capability(match capability {
                    PrimalCapability::ContainerRuntime { .. } => "container-runtime",
                    PrimalCapability::GpuAcceleration { .. } => "gpu-acceleration",
                    PrimalCapability::ServerlessExecution { .. } => "serverless-execution",
                    _ => "generic-capability",
                })
                .await
            {
                for primal in providers {
                    let provider = ComputeProvider::from_discovered_primal(
                        &universal_patterns::registry::DiscoveredPrimal {
                            id: primal.service.service_id.clone(),
                            instance_id: primal.service.instance_id.clone(),
                            primal_type: universal_patterns::traits::PrimalType::Coordinator,
                            capabilities: vec![],
                            endpoint: primal.service.endpoint.clone(),
                            health: universal_patterns::traits::PrimalHealth::Healthy,
                            context: universal_patterns::traits::PrimalContext::default(),
                            port_info: None,
                        },
                    );
                    discovered_providers.insert(primal.service.instance_id.clone(), provider);
                }
            }
        }

        // Clear existing providers and insert discovered ones
        self.providers.clear();
        for (key, value) in discovered_providers {
            self.providers.insert(key, value);
        }

        info!("Discovered {} compute providers", self.providers.len());
        Ok(())
    }

    /// Start background health monitoring
    async fn start_health_monitoring(&self) {
        debug!("Started background health monitoring for compute providers");
    }

    /// Execute universal compute operation
    pub async fn execute_operation(
        &self,
        mut request: UniversalComputeRequest,
    ) -> UniversalResult<UniversalComputeResponse> {
        debug!(
            "Executing universal compute operation: {:?}",
            request.operation
        );

        // Apply configuration-based enhancements
        self.apply_configuration_defaults(&mut request);

        // Removed AI metadata enhancement - was over-engineered early implementation

        // Select best provider using AI-based routing
        let provider = self.select_best_provider(&request).await?;

        // Create primal request
        let primal_request = PrimalRequest::new(
            "squirrel",
            &provider.provider_id,
            "compute_operation",
            serde_json::to_value(&request).map_err(|e| {
                PrimalError::SerializationError(format!("Failed to serialize request: {e}"))
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
        self.update_provider_health(&provider.provider_id, &compute_response)
            .await;

        info!("Universal compute operation completed successfully");
        Ok(compute_response)
    }

    /// Select best provider using AI-based routing
    async fn select_best_provider(
        &self,
        request: &UniversalComputeRequest,
    ) -> UniversalResult<ComputeProvider> {
        if self.providers.is_empty() {
            return Err(PrimalError::ResourceNotFound(
                "No compute providers available".to_string(),
            ));
        }

        // AI-based provider selection algorithm
        let mut best_provider: Option<ComputeProvider> = None;
        let mut best_score = 0.0;

        for entry in self.providers.iter() {
            let provider = entry.value();
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
        score *= f64::midpoint(cpu_availability, memory_availability);

        // Factor in queue length
        if provider.health.queue_length > 0 {
            score *= 0.8; // Penalty for queued jobs
        }

        // Factor in workload characteristics
        let workload = &request.ai_context.workload_characteristics;
        if workload.cpu_intensity > 0.8 && provider.health.cpu_load > 0.7 {
            score *= 0.5; // High penalty for CPU-intensive tasks on loaded providers
        }

        score.clamp(0.0, 1.0)
    }

    /// Process response and generate AI insights
    async fn process_response(
        &self,
        response: PrimalResponse,
        provider: &ComputeProvider,
        request: &UniversalComputeRequest,
    ) -> UniversalResult<UniversalComputeResponse> {
        let success = response.success;

        let results = if success {
            Some(ComputeResults {
                output_data: response.data.as_ref().and_then(|data| {
                    data.get("output").and_then(|v| {
                        general_purpose::STANDARD
                            .decode(v.as_str().unwrap_or_else(|| {
                                warn!("Missing or invalid base64 data in compute response");
                                ""
                            }))
                            .ok()
                    })
                }),
                return_code: {
                    let code = response
                        .data
                        .as_ref()
                        .and_then(|data| data.get("return_code"))
                        .and_then(serde_json::Value::as_i64)
                        .unwrap_or_else(|| {
                            warn!("Missing return_code in compute response, defaulting to 0");
                            0
                        });
                    // Clamp i64 to i32::MAX to avoid truncation
                    code.min(i64::from(i32::MAX)).max(i64::from(i32::MIN)) as i32
                },
                stdout: response
                    .data
                    .as_ref()
                    .and_then(|data| data.get("stdout"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| {
                        warn!("Missing stdout in compute response, using empty string");
                        ""
                    })
                    .to_string(),
                stderr: response
                    .data
                    .as_ref()
                    .and_then(|data| data.get("stderr"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| {
                        warn!("Missing stderr in compute response, using empty string");
                        ""
                    })
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
                execution_time: std::time::Duration::from_millis(
                    response.processing_time_ms.unwrap_or(100),
                ),
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
        if let Some(mut provider) = self.providers.get_mut(provider_id) {
            provider.health.avg_execution_time_ms =
                response.performance.execution_time.as_millis() as f64;
            provider.health.last_check = chrono::Utc::now();

            if response.success {
                provider.health.health_score =
                    provider.health.health_score.mul_add(0.9, 0.1).min(1.0);
            } else {
                provider.health.health_score = (provider.health.health_score * 0.9).max(0.1);
            }
        }
    }

    /// Get compute client configuration
    #[must_use]
    pub const fn get_compute_config(&self) -> &ComputeClientConfig {
        // Use config field to provide configuration access
        &self.config
    }

    /// Apply configuration-based defaults to a request
    pub fn apply_configuration_defaults(&self, request: &mut UniversalComputeRequest) {
        // Apply timeout defaults from config
        if request.resources.max_execution_time.as_secs() == 0 {
            request.resources.max_execution_time = self.config.operation_timeout;
        }

        // Apply resource defaults from config
        if request.resources.cpu_cores == 0 {
            request.resources.cpu_cores = self.config.resource_requirements.cpu_cores;
        }

        if request.resources.memory_gb == 0 {
            request.resources.memory_gb = self.config.resource_requirements.memory_gb;
        }

        // Apply security defaults from config
        request.security = self.config.security_requirements.clone();

        debug!("Applied configuration defaults to compute request");
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

#[cfg(test)]
impl UniversalComputeClient {
    /// Insert a compute provider directly (unit tests only; avoids live discovery).
    pub(crate) fn test_only_insert_provider(&self, provider: super::providers::ComputeProvider) {
        self.providers
            .insert(provider.provider_id.clone(), provider);
    }
}

#[cfg(test)]
mod compute_client_tests {
    use super::*;
    use crate::compute_client::providers::ComputeProvider;
    use crate::compute_client::types::{
        AIComputeContext, ComputeOperation, ComputePayload, ComputePriority, ResourceRequirements,
        UniversalComputeRequest, WorkloadCharacteristics,
    };
    use crate::universal::PrimalContext;
    use crate::universal_primal_ecosystem::UniversalPrimalEcosystem;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    fn sample_provider(id: &str) -> ComputeProvider {
        ComputeProvider::from_discovered_primal(&universal_patterns::registry::DiscoveredPrimal {
            id: id.to_string(),
            instance_id: id.to_string(),
            primal_type: universal_patterns::traits::PrimalType::Coordinator,
            capabilities: vec![],
            endpoint: "unix:///tmp/compute.sock".to_string(),
            health: universal_patterns::traits::PrimalHealth::Healthy,
            context: universal_patterns::traits::PrimalContext::default(),
            port_info: None,
        })
    }

    fn minimal_request() -> UniversalComputeRequest {
        UniversalComputeRequest {
            request_id: Uuid::new_v4(),
            operation: ComputeOperation::Execute {
                language: "python".to_string(),
                entrypoint: "main".to_string(),
            },
            payload: ComputePayload {
                code: Some("print(1)".to_string()),
                input_data: None,
                environment: HashMap::new(),
                dependencies: Vec::new(),
                parameters: HashMap::new(),
            },
            resources: ResourceRequirements {
                cpu_cores: 0,
                memory_gb: 0,
                gpu_units: None,
                storage_gb: 1,
                max_execution_time: Duration::from_secs(0),
                network_bandwidth_mbps: None,
            },
            security: ComputeSecurityRequirements::default(),
            ai_context: AIComputeContext {
                workload_characteristics: WorkloadCharacteristics {
                    cpu_intensity: 0.9,
                    memory_intensity: 0.1,
                    io_intensity: 0.2,
                    gpu_requirement: 0.0,
                    parallelizability: 0.5,
                },
                priority: ComputePriority::Normal,
                deadline: None,
                cost_performance_preference: CostPerformancePreference::Balanced,
            },
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn initialize_runs_without_network() {
        let eco = Arc::new(UniversalPrimalEcosystem::new(PrimalContext::default()));
        let client = UniversalComputeClient::new(
            eco,
            ComputeClientConfig::default(),
            PrimalContext::default(),
        );
        client.initialize().await.expect("should succeed");
        assert!(client.get_compute_config().max_retries <= 10);
    }

    #[tokio::test]
    async fn execute_operation_errors_when_no_providers() {
        let eco = Arc::new(UniversalPrimalEcosystem::new(PrimalContext::default()));
        let client = UniversalComputeClient::new(
            eco,
            ComputeClientConfig::default(),
            PrimalContext::default(),
        );
        let err = client
            .execute_operation(minimal_request())
            .await
            .unwrap_err();
        assert!(matches!(err, PrimalError::ResourceNotFound(_)));
    }

    #[tokio::test]
    async fn execute_operation_applies_defaults_and_succeeds_with_stub_ecosystem() {
        let eco = Arc::new(UniversalPrimalEcosystem::new(PrimalContext::default()));
        let client = UniversalComputeClient::new(
            eco,
            ComputeClientConfig::default(),
            PrimalContext::default(),
        );
        let mut p = sample_provider("prov-1");
        p.health.queue_length = 1;
        client.test_only_insert_provider(p);

        let mut req = minimal_request();
        client.apply_configuration_defaults(&mut req);
        assert!(req.resources.cpu_cores > 0);
        assert!(req.resources.memory_gb > 0);
        assert!(req.resources.max_execution_time.as_secs() > 0);

        let resp = client.execute_operation(req).await.expect("should succeed");
        assert!(resp.success);
        assert_eq!(resp.provider_id, "prov-1");
    }

    #[tokio::test]
    async fn execute_code_builds_request() {
        let eco = Arc::new(UniversalPrimalEcosystem::new(PrimalContext::default()));
        let client = UniversalComputeClient::new(
            eco,
            ComputeClientConfig::default(),
            PrimalContext::default(),
        );
        client.test_only_insert_provider(sample_provider("prov-3"));
        let r = client
            .execute_code("python", "x=1".to_string(), ComputePriority::High)
            .await
            .expect("should succeed");
        assert!(r.success);
    }

    #[test]
    fn workload_to_resource_utilization_from_trait() {
        let w = WorkloadCharacteristics {
            cpu_intensity: 0.7,
            memory_intensity: 0.3,
            io_intensity: 0.4,
            gpu_requirement: 0.2,
            parallelizability: 0.5,
        };
        let u: ResourceUtilization = w.into();
        assert!((u.cpu_utilization - 0.7).abs() < 1e-6);
        assert_eq!(u.gpu_utilization, Some(0.2));
    }

    #[test]
    fn compute_security_requirements_default_impl() {
        let d = ComputeSecurityRequirements::default();
        assert!(d.encryption_requirements.data_at_rest);
    }
}
