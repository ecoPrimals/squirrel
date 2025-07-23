//! Universal AI Provider Configuration
//!
//! Configuration structures and defaults for capability-based AI provider discovery.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Remove the conflicting import - use our local CostTier
use super::types::{AICapability, CostProfile, DataFormat, ProcessingType, QualityProfile};

/// Universal AI provider configuration - purely capability-based
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalAIConfig {
    /// Capability discovery configuration
    pub discovery: CapabilityDiscoveryConfig,
    /// Performance requirements
    pub requirements: CapabilityRequirements,
    /// Self-capabilities (what we provide to the ecosystem)
    pub self_capabilities: Vec<AICapability>,
    /// Local inference configuration
    pub local_inference: LocalInferenceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDiscoveryConfig {
    /// Discovery methods to use
    pub methods: Vec<DiscoveryMethod>,
    /// How often to refresh capabilities
    pub refresh_interval_ms: u64,
    /// Query timeout for discovery
    pub query_timeout_ms: u64,
    /// Cache duration for capabilities
    pub cache_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// Environment variable announcements
    Environment { prefix: String },
    /// Network service discovery
    Network { port_range: (u16, u16) },
    /// File-based capability announcements
    FileSystem { directory: PathBuf },
    /// Process-based discovery
    Process { search_paths: Vec<PathBuf> },
    /// Custom discovery method
    Custom {
        name: String,
        config: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequirements {
    /// Required capabilities for different tasks
    pub text_generation: CapabilityRequirement,
    pub code_generation: CapabilityRequirement,
    pub question_answering: CapabilityRequirement,
    pub analysis: CapabilityRequirement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequirement {
    pub max_latency_ms: u64,
    pub min_accuracy: f64,
    pub min_reliability: f64,
    pub max_cost_per_request: f64,
    pub preferred_processing_type: Option<ProcessingType>,
    pub required_specializations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalInferenceConfig {
    /// Enable local inference capabilities
    pub enabled: bool,
    /// Local model directory
    pub models_directory: Option<PathBuf>,
    /// Python integration for HuggingFace
    pub python_integration: PythonIntegrationConfig,
    /// Native inference (llama.cpp, etc.)
    pub native_inference: NativeInferenceConfig,
    /// Container-based inference
    pub container_inference: ContainerInferenceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonIntegrationConfig {
    pub enabled: bool,
    pub python_executable: Option<String>,
    pub virtual_env: Option<PathBuf>,
    pub huggingface_cache: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeInferenceConfig {
    pub enabled: bool,
    pub supported_formats: Vec<String>, // gguf, ggml, etc.
    pub max_model_size_gb: f64,
    pub gpu_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInferenceConfig {
    pub enabled: bool,
    pub container_runtime: String, // docker, podman, etc.
    pub max_containers: u32,
}

impl Default for UniversalAIConfig {
    fn default() -> Self {
        Self {
            discovery: CapabilityDiscoveryConfig {
                methods: vec![
                    DiscoveryMethod::Environment {
                        prefix: "CAPABILITY_".to_string(),
                    },
                    DiscoveryMethod::Network {
                        port_range: (8000, 9000),
                    },
                    DiscoveryMethod::Process {
                        search_paths: vec![PathBuf::from("/usr/local/bin"), PathBuf::from("./bin")],
                    },
                ],
                refresh_interval_ms: 30000, // 30 seconds
                query_timeout_ms: 5000,
                cache_duration_ms: 300000, // 5 minutes
            },
            requirements: CapabilityRequirements {
                text_generation: CapabilityRequirement {
                    max_latency_ms: 10000,
                    min_accuracy: 0.80,
                    min_reliability: 0.90,
                    max_cost_per_request: 0.01,
                    preferred_processing_type: Some(ProcessingType::Interactive),
                    required_specializations: vec![],
                },
                code_generation: CapabilityRequirement {
                    max_latency_ms: 15000,
                    min_accuracy: 0.85,
                    min_reliability: 0.85,
                    max_cost_per_request: 0.02,
                    preferred_processing_type: Some(ProcessingType::Synchronous),
                    required_specializations: vec!["programming".to_string()],
                },
                question_answering: CapabilityRequirement {
                    max_latency_ms: 8000,
                    min_accuracy: 0.90,
                    min_reliability: 0.95,
                    max_cost_per_request: 0.005,
                    preferred_processing_type: Some(ProcessingType::Interactive),
                    required_specializations: vec![],
                },
                analysis: CapabilityRequirement {
                    max_latency_ms: 20000,
                    min_accuracy: 0.75,
                    min_reliability: 0.80,
                    max_cost_per_request: 0.03,
                    preferred_processing_type: Some(ProcessingType::Batch),
                    required_specializations: vec!["analysis".to_string()],
                },
            },
            self_capabilities: vec![AICapability {
                capability_type: "ai-routing".to_string(),
                input_formats: vec![DataFormat::JSON],
                output_formats: vec![DataFormat::JSON, DataFormat::Stream],
                processing_type: ProcessingType::Interactive,
                quality_profile: QualityProfile {
                    accuracy: 0.95,
                    consistency: 0.90,
                    context_understanding: 0.85,
                    specializations: vec!["routing".to_string(), "optimization".to_string()],
                },
                cost_profile: CostProfile {
                    cost_per_request: 0.0,
                    cost_per_unit: 0.0,
                    is_free: true,
                    tier: super::types::CostTier::Free,
                },
            }],
            local_inference: LocalInferenceConfig {
                enabled: true,
                models_directory: Some(PathBuf::from("./models")),
                python_integration: PythonIntegrationConfig {
                    enabled: true,
                    python_executable: None,
                    virtual_env: None,
                    huggingface_cache: None,
                },
                native_inference: NativeInferenceConfig {
                    enabled: true,
                    supported_formats: vec!["gguf".to_string(), "ggml".to_string()],
                    max_model_size_gb: 8.0,
                    gpu_enabled: false,
                },
                container_inference: ContainerInferenceConfig {
                    enabled: false,
                    container_runtime: "docker".to_string(),
                    max_containers: 3,
                },
            },
        }
    }
}
