//! Core type definitions for the universal primal system
//!
//! This module contains fundamental type definitions including primal types,
//! capabilities, and basic information structures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Information about a primal's identity and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    pub primal_id: String,
    pub instance_id: String,
    pub primal_type: PrimalType,
    pub capabilities: Vec<PrimalCapability>,
    pub endpoints: Vec<String>,
    pub metadata: HashMap<String, String>,
    /// Convenience alias for `primal_id` (for compatibility)
    #[serde(skip)]
    pub id: Option<String>,
    /// Version information
    pub version: String,
}

/// Types of primals in the ecosystem
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalType {
    AI,
    Storage,
    Compute,
    Network,
    Security,
    Coordination,
}

impl std::fmt::Display for PrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AI => write!(f, "AI"),
            Self::Storage => write!(f, "Storage"),
            Self::Compute => write!(f, "Compute"),
            Self::Network => write!(f, "Network"),
            Self::Security => write!(f, "Security"),
            Self::Coordination => write!(f, "Coordination"),
        }
    }
}

/// Capabilities that primals can provide
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalCapability {
    ModelInference {
        models: Vec<String>,
    },
    ContextManagement {
        max_context_length: usize,
    },
    MemoryManagement {
        persistent: bool,
    },
    ToolExecution {
        tools: Vec<String>,
    },
    DataStorage {
        storage_type: String,
        max_size_bytes: u64,
    },
    Authentication {
        methods: Vec<String>,
    },
    ServiceDiscovery,
    LoadBalancing,
    CircuitBreaking,
    RateLimiting {
        max_requests_per_second: u32,
    },
    Monitoring {
        metrics: Vec<String>,
    },
    Logging {
        levels: Vec<String>,
    },
    ContainerRuntime {
        container_types: Vec<String>,
        orchestrators: Vec<String>,
    },
    ServerlessExecution {
        languages: Vec<String>,
    },
    GpuAcceleration {
        gpu_types: Vec<String>,
        cuda_support: bool,
    },
    ObjectStorage {
        storage_types: Vec<String>,
        backends: Vec<String>,
    },
    FileSystem {
        fs_types: Vec<String>,
    },
    Encryption {
        algorithms: Vec<String>,
    },
    KeyManagement {
        key_types: Vec<String>,
        hsm_support: bool,
    },
    NaturalLanguage {
        languages: Vec<String>,
    },
    AgentFramework {
        frameworks: Vec<String>,
        mcp_support: bool,
    },
    Custom {
        name: String,
        description: String,
        metadata: HashMap<String, String>,
        attributes: HashMap<String, String>,
    },
}

impl Hash for PrimalCapability {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::ModelInference { models } => {
                for model in models {
                    model.hash(state);
                }
            }
            Self::ContextManagement { max_context_length } => {
                max_context_length.hash(state);
            }
            Self::MemoryManagement { persistent } => {
                persistent.hash(state);
            }
            Self::ToolExecution { tools } => {
                for tool in tools {
                    tool.hash(state);
                }
            }
            Self::DataStorage {
                storage_type,
                max_size_bytes,
            } => {
                storage_type.hash(state);
                max_size_bytes.hash(state);
            }
            Self::Authentication { methods } => {
                for method in methods {
                    method.hash(state);
                }
            }
            Self::RateLimiting {
                max_requests_per_second,
            } => {
                max_requests_per_second.hash(state);
            }
            Self::Monitoring { metrics } => {
                for metric in metrics {
                    metric.hash(state);
                }
            }
            Self::Logging { levels } => {
                for level in levels {
                    level.hash(state);
                }
            }
            Self::ContainerRuntime {
                container_types,
                orchestrators,
            } => {
                for ct in container_types {
                    ct.hash(state);
                }
                for orch in orchestrators {
                    orch.hash(state);
                }
            }
            Self::ServerlessExecution { languages } => {
                for lang in languages {
                    lang.hash(state);
                }
            }
            Self::GpuAcceleration {
                gpu_types,
                cuda_support,
            } => {
                for gt in gpu_types {
                    gt.hash(state);
                }
                cuda_support.hash(state);
            }
            Self::ObjectStorage {
                storage_types,
                backends,
            } => {
                for st in storage_types {
                    st.hash(state);
                }
                for backend in backends {
                    backend.hash(state);
                }
            }
            Self::Encryption { algorithms } => {
                for alg in algorithms {
                    alg.hash(state);
                }
            }
            Self::KeyManagement {
                key_types,
                hsm_support,
            } => {
                for kt in key_types {
                    kt.hash(state);
                }
                hsm_support.hash(state);
            }
            Self::Custom {
                name, description, ..
            } => {
                name.hash(state);
                description.hash(state);
            }
            _ => {}
        }
    }
}

/// Squirrel-specific capabilities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SquirrelCapability {
    AICoordination,
    MCPProtocol,
    ContextAwareness,
    MultiTenancy,
    ServiceMeshIntegration,
    DynamicRouting,
    HealthMonitoring,
    GracefulShutdown,
    LoadBalancing,
    CircuitBreaking,
}
