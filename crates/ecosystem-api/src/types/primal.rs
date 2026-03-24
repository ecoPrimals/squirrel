// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Primal type definitions and capability system.

use serde::{Deserialize, Serialize};
use universal_constants::primal_names;

/// Standardized primal types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalType {
    /// `ToadStool` compute platform
    ToadStool,
    /// Songbird service mesh
    Songbird,
    /// `BearDog` security framework
    BearDog,
    /// `NestGate` storage system
    NestGate,
    /// Squirrel AI platform
    Squirrel,
    /// biomeOS orchestration platform
    BiomeOS,
    /// Any primal that provides the required capabilities (for capability-based discovery)
    Any,
}

impl PrimalType {
    /// Get string representation (for serialization/backward compatibility)
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ToadStool => primal_names::TOADSTOOL,
            Self::Songbird => primal_names::SONGBIRD,
            Self::BearDog => primal_names::BEARDOG,
            Self::NestGate => primal_names::NESTGATE,
            Self::Squirrel => primal_names::SQUIRREL,
            Self::BiomeOS => primal_names::BIOMEOS,
            Self::Any => "any",
        }
    }

    /// Get capability for discovery (use when discovering OTHER primals by capability)
    #[must_use]
    pub const fn capability(&self) -> &'static str {
        match self {
            Self::ToadStool => "compute",
            Self::Songbird => "service-mesh",
            Self::BearDog => "security",
            Self::NestGate => "storage",
            Self::Squirrel => "squirrel",
            Self::BiomeOS => "ecosystem",
            Self::Any => "any",
        }
    }
}

/// Standardized capability system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalCapability {
    /// Container runtime with supported orchestrators (`ToadStool`)
    ContainerRuntime {
        /// Supported orchestrator names
        orchestrators: Vec<String>,
    },
    /// Serverless execution with supported languages (`ToadStool`)
    ServerlessExecution {
        /// Supported language identifiers
        languages: Vec<String>,
    },
    /// GPU acceleration with optional CUDA support (`ToadStool`)
    GpuAcceleration {
        /// Whether CUDA is supported
        cuda_support: bool,
    },
    /// Native execution with supported architectures (`ToadStool`)
    NativeExecution {
        /// Supported CPU architectures
        architectures: Vec<String>,
    },
    /// WebAssembly execution with WASI support (`ToadStool`)
    WasmExecution {
        /// Whether WASI is supported
        wasi_support: bool,
    },

    /// Authentication methods (`BearDog`)
    Authentication {
        /// Supported auth method identifiers
        methods: Vec<String>,
    },
    /// Encryption algorithms (`BearDog`)
    Encryption {
        /// Supported algorithm names
        algorithms: Vec<String>,
    },
    /// Key management with optional HSM support (`BearDog`)
    KeyManagement {
        /// Whether HSM is supported
        hsm_support: bool,
    },
    /// Threat detection with optional ML (`BearDog`)
    ThreatDetection {
        /// Whether ML-based detection is enabled
        ml_enabled: bool,
    },
    /// Compliance frameworks (`BearDog`)
    Compliance {
        /// Supported compliance framework names
        frameworks: Vec<String>,
    },

    /// File system with optional ZFS support (`NestGate`)
    FileSystem {
        /// Whether ZFS is supported
        supports_zfs: bool,
    },
    /// Object storage backends (`NestGate`)
    ObjectStorage {
        /// Backend storage identifiers
        backends: Vec<String>,
    },
    /// Data replication consistency model (`NestGate`)
    DataReplication {
        /// Consistency model name
        consistency: String,
    },
    /// Volume management protocols (`NestGate`)
    VolumeManagement {
        /// Supported protocol names
        protocols: Vec<String>,
    },
    /// Backup/restore with optional incremental (`NestGate`)
    BackupRestore {
        /// Whether incremental backup is supported
        incremental: bool,
    },

    /// Service discovery protocols (`Songbird`)
    ServiceDiscovery {
        /// Supported protocol names
        protocols: Vec<String>,
    },
    /// Network routing protocols (`Songbird`)
    NetworkRouting {
        /// Supported protocol names
        protocols: Vec<String>,
    },
    /// Load balancing algorithms (`Songbird`)
    LoadBalancing {
        /// Algorithm identifiers
        algorithms: Vec<String>,
    },
    /// Circuit breaking enabled (`Songbird`)
    CircuitBreaking {
        /// Whether circuit breaking is enabled
        enabled: bool,
    },

    /// Model inference with supported models (`Squirrel`)
    ModelInference {
        /// Supported model identifiers
        models: Vec<String>,
    },
    /// Agent framework with MCP support (`Squirrel`)
    AgentFramework {
        /// Whether MCP protocol is supported
        mcp_support: bool,
    },
    /// Machine learning with training support (`Squirrel`)
    MachineLearning {
        /// Whether training (vs inference only) is supported
        training_support: bool,
    },
    /// Natural language support (`Squirrel`)
    NaturalLanguage {
        /// Supported language codes
        languages: Vec<String>,
    },

    /// Orchestration of primals (`biomeOS`)
    Orchestration {
        /// Primal type identifiers
        primals: Vec<String>,
    },
    /// Manifest formats supported (`biomeOS`)
    Manifests {
        /// Format identifiers (e.g. JSON, YAML)
        formats: Vec<String>,
    },
    /// Deployment strategies (`biomeOS`)
    Deployment {
        /// Strategy names
        strategies: Vec<String>,
    },
    /// Monitoring metrics (`biomeOS`)
    Monitoring {
        /// Metric identifiers
        metrics: Vec<String>,
    },
}

/// Dependency on another primal's capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalDependency {
    /// Type of primal (or Any for capability-based discovery)
    pub primal_type: PrimalType,
    /// Human-readable name for the dependency
    pub name: String,
    /// Required capabilities (used when `primal_type` is Any)
    pub capabilities: Vec<String>,
    /// Whether this dependency is required for operation
    pub required: bool,
    /// Minimum version requirement
    pub min_version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primal_type_as_str_and_capability() {
        let cases = [
            (PrimalType::ToadStool, "toadstool", "compute"),
            (PrimalType::Songbird, "songbird", "service-mesh"),
            (PrimalType::BearDog, "beardog", "security"),
            (PrimalType::NestGate, "nestgate", "storage"),
            (PrimalType::Squirrel, "squirrel", "squirrel"),
            (PrimalType::BiomeOS, "biomeos", "ecosystem"),
            (PrimalType::Any, "any", "any"),
        ];
        for (t, s, cap) in cases {
            assert_eq!(t.as_str(), s);
            assert_eq!(t.capability(), cap);
        }
    }

    #[test]
    fn primal_capability_serde_roundtrip_samples() {
        let samples = vec![
            PrimalCapability::ContainerRuntime {
                orchestrators: vec!["k8s".into()],
            },
            PrimalCapability::ServerlessExecution {
                languages: vec!["rust".into()],
            },
            PrimalCapability::GpuAcceleration { cuda_support: true },
            PrimalCapability::NativeExecution {
                architectures: vec!["aarch64".into()],
            },
            PrimalCapability::WasmExecution {
                wasi_support: false,
            },
            PrimalCapability::Authentication {
                methods: vec!["jwt".into()],
            },
            PrimalCapability::Encryption {
                algorithms: vec!["aes".into()],
            },
            PrimalCapability::KeyManagement { hsm_support: false },
            PrimalCapability::ThreatDetection { ml_enabled: true },
            PrimalCapability::Compliance {
                frameworks: vec!["soc2".into()],
            },
            PrimalCapability::FileSystem { supports_zfs: true },
            PrimalCapability::ObjectStorage {
                backends: vec!["s3".into()],
            },
            PrimalCapability::DataReplication {
                consistency: "strong".into(),
            },
            PrimalCapability::VolumeManagement {
                protocols: vec!["nfs".into()],
            },
            PrimalCapability::BackupRestore { incremental: false },
            PrimalCapability::ServiceDiscovery {
                protocols: vec!["mdns".into()],
            },
            PrimalCapability::NetworkRouting {
                protocols: vec!["tcp".into()],
            },
            PrimalCapability::LoadBalancing {
                algorithms: vec!["rr".into()],
            },
            PrimalCapability::CircuitBreaking { enabled: true },
            PrimalCapability::ModelInference {
                models: vec!["m".into()],
            },
            PrimalCapability::AgentFramework { mcp_support: true },
            PrimalCapability::MachineLearning {
                training_support: false,
            },
            PrimalCapability::NaturalLanguage {
                languages: vec!["en".into()],
            },
            PrimalCapability::Orchestration {
                primals: vec!["p".into()],
            },
            PrimalCapability::Manifests {
                formats: vec!["yaml".into()],
            },
            PrimalCapability::Deployment {
                strategies: vec!["bluegreen".into()],
            },
            PrimalCapability::Monitoring {
                metrics: vec!["cpu".into()],
            },
        ];
        for cap in samples {
            let json = serde_json::to_string(&cap).expect("should succeed");
            let back: PrimalCapability = serde_json::from_str(&json).expect("should succeed");
            assert_eq!(back, cap);
        }
    }

    #[test]
    fn primal_dependency_serde() {
        let d = PrimalDependency {
            primal_type: PrimalType::Any,
            name: "dep".into(),
            capabilities: vec!["a".into()],
            required: true,
            min_version: Some("1.0.0".into()),
        };
        let json = serde_json::to_string(&d).expect("should succeed");
        let back: PrimalDependency = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(back.name, "dep");
        assert!(back.required);
    }
}
