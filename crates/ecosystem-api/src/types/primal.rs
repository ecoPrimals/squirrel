// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Primal type definitions and capability system.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use universal_constants::primal_names;

/// Runtime capability identifier — accepts any string-based capability domain.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CapabilityIdentifier(Arc<str>);

impl CapabilityIdentifier {
    /// Create from a string-based capability domain (e.g. `"security"`, `"storage"`).
    #[must_use]
    pub fn new(capability: impl AsRef<str>) -> Self {
        Self(Arc::from(capability.as_ref()))
    }

    /// Create from a [`CapabilityDomain`].
    #[must_use]
    pub fn from_domain(domain: &CapabilityDomain) -> Self {
        Self(Arc::from(domain.as_str()))
    }

    /// View as a [`CapabilityDomain`] for discovery routing.
    #[must_use]
    pub fn as_domain(&self) -> CapabilityDomain {
        CapabilityDomain::new(self.as_str())
    }

    /// Get the capability domain string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Prefix for `{PREFIX}_ENDPOINT`-style configuration.
    #[must_use]
    pub fn endpoint_env_prefix(&self) -> String {
        self.as_str().replace('-', "_").to_uppercase()
    }
}

/// Capability domain for ecosystem routing — what is needed, not who provides it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CapabilityDomain(Arc<str>);

impl CapabilityDomain {
    /// Create a capability domain from a string (e.g. `"security"`, `"storage"`).
    #[must_use]
    pub fn new(domain: impl AsRef<str>) -> Self {
        Self(Arc::from(domain.as_ref()))
    }

    /// Get the domain string used for discovery routing.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Prefix for `{PREFIX}_ENDPOINT`-style configuration.
    #[must_use]
    pub fn endpoint_env_prefix(&self) -> String {
        self.as_str().replace('-', "_").to_uppercase()
    }
}

impl From<&str> for CapabilityDomain {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for CapabilityDomain {
    fn from(s: String) -> Self {
        Self(Arc::from(s))
    }
}

impl From<&str> for CapabilityIdentifier {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for CapabilityIdentifier {
    fn from(s: String) -> Self {
        Self(Arc::from(s))
    }
}

impl From<CapabilityDomain> for CapabilityIdentifier {
    fn from(domain: CapabilityDomain) -> Self {
        Self::from_domain(&domain)
    }
}

impl std::fmt::Display for CapabilityDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for CapabilityIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Standardized primal types
#[deprecated(
    since = "0.2.0",
    note = "Use CapabilityDomain for capability-based discovery instead of hardcoded primal types"
)]
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

#[expect(deprecated, reason = "impl block for deprecated enum (serde backward compat)")]
impl PrimalType {
    /// Get string representation (for serialization/backward compatibility)
    #[must_use]
    #[deprecated(since = "0.2.0", note = "Use CapabilityDomain for discovery")]
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

    /// Get capability domain for discovery (use when discovering primals by capability)
    #[must_use]
    pub const fn capability(&self) -> &'static str {
        match self {
            Self::ToadStool => "compute",
            Self::Songbird => "service-mesh",
            Self::BearDog => "security",
            Self::NestGate => "storage",
            Self::Squirrel => "inference",
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
    /// Capability domain (e.g. `"security"`, `"compute"`, `"any"`)
    pub primal_type: String,
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

    #[expect(deprecated, reason = "tests deprecated enum backward compat")]
    #[test]
    fn capability_domain_is_primary_routing_key() {
        let cases = [
            (PrimalType::ToadStool, "compute"),
            (PrimalType::Songbird, "service-mesh"),
            (PrimalType::BearDog, "security"),
            (PrimalType::NestGate, "storage"),
            (PrimalType::Squirrel, "inference"),
            (PrimalType::BiomeOS, "ecosystem"),
            (PrimalType::Any, "any"),
        ];
        for (t, domain) in cases {
            assert_eq!(t.capability(), domain);
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
    fn capability_identifier_and_domain_strings() {
        let id = CapabilityIdentifier::new("service-mesh");
        assert_eq!(id.as_str(), "service-mesh");
        assert_eq!(id.endpoint_env_prefix(), "SERVICE_MESH");
        let domain: CapabilityDomain = "storage".into();
        assert_eq!(
            CapabilityIdentifier::from_domain(&domain).as_str(),
            "storage"
        );
        let id_from_domain: CapabilityIdentifier = domain.into();
        assert_eq!(id_from_domain.as_str(), "storage");
    }

    #[test]
    fn primal_dependency_serde() {
        let d = PrimalDependency {
            primal_type: "any".to_string(),
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
