// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Primal capability types.

use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// Universal capabilities that any primal can provide
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalCapability {
    // Security capabilities (BearDog)
    /// Authentication with supported methods
    Authentication {
        /// List of supported authentication methods
        methods: Vec<String>,
    },
    /// Encryption with supported algorithms
    Encryption {
        /// List of supported encryption algorithms
        algorithms: Vec<String>,
    },
    /// Key management with HSM support
    KeyManagement {
        /// Whether HSM (Hardware Security Module) is supported
        hsm_support: bool,
    },
    /// Threat detection with ML capabilities
    ThreatDetection {
        /// Whether machine learning is enabled for threat detection
        ml_enabled: bool,
    },
    /// Audit logging with compliance standards
    AuditLogging {
        /// List of supported compliance standards
        compliance: Vec<String>,
    },
    /// Authorization and access control
    Authorization {
        /// Whether RBAC (Role-Based Access Control) is supported
        rbac_support: bool,
    },

    // Storage capabilities (NestGate)
    /// File system support
    FileSystem {
        /// Whether ZFS file system is supported
        supports_zfs: bool,
    },
    /// Object storage with backends
    ObjectStorage {
        /// List of supported storage backends
        backends: Vec<String>,
    },
    /// Data replication
    DataReplication {
        /// Consistency model for data replication
        consistency: String,
    },
    /// Backup capabilities
    Backup {
        /// Whether incremental backups are supported
        incremental: bool,
    },
    /// Data archiving
    DataArchiving {
        /// List of supported compression algorithms
        compression: Vec<String>,
    },

    // Compute capabilities (Toadstool)
    /// Container runtime support
    ContainerRuntime {
        /// List of supported container orchestrators
        orchestrators: Vec<String>,
    },
    /// Serverless execution
    ServerlessExecution {
        /// List of supported programming languages
        languages: Vec<String>,
    },
    /// GPU acceleration
    GpuAcceleration {
        /// Whether CUDA is supported
        cuda_support: bool,
    },
    /// Load balancing
    LoadBalancing {
        /// List of supported load balancing algorithms
        algorithms: Vec<String>,
    },
    /// Auto-scaling
    AutoScaling {
        /// List of supported scaling metrics
        metrics: Vec<String>,
    },

    // AI capabilities (Squirrel)
    /// Model inference
    ModelInference {
        /// List of supported AI models
        models: Vec<String>,
    },
    /// Agent framework
    AgentFramework {
        /// Whether MCP (Model Context Protocol) is supported
        mcp_support: bool,
    },
    /// Machine learning
    MachineLearning {
        /// Whether training is supported
        training_support: bool,
    },
    /// Natural language processing
    NaturalLanguage {
        /// List of supported languages
        languages: Vec<String>,
    },
    /// Computer vision
    ComputerVision {
        /// List of supported computer vision models
        models: Vec<String>,
    },

    // Networking capabilities
    /// Service discovery
    ServiceDiscovery {
        /// List of supported discovery protocols
        protocols: Vec<String>,
    },
    /// Network routing
    NetworkRouting {
        /// List of supported routing protocols
        protocols: Vec<String>,
    },
    /// Proxy services
    ProxyServices {
        /// List of supported proxy types
        types: Vec<String>,
    },
    /// VPN capabilities
    VpnServices {
        /// List of supported VPN protocols
        protocols: Vec<String>,
    },

    // Generic capabilities
    /// Custom capability
    Custom {
        /// Name of the custom capability
        name: String,
        /// Custom attributes for the capability
        attributes: String, // Changed from HashMap to String to fix Hash issues
    },
}

impl Hash for PrimalCapability {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            PrimalCapability::Authentication { methods } => {
                "authentication".hash(state);
                methods.hash(state);
            }
            PrimalCapability::Encryption { algorithms } => {
                "encryption".hash(state);
                algorithms.hash(state);
            }
            PrimalCapability::KeyManagement { hsm_support } => {
                "key_management".hash(state);
                hsm_support.hash(state);
            }
            PrimalCapability::ThreatDetection { ml_enabled } => {
                "threat_detection".hash(state);
                ml_enabled.hash(state);
            }
            PrimalCapability::AuditLogging { compliance } => {
                "audit_logging".hash(state);
                compliance.hash(state);
            }
            PrimalCapability::Authorization { rbac_support } => {
                "authorization".hash(state);
                rbac_support.hash(state);
            }
            PrimalCapability::FileSystem { supports_zfs } => {
                "file_system".hash(state);
                supports_zfs.hash(state);
            }
            PrimalCapability::ObjectStorage { backends } => {
                "object_storage".hash(state);
                backends.hash(state);
            }
            PrimalCapability::DataReplication { consistency } => {
                "data_replication".hash(state);
                consistency.hash(state);
            }
            PrimalCapability::Backup { incremental } => {
                "backup".hash(state);
                incremental.hash(state);
            }
            PrimalCapability::DataArchiving { compression } => {
                "data_archiving".hash(state);
                compression.hash(state);
            }
            PrimalCapability::ContainerRuntime { orchestrators } => {
                "container_runtime".hash(state);
                orchestrators.hash(state);
            }
            PrimalCapability::ServerlessExecution { languages } => {
                "serverless_execution".hash(state);
                languages.hash(state);
            }
            PrimalCapability::GpuAcceleration { cuda_support } => {
                "gpu_acceleration".hash(state);
                cuda_support.hash(state);
            }
            PrimalCapability::LoadBalancing { algorithms } => {
                "load_balancing".hash(state);
                algorithms.hash(state);
            }
            PrimalCapability::AutoScaling { metrics } => {
                "auto_scaling".hash(state);
                metrics.hash(state);
            }
            PrimalCapability::ModelInference { models } => {
                "model_inference".hash(state);
                models.hash(state);
            }
            PrimalCapability::AgentFramework { mcp_support } => {
                "agent_framework".hash(state);
                mcp_support.hash(state);
            }
            PrimalCapability::MachineLearning { training_support } => {
                "machine_learning".hash(state);
                training_support.hash(state);
            }
            PrimalCapability::NaturalLanguage { languages } => {
                "natural_language".hash(state);
                languages.hash(state);
            }
            PrimalCapability::ComputerVision { models } => {
                "computer_vision".hash(state);
                models.hash(state);
            }
            PrimalCapability::ServiceDiscovery { protocols } => {
                "service_discovery".hash(state);
                protocols.hash(state);
            }
            PrimalCapability::NetworkRouting { protocols } => {
                "network_routing".hash(state);
                protocols.hash(state);
            }
            PrimalCapability::ProxyServices { types } => {
                "proxy_services".hash(state);
                types.hash(state);
            }
            PrimalCapability::VpnServices { protocols } => {
                "vpn_services".hash(state);
                protocols.hash(state);
            }
            PrimalCapability::Custom { name, attributes } => {
                "custom".hash(state);
                name.hash(state);
                attributes.hash(state);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // --- Serde roundtrip tests ---
    #[test]
    fn test_authentication_serde() {
        let cap = PrimalCapability::Authentication {
            methods: vec!["oauth2".to_string(), "jwt".to_string()],
        };
        let json = serde_json::to_string(&cap).unwrap();
        let deserialized: PrimalCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(cap, deserialized);
    }

    #[test]
    fn test_encryption_serde() {
        let cap = PrimalCapability::Encryption {
            algorithms: vec!["AES-256".to_string(), "ChaCha20".to_string()],
        };
        let json = serde_json::to_string(&cap).unwrap();
        let deserialized: PrimalCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(cap, deserialized);
    }

    #[test]
    fn test_key_management_serde() {
        let cap = PrimalCapability::KeyManagement { hsm_support: true };
        let json = serde_json::to_string(&cap).unwrap();
        let deserialized: PrimalCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(cap, deserialized);
    }

    #[test]
    fn test_threat_detection_serde() {
        let cap = PrimalCapability::ThreatDetection { ml_enabled: false };
        let json = serde_json::to_string(&cap).unwrap();
        let deserialized: PrimalCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(cap, deserialized);
    }

    #[test]
    fn test_model_inference_serde() {
        let cap = PrimalCapability::ModelInference {
            models: vec!["gpt-4".to_string(), "claude-3".to_string()],
        };
        let json = serde_json::to_string(&cap).unwrap();
        let deserialized: PrimalCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(cap, deserialized);
    }

    #[test]
    fn test_custom_capability_serde() {
        let cap = PrimalCapability::Custom {
            name: "my_cap".to_string(),
            attributes: "key=val".to_string(),
        };
        let json = serde_json::to_string(&cap).unwrap();
        let deserialized: PrimalCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(cap, deserialized);
    }

    // --- Equality tests ---
    #[test]
    fn test_capability_equality() {
        let a = PrimalCapability::GpuAcceleration { cuda_support: true };
        let b = PrimalCapability::GpuAcceleration { cuda_support: true };
        let c = PrimalCapability::GpuAcceleration {
            cuda_support: false,
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_different_variants_not_equal() {
        let auth = PrimalCapability::Authentication {
            methods: vec!["jwt".to_string()],
        };
        let enc = PrimalCapability::Encryption {
            algorithms: vec!["jwt".to_string()],
        };
        assert_ne!(auth, enc);
    }

    // --- Hash tests ---
    #[test]
    fn test_capability_hash_in_set() {
        let mut set = HashSet::new();
        set.insert(PrimalCapability::Authentication {
            methods: vec!["oauth2".to_string()],
        });
        set.insert(PrimalCapability::Encryption {
            algorithms: vec!["AES-256".to_string()],
        });
        // Duplicate - should not increase set size
        set.insert(PrimalCapability::Authentication {
            methods: vec!["oauth2".to_string()],
        });
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_capability_hash_different_data() {
        let mut set = HashSet::new();
        set.insert(PrimalCapability::KeyManagement { hsm_support: true });
        set.insert(PrimalCapability::KeyManagement { hsm_support: false });
        assert_eq!(set.len(), 2);
    }

    // --- Clone tests ---
    #[test]
    fn test_capability_clone() {
        let cap = PrimalCapability::AgentFramework { mcp_support: true };
        let cloned = cap.clone();
        assert_eq!(cap, cloned);
    }

    // --- All variants test ---
    #[test]
    fn test_all_capability_variants() {
        let caps = vec![
            PrimalCapability::Authentication {
                methods: vec!["jwt".to_string()],
            },
            PrimalCapability::Encryption {
                algorithms: vec!["AES".to_string()],
            },
            PrimalCapability::KeyManagement { hsm_support: true },
            PrimalCapability::ThreatDetection { ml_enabled: true },
            PrimalCapability::AuditLogging {
                compliance: vec!["SOC2".to_string()],
            },
            PrimalCapability::Authorization { rbac_support: true },
            PrimalCapability::FileSystem { supports_zfs: true },
            PrimalCapability::ObjectStorage {
                backends: vec!["s3".to_string()],
            },
            PrimalCapability::DataReplication {
                consistency: "strong".to_string(),
            },
            PrimalCapability::Backup { incremental: true },
            PrimalCapability::DataArchiving {
                compression: vec!["zstd".to_string()],
            },
            PrimalCapability::ContainerRuntime {
                orchestrators: vec!["k8s".to_string()],
            },
            PrimalCapability::ServerlessExecution {
                languages: vec!["rust".to_string()],
            },
            PrimalCapability::GpuAcceleration { cuda_support: true },
            PrimalCapability::LoadBalancing {
                algorithms: vec!["round-robin".to_string()],
            },
            PrimalCapability::AutoScaling {
                metrics: vec!["cpu".to_string()],
            },
            PrimalCapability::ModelInference {
                models: vec!["gpt-4".to_string()],
            },
            PrimalCapability::AgentFramework { mcp_support: true },
            PrimalCapability::MachineLearning {
                training_support: true,
            },
            PrimalCapability::NaturalLanguage {
                languages: vec!["en".to_string()],
            },
            PrimalCapability::ComputerVision {
                models: vec!["yolo".to_string()],
            },
            PrimalCapability::ServiceDiscovery {
                protocols: vec!["mdns".to_string()],
            },
            PrimalCapability::NetworkRouting {
                protocols: vec!["bgp".to_string()],
            },
            PrimalCapability::ProxyServices {
                types: vec!["reverse".to_string()],
            },
            PrimalCapability::VpnServices {
                protocols: vec!["wireguard".to_string()],
            },
            PrimalCapability::Custom {
                name: "test".to_string(),
                attributes: "data".to_string(),
            },
        ];

        // All 26 variants should be present and unique
        assert_eq!(caps.len(), 26);
        let mut set = HashSet::new();
        for cap in &caps {
            set.insert(cap.clone());
        }
        assert_eq!(set.len(), 26);
    }

    // --- Serde all variants test ---
    #[test]
    fn test_all_variants_serde_roundtrip() {
        let caps = vec![
            PrimalCapability::Authorization { rbac_support: true },
            PrimalCapability::FileSystem {
                supports_zfs: false,
            },
            PrimalCapability::ObjectStorage {
                backends: vec!["minio".to_string()],
            },
            PrimalCapability::DataReplication {
                consistency: "eventual".to_string(),
            },
            PrimalCapability::Backup { incremental: false },
            PrimalCapability::DataArchiving {
                compression: vec!["lz4".to_string(), "zstd".to_string()],
            },
            PrimalCapability::ContainerRuntime {
                orchestrators: vec![],
            },
            PrimalCapability::ServerlessExecution {
                languages: vec!["python".to_string(), "rust".to_string()],
            },
            PrimalCapability::LoadBalancing {
                algorithms: vec!["least-connections".to_string()],
            },
            PrimalCapability::AutoScaling {
                metrics: vec!["memory".to_string(), "cpu".to_string()],
            },
            PrimalCapability::MachineLearning {
                training_support: false,
            },
            PrimalCapability::NaturalLanguage {
                languages: vec!["en".to_string(), "fr".to_string()],
            },
            PrimalCapability::ComputerVision {
                models: vec!["resnet".to_string()],
            },
            PrimalCapability::ServiceDiscovery {
                protocols: vec!["consul".to_string()],
            },
            PrimalCapability::NetworkRouting {
                protocols: vec!["ospf".to_string()],
            },
            PrimalCapability::ProxyServices {
                types: vec!["forward".to_string(), "reverse".to_string()],
            },
            PrimalCapability::VpnServices {
                protocols: vec!["ipsec".to_string()],
            },
        ];

        for cap in &caps {
            let json = serde_json::to_string(cap).unwrap();
            let deserialized: PrimalCapability = serde_json::from_str(&json).unwrap();
            assert_eq!(*cap, deserialized, "Failed roundtrip for {:?}", cap);
        }
    }
}
