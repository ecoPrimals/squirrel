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
        cores: u32,
        memory_gb: u32,
        architecture: String,
    },

    /// GPU-accelerated computing
    GpuAccelerated {
        gpu_memory_gb: u32,
        cuda_support: bool,
        frameworks: Vec<String>,
    },

    /// Container runtime
    ContainerRuntime {
        orchestrators: Vec<String>,
        isolation_level: String,
    },

    /// Serverless execution
    ServerlessExecution {
        languages: Vec<String>,
        cold_start_ms: u64,
    },

    /// AI/ML specific compute
    MachineLearning {
        frameworks: Vec<String>,
        training_support: bool,
        inference_support: bool,
    },

    /// High-performance computing
    HighPerformanceComputing {
        interconnect: String,
        parallel_processing: bool,
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
        language: String,
        entrypoint: String,
    },

    /// Train ML model
    TrainModel {
        framework: String,
        model_type: String,
    },

    /// Run inference
    RunInference { model_id: String, batch_size: u32 },

    /// Batch processing
    BatchProcess { job_type: String, parallelism: u32 },

    /// Stream processing
    StreamProcess {
        stream_source: String,
        processing_window: Duration,
    },

    /// Custom workload
    CustomWorkload {
        workload_type: String,
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
        cost_weight: f64,
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
                        // Example orchestrators that COULD be used
                        // Actual orchestrator is discovered at runtime
                        orchestrators: vec!["kubernetes".to_string()],
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
