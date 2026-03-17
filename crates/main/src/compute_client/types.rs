// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Compute Client Types and Structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

// ============================================================================
// CONFIGURATION TYPES
// ============================================================================

/// Configuration for universal compute client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeClientConfig {
    /// Timeout for compute operations
    pub operation_timeout: Duration,

    /// Maximum retries for failed operations
    pub max_retries: u32,

    /// Preferred compute capabilities
    pub preferred_capabilities: Vec<ComputeCapabilityPreference>,

    /// Resource requirements
    pub resource_requirements: ResourceRequirements,

    /// Security requirements
    pub security_requirements: ComputeSecurityRequirements,
}

/// Compute capability preferences for intelligent routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeCapabilityPreference {
    /// Capability type
    pub capability: ComputeCapabilityType,

    /// Priority weight (0.0 - 1.0)
    pub weight: f64,

    /// Required vs optional
    pub required: bool,
}

/// Types of compute capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeCapabilityType {
    /// CPU-intensive computations
    CpuIntensive {
        /// CPU cores required
        cores: u32,
        /// Memory in GB
        memory_gb: u32,
        /// CPU architecture
        architecture: String,
    },

    /// GPU-accelerated computing
    GpuAccelerated {
        /// GPU memory in GB
        gpu_memory_gb: u32,
        /// CUDA support
        cuda_support: bool,
        /// Supported frameworks
        frameworks: Vec<String>,
    },

    /// Container runtime
    ContainerRuntime {
        /// Supported orchestrators
        orchestrators: Vec<String>,
        /// Isolation level
        isolation_level: String,
    },

    /// Serverless execution
    ServerlessExecution {
        /// Supported languages
        languages: Vec<String>,
        /// Cold start time in ms
        cold_start_ms: u64,
    },

    /// AI/ML specific compute
    MachineLearning {
        /// Supported ML frameworks
        frameworks: Vec<String>,
        /// Training support
        training_support: bool,
        /// Inference support
        inference_support: bool,
    },

    /// High-performance computing
    HighPerformanceComputing {
        /// Interconnect type
        interconnect: String,
        /// Parallel processing support
        parallel_processing: bool,
        /// Distributed compute support
        distributed_compute: bool,
    },
}

/// Resource requirements for compute operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU cores required
    pub cpu_cores: u32,

    /// Memory in GB
    pub memory_gb: u32,

    /// GPU units (optional)
    pub gpu_units: Option<u32>,

    /// Storage in GB
    pub storage_gb: u32,

    /// Maximum execution time
    pub max_execution_time: Duration,

    /// Network bandwidth requirements (Mbps)
    pub network_bandwidth_mbps: Option<f64>,
}

/// Security requirements for compute operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeSecurityRequirements {
    /// Isolation level
    pub isolation_level: IsolationLevel,

    /// Trusted execution environment
    pub trusted_execution: bool,

    /// Data encryption requirements
    pub encryption_requirements: EncryptionRequirements,

    /// Network security
    pub network_security: NetworkSecurityLevel,
}

/// Compute isolation levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// Process-level isolation
    Process,

    /// Container isolation
    Container,

    /// Virtual machine isolation
    VirtualMachine,

    /// Hardware-level isolation
    Hardware,
}

/// Encryption requirements for compute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionRequirements {
    /// Data at rest encryption
    pub data_at_rest: bool,

    /// Data in transit encryption
    pub data_in_transit: bool,

    /// Data in use encryption (homomorphic)
    pub data_in_use: bool,
}

/// Network security levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkSecurityLevel {
    /// Basic network security
    Basic,

    /// VPN-protected
    VpnProtected,

    /// Private network only
    PrivateNetwork,

    /// Air-gapped environment
    AirGapped,
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

/// Universal compute request - AI-first design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalComputeRequest {
    /// Unique request identifier
    pub request_id: Uuid,

    /// Operation type
    pub operation: ComputeOperation,

    /// Compute payload
    pub payload: ComputePayload,

    /// Resource requirements
    pub resources: ResourceRequirements,

    /// Security requirements
    pub security: ComputeSecurityRequirements,

    /// AI context for intelligent routing
    pub ai_context: AIComputeContext,

    /// Request metadata
    pub metadata: HashMap<String, String>,
}

/// Types of compute operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeOperation {
    /// Execute code
    Execute {
        /// Programming language
        language: String,
        /// Entry point
        entrypoint: String,
    },

    /// Train ML model
    TrainModel {
        /// ML framework
        framework: String,
        /// Model type
        model_type: String,
    },

    /// Run inference
    RunInference {
        /// Model identifier
        model_id: String,
        /// Batch size
        batch_size: u32,
    },

    /// Batch processing
    BatchProcess {
        /// Job type
        job_type: String,
        /// Parallelism level
        parallelism: u32,
    },

    /// Stream processing
    StreamProcess {
        /// Stream source
        stream_source: String,
        /// Processing window
        processing_window: Duration,
    },

    /// Custom workload
    CustomWorkload {
        /// Workload type
        workload_type: String,
        /// Workload configuration
        configuration: HashMap<String, serde_json::Value>,
    },
}

/// Compute payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputePayload {
    /// Code or configuration
    pub code: Option<String>,

    /// Input data
    pub input_data: Option<Vec<u8>>,

    /// Environment variables
    pub environment: HashMap<String, String>,

    /// Dependencies
    pub dependencies: Vec<String>,

    /// Configuration parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// AI context for intelligent compute routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIComputeContext {
    /// Expected workload characteristics
    pub workload_characteristics: WorkloadCharacteristics,

    /// Priority level
    pub priority: ComputePriority,

    /// Deadline for completion
    pub deadline: Option<DateTime<Utc>>,

    /// Cost vs performance preference
    pub cost_performance_preference: CostPerformancePreference,
}

/// Workload characteristics for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadCharacteristics {
    /// CPU intensity (0.0 - 1.0)
    pub cpu_intensity: f64,

    /// Memory intensity (0.0 - 1.0)
    pub memory_intensity: f64,

    /// I/O intensity (0.0 - 1.0)
    pub io_intensity: f64,

    /// GPU requirement (0.0 - 1.0)
    pub gpu_requirement: f64,

    /// Parallelizability (0.0 - 1.0)
    pub parallelizability: f64,
}

/// Compute priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputePriority {
    /// Low priority - can be delayed
    Low,

    /// Normal priority
    Normal,

    /// High priority - expedited processing
    High,

    /// Critical priority - immediate processing
    Critical,
}

/// Cost vs performance preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostPerformancePreference {
    /// Optimize for minimum cost
    MinimizeCost,

    /// Balance cost and performance
    Balanced,

    /// Optimize for maximum performance
    MaximizePerformance,

    /// Custom weights
    Custom {
        /// Weight for cost (0.0 - 1.0)
        cost_weight: f64,
        /// Weight for performance (0.0 - 1.0)
        performance_weight: f64,
    },
}

/// Universal compute response - AI-first design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalComputeResponse {
    /// Request identifier
    pub request_id: Uuid,

    /// Operation success
    pub success: bool,

    /// Compute results
    pub results: Option<ComputeResults>,

    /// Provider that handled the request
    pub provider_id: String,

    /// Performance metrics
    pub performance: ComputePerformanceMetrics,

    /// AI insights and recommendations
    pub ai_insights: AIComputeInsights,

    /// Error information (if applicable)
    pub error: Option<String>,
}

/// Compute operation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResults {
    /// Output data
    pub output_data: Option<Vec<u8>>,

    /// Return code
    pub return_code: i32,

    /// Standard output
    pub stdout: String,

    /// Standard error
    pub stderr: String,

    /// Result metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Performance metrics for compute operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputePerformanceMetrics {
    /// Total execution time
    pub execution_time: Duration,

    /// Queue wait time
    pub queue_time: Duration,

    /// Resource utilization
    pub resource_utilization: ResourceUtilization,

    /// Cost breakdown
    pub cost_breakdown: CostBreakdown,

    /// Provider health during operation
    pub provider_health: f64,
}

/// Resource utilization prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    /// CPU utilization (0.0 - 1.0)
    pub cpu_utilization: f64,

    /// Memory utilization (0.0 - 1.0)
    pub memory_utilization: f64,

    /// GPU utilization (0.0 - 1.0)
    pub gpu_utilization: Option<f64>,

    /// Network utilization (0.0 - 1.0)
    pub network_utilization: Option<f64>,
}

/// Cost breakdown for compute operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    /// CPU cost
    pub cpu_cost: f64,

    /// Memory cost
    pub memory_cost: f64,

    /// GPU cost (if applicable)
    pub gpu_cost: Option<f64>,

    /// Storage cost
    pub storage_cost: f64,

    /// Network cost
    pub network_cost: f64,

    /// Total cost
    pub total_cost: f64,
}

/// AI insights and recommendations for compute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIComputeInsights {
    /// Confidence in operation success
    pub confidence_score: f64,

    /// Performance optimizations
    pub performance_optimizations: Vec<String>,

    /// Cost optimizations
    pub cost_optimizations: Vec<String>,

    /// Alternative providers
    pub alternative_providers: Vec<String>,

    /// Workload analysis
    pub workload_analysis: WorkloadAnalysis,
}

/// Workload analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadAnalysis {
    /// Detected workload patterns
    pub patterns: Vec<String>,

    /// Resource efficiency score
    pub efficiency_score: f64,

    /// Bottleneck analysis
    pub bottlenecks: Vec<String>,

    /// Optimization recommendations
    pub recommendations: Vec<String>,
}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

impl Default for ComputeClientConfig {
    fn default() -> Self {
        Self {
            operation_timeout: Duration::from_secs(3600), // 1 hour
            max_retries: 3,
            preferred_capabilities: vec![
                ComputeCapabilityPreference {
                    capability: ComputeCapabilityType::ContainerRuntime {
                        // Orchestrator is discovered at runtime via ComputeProvider trait
                        // No hardcoded vendor names - infant primal pattern
                        orchestrators: vec![], // Auto-detected
                        isolation_level: "container".to_string(),
                    },
                    weight: 0.8,
                    required: true,
                },
                ComputeCapabilityPreference {
                    capability: ComputeCapabilityType::GpuAccelerated {
                        gpu_memory_gb: 8,
                        cuda_support: true,
                        frameworks: vec!["pytorch".to_string(), "tensorflow".to_string()],
                    },
                    weight: 0.6,
                    required: false,
                },
            ],
            resource_requirements: ResourceRequirements {
                cpu_cores: 2,
                memory_gb: 4,
                gpu_units: None,
                storage_gb: 10,
                max_execution_time: Duration::from_secs(300),
                network_bandwidth_mbps: Some(100.0),
            },
            security_requirements: ComputeSecurityRequirements {
                isolation_level: IsolationLevel::Container,
                trusted_execution: false,
                encryption_requirements: EncryptionRequirements {
                    data_at_rest: true,
                    data_in_transit: true,
                    data_in_use: false,
                },
                network_security: NetworkSecurityLevel::Basic,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_client_config_default() {
        let config = ComputeClientConfig::default();
        assert_eq!(config.operation_timeout, Duration::from_secs(3600));
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.preferred_capabilities.len(), 2);
        assert_eq!(config.resource_requirements.cpu_cores, 2);
        assert_eq!(config.resource_requirements.memory_gb, 4);
        assert!(config.resource_requirements.gpu_units.is_none());
    }

    #[test]
    fn test_compute_client_config_serde() {
        let config = ComputeClientConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ComputeClientConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.max_retries, 3);
        assert_eq!(deserialized.preferred_capabilities.len(), 2);
    }

    #[test]
    fn test_compute_capability_types_serde() {
        let cpu = ComputeCapabilityType::CpuIntensive {
            cores: 16,
            memory_gb: 64,
            architecture: "x86_64".to_string(),
        };
        let json = serde_json::to_string(&cpu).unwrap();
        assert!(json.contains("CpuIntensive"));

        let gpu = ComputeCapabilityType::GpuAccelerated {
            gpu_memory_gb: 24,
            cuda_support: true,
            frameworks: vec!["pytorch".to_string()],
        };
        let json = serde_json::to_string(&gpu).unwrap();
        assert!(json.contains("GpuAccelerated"));

        let ml = ComputeCapabilityType::MachineLearning {
            frameworks: vec!["pytorch".to_string()],
            training_support: true,
            inference_support: true,
        };
        let json = serde_json::to_string(&ml).unwrap();
        assert!(json.contains("MachineLearning"));

        let serverless = ComputeCapabilityType::ServerlessExecution {
            languages: vec!["rust".to_string()],
            cold_start_ms: 50,
        };
        let json = serde_json::to_string(&serverless).unwrap();
        assert!(json.contains("ServerlessExecution"));

        let hpc = ComputeCapabilityType::HighPerformanceComputing {
            interconnect: "infiniband".to_string(),
            parallel_processing: true,
            distributed_compute: true,
        };
        let json = serde_json::to_string(&hpc).unwrap();
        assert!(json.contains("HighPerformanceComputing"));
    }

    #[test]
    fn test_isolation_level_serde() {
        let levels = vec![
            IsolationLevel::Process,
            IsolationLevel::Container,
            IsolationLevel::VirtualMachine,
            IsolationLevel::Hardware,
        ];
        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let deserialized: IsolationLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{deserialized:?}"), format!("{level:?}"));
        }
    }

    #[test]
    fn test_network_security_level_serde() {
        let levels = vec![
            NetworkSecurityLevel::Basic,
            NetworkSecurityLevel::VpnProtected,
            NetworkSecurityLevel::PrivateNetwork,
            NetworkSecurityLevel::AirGapped,
        ];
        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let deserialized: NetworkSecurityLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{deserialized:?}"), format!("{level:?}"));
        }
    }

    #[test]
    fn test_compute_operation_serde() {
        let ops = vec![
            ComputeOperation::Execute {
                language: "rust".to_string(),
                entrypoint: "main".to_string(),
            },
            ComputeOperation::TrainModel {
                framework: "pytorch".to_string(),
                model_type: "transformer".to_string(),
            },
            ComputeOperation::RunInference {
                model_id: "model-123".to_string(),
                batch_size: 32,
            },
            ComputeOperation::BatchProcess {
                job_type: "etl".to_string(),
                parallelism: 4,
            },
        ];
        for op in ops {
            let json = serde_json::to_string(&op).unwrap();
            let deserialized: ComputeOperation = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{deserialized:?}"), format!("{op:?}"));
        }
    }

    #[test]
    fn test_compute_priority_serde() {
        let priorities = vec![
            ComputePriority::Low,
            ComputePriority::Normal,
            ComputePriority::High,
            ComputePriority::Critical,
        ];
        for priority in priorities {
            let json = serde_json::to_string(&priority).unwrap();
            let deserialized: ComputePriority = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{deserialized:?}"), format!("{priority:?}"));
        }
    }

    #[test]
    fn test_cost_performance_preference_serde() {
        let prefs = vec![
            CostPerformancePreference::MinimizeCost,
            CostPerformancePreference::Balanced,
            CostPerformancePreference::MaximizePerformance,
            CostPerformancePreference::Custom {
                cost_weight: 0.3,
                performance_weight: 0.7,
            },
        ];
        for pref in prefs {
            let json = serde_json::to_string(&pref).unwrap();
            let deserialized: CostPerformancePreference = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{deserialized:?}"), format!("{pref:?}"));
        }
    }

    #[test]
    fn test_compute_results_serde() {
        let results = ComputeResults {
            output_data: Some(vec![1, 2, 3]),
            return_code: 0,
            stdout: "Success".to_string(),
            stderr: String::new(),
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&results).unwrap();
        let deserialized: ComputeResults = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.return_code, 0);
        assert_eq!(deserialized.stdout, "Success");
    }

    #[test]
    fn test_resource_utilization_serde() {
        let util = ResourceUtilization {
            cpu_utilization: 0.75,
            memory_utilization: 0.5,
            gpu_utilization: Some(0.9),
            network_utilization: None,
        };
        let json = serde_json::to_string(&util).unwrap();
        let deserialized: ResourceUtilization = serde_json::from_str(&json).unwrap();
        assert!((deserialized.cpu_utilization - 0.75).abs() < f64::EPSILON);
        assert!(deserialized.gpu_utilization.is_some());
        assert!(deserialized.network_utilization.is_none());
    }

    #[test]
    fn test_cost_breakdown_serde() {
        let cost = CostBreakdown {
            cpu_cost: 10.0,
            memory_cost: 5.0,
            gpu_cost: Some(20.0),
            storage_cost: 2.0,
            network_cost: 1.0,
            total_cost: 38.0,
        };
        let json = serde_json::to_string(&cost).unwrap();
        let deserialized: CostBreakdown = serde_json::from_str(&json).unwrap();
        assert!((deserialized.total_cost - 38.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_workload_characteristics_serde() {
        let wc = WorkloadCharacteristics {
            cpu_intensity: 0.8,
            memory_intensity: 0.6,
            io_intensity: 0.2,
            gpu_requirement: 0.9,
            parallelizability: 0.7,
        };
        let json = serde_json::to_string(&wc).unwrap();
        let deserialized: WorkloadCharacteristics = serde_json::from_str(&json).unwrap();
        assert!((deserialized.cpu_intensity - 0.8).abs() < f64::EPSILON);
        assert!((deserialized.gpu_requirement - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_encryption_requirements_serde() {
        let enc = EncryptionRequirements {
            data_at_rest: true,
            data_in_transit: true,
            data_in_use: false,
        };
        let json = serde_json::to_string(&enc).unwrap();
        let deserialized: EncryptionRequirements = serde_json::from_str(&json).unwrap();
        assert!(deserialized.data_at_rest);
        assert!(deserialized.data_in_transit);
        assert!(!deserialized.data_in_use);
    }
}
