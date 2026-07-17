// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Compute Client Types and Structures
//!
//! Configuration types for the universal compute client.  Request/response
//! types live in [`super::compute_request_types`].

pub use super::compute_request_types::*;

use serde::{Deserialize, Serialize};
use std::time::Duration;

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
    use std::collections::HashMap;

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
        let json = serde_json::to_string(&config).expect("should succeed");
        let deserialized: ComputeClientConfig =
            serde_json::from_str(&json).expect("should succeed");
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
        let json = serde_json::to_string(&cpu).expect("should succeed");
        assert!(json.contains("CpuIntensive"));

        let gpu = ComputeCapabilityType::GpuAccelerated {
            gpu_memory_gb: 24,
            cuda_support: true,
            frameworks: vec!["pytorch".to_string()],
        };
        let json = serde_json::to_string(&gpu).expect("should succeed");
        assert!(json.contains("GpuAccelerated"));

        let ml = ComputeCapabilityType::MachineLearning {
            frameworks: vec!["pytorch".to_string()],
            training_support: true,
            inference_support: true,
        };
        let json = serde_json::to_string(&ml).expect("should succeed");
        assert!(json.contains("MachineLearning"));

        let serverless = ComputeCapabilityType::ServerlessExecution {
            languages: vec!["rust".to_string()],
            cold_start_ms: 50,
        };
        let json = serde_json::to_string(&serverless).expect("should succeed");
        assert!(json.contains("ServerlessExecution"));

        let hpc = ComputeCapabilityType::HighPerformanceComputing {
            interconnect: "infiniband".to_string(),
            parallel_processing: true,
            distributed_compute: true,
        };
        let json = serde_json::to_string(&hpc).expect("should succeed");
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
            let json = serde_json::to_string(&level).expect("should succeed");
            let deserialized: IsolationLevel = serde_json::from_str(&json).expect("should succeed");
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
            let json = serde_json::to_string(&level).expect("should succeed");
            let deserialized: NetworkSecurityLevel =
                serde_json::from_str(&json).expect("should succeed");
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
            let json = serde_json::to_string(&op).expect("should succeed");
            let deserialized: ComputeOperation =
                serde_json::from_str(&json).expect("should succeed");
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
            let json = serde_json::to_string(&priority).expect("should succeed");
            let deserialized: ComputePriority =
                serde_json::from_str(&json).expect("should succeed");
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
            let json = serde_json::to_string(&pref).expect("should succeed");
            let deserialized: CostPerformancePreference =
                serde_json::from_str(&json).expect("should succeed");
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
        let json = serde_json::to_string(&results).expect("should succeed");
        let deserialized: ComputeResults = serde_json::from_str(&json).expect("should succeed");
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
        let json = serde_json::to_string(&util).expect("should succeed");
        let deserialized: ResourceUtilization =
            serde_json::from_str(&json).expect("should succeed");
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
        let json = serde_json::to_string(&cost).expect("should succeed");
        let deserialized: CostBreakdown = serde_json::from_str(&json).expect("should succeed");
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
        let json = serde_json::to_string(&wc).expect("should succeed");
        let deserialized: WorkloadCharacteristics =
            serde_json::from_str(&json).expect("should succeed");
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
        let json = serde_json::to_string(&enc).expect("should succeed");
        let deserialized: EncryptionRequirements =
            serde_json::from_str(&json).expect("should succeed");
        assert!(deserialized.data_at_rest);
        assert!(deserialized.data_in_transit);
        assert!(!deserialized.data_in_use);
    }
}
