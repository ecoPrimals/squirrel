// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Primal type definitions and capability system.

use serde::{Deserialize, Serialize};

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
    pub fn as_str(&self) -> &'static str {
        match self {
            PrimalType::ToadStool => "toadstool",
            PrimalType::Songbird => "songbird",
            PrimalType::BearDog => "beardog",
            PrimalType::NestGate => "nestgate",
            PrimalType::Squirrel => "squirrel",
            PrimalType::BiomeOS => "biomeos",
            PrimalType::Any => "any",
        }
    }

    /// Get capability for discovery (use when discovering OTHER primals by capability)
    #[must_use]
    pub fn capability(&self) -> &'static str {
        match self {
            PrimalType::ToadStool => "compute",
            PrimalType::Songbird => "service-mesh",
            PrimalType::BearDog => "security",
            PrimalType::NestGate => "storage",
            PrimalType::Squirrel => "squirrel",
            PrimalType::BiomeOS => "ecosystem",
            PrimalType::Any => "any",
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
