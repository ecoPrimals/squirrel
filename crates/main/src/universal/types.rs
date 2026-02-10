// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_primal_type_serialization() {
        let pt = PrimalType::AI;
        let json = serde_json::to_string(&pt).expect("serialize");
        assert_eq!(json, "\"AI\"");
        let deser: PrimalType = serde_json::from_str("\"Storage\"").expect("deserialize");
        assert_eq!(deser, PrimalType::Storage);
    }

    #[test]
    fn test_primal_type_display() {
        assert_eq!(PrimalType::AI.to_string(), "AI");
        assert_eq!(PrimalType::Storage.to_string(), "Storage");
        assert_eq!(PrimalType::Compute.to_string(), "Compute");
        assert_eq!(PrimalType::Network.to_string(), "Network");
        assert_eq!(PrimalType::Security.to_string(), "Security");
        assert_eq!(PrimalType::Coordination.to_string(), "Coordination");
    }

    #[test]
    fn test_primal_type_all_variants_serde() {
        for variant in [
            PrimalType::AI,
            PrimalType::Storage,
            PrimalType::Compute,
            PrimalType::Network,
            PrimalType::Security,
            PrimalType::Coordination,
        ] {
            let json = serde_json::to_string(&variant).expect("serialize");
            let deser: PrimalType = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(deser, variant);
        }
    }

    #[test]
    fn test_primal_type_hash() {
        let mut set = HashSet::new();
        set.insert(PrimalType::AI);
        set.insert(PrimalType::Storage);
        assert!(set.contains(&PrimalType::AI));
        assert!(!set.contains(&PrimalType::Compute));
    }

    #[test]
    fn test_primal_info_serde() {
        let info = PrimalInfo {
            primal_id: "squirrel-1".to_string(),
            instance_id: "inst-123".to_string(),
            primal_type: PrimalType::AI,
            capabilities: vec![PrimalCapability::ServiceDiscovery],
            endpoints: vec!["http://localhost:9010".to_string()],
            metadata: HashMap::from([("version".to_string(), "1.0".to_string())]),
            id: None,
            version: "1.0.0".to_string(),
        };
        let json = serde_json::to_string(&info).expect("serialize");
        let deser: PrimalInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.primal_id, "squirrel-1");
        assert_eq!(deser.primal_type, PrimalType::AI);
        assert_eq!(deser.endpoints.len(), 1);
    }

    #[test]
    fn test_primal_capability_model_inference_serde() {
        let cap = PrimalCapability::ModelInference {
            models: vec!["gpt-4".to_string()],
        };
        let json = serde_json::to_string(&cap).expect("serialize");
        let deser: PrimalCapability = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser, cap);
    }

    #[test]
    fn test_primal_capability_data_storage_serde() {
        let cap = PrimalCapability::DataStorage {
            storage_type: "object".to_string(),
            max_size_bytes: 1_000_000,
        };
        let json = serde_json::to_string(&cap).expect("serialize");
        let deser: PrimalCapability = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser, cap);
    }

    #[test]
    fn test_primal_capability_custom_serde() {
        let cap = PrimalCapability::Custom {
            name: "test".to_string(),
            description: "test capability".to_string(),
            metadata: HashMap::new(),
            attributes: HashMap::new(),
        };
        let json = serde_json::to_string(&cap).expect("serialize");
        let deser: PrimalCapability = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser, cap);
    }

    #[test]
    fn test_primal_capability_hash() {
        let mut set = HashSet::new();
        set.insert(PrimalCapability::ServiceDiscovery);
        set.insert(PrimalCapability::LoadBalancing);
        assert!(set.contains(&PrimalCapability::ServiceDiscovery));
        assert!(!set.contains(&PrimalCapability::CircuitBreaking));
    }

    #[test]
    fn test_squirrel_capability_serde() {
        for cap in [
            SquirrelCapability::AICoordination,
            SquirrelCapability::MCPProtocol,
            SquirrelCapability::ContextAwareness,
            SquirrelCapability::MultiTenancy,
            SquirrelCapability::ServiceMeshIntegration,
            SquirrelCapability::DynamicRouting,
            SquirrelCapability::HealthMonitoring,
            SquirrelCapability::GracefulShutdown,
            SquirrelCapability::LoadBalancing,
            SquirrelCapability::CircuitBreaking,
        ] {
            let json = serde_json::to_string(&cap).expect("serialize");
            let deser: SquirrelCapability = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(deser, cap);
        }
    }

    #[test]
    fn test_primal_capability_gpu_acceleration_hash() {
        let cap = PrimalCapability::GpuAcceleration {
            gpu_types: vec!["nvidia".to_string()],
            cuda_support: true,
        };
        let mut set = HashSet::new();
        set.insert(cap.clone());
        assert!(set.contains(&cap));
    }

    #[test]
    fn test_primal_capability_all_unit_variants() {
        let caps = vec![
            PrimalCapability::ServiceDiscovery,
            PrimalCapability::LoadBalancing,
            PrimalCapability::CircuitBreaking,
        ];
        for cap in caps {
            let json = serde_json::to_string(&cap).expect("serialize");
            let deser: PrimalCapability = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(deser, cap);
        }
    }
}
