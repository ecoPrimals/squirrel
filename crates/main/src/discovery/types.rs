// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Discovery types and common structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime};

/// Result type for discovery operations
pub type DiscoveryResult<T> = Result<T, DiscoveryError>;

/// Errors that can occur during capability discovery
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    /// The requested capability was not found
    #[error("Capability not found: {capability}")]
    CapabilityNotFound {
        /// The capability that was not found
        capability: String,
    },

    /// Discovery operation timed out
    #[error("Discovery timeout after {timeout:?}")]
    Timeout {
        /// The timeout duration that was exceeded
        timeout: Duration,
    },

    /// A discovery mechanism failed
    #[error("Discovery mechanism failed: {mechanism}: {reason}")]
    MechanismFailed {
        /// The mechanism that failed (e.g., "mdns", "registry")
        mechanism: String,
        /// Human-readable failure reason
        reason: String,
    },

    /// No discovery mechanisms were configured or available
    #[error("No discovery mechanisms available")]
    NoMechanismsAvailable,

    /// Network I/O error during discovery
    #[error("Network error: {0}")]
    NetworkError(#[from] std::io::Error),

    /// Failed to parse discovery response or configuration
    #[error("Parse error: {0}")]
    ParseError(String),

    /// The requested feature is not supported
    #[error("Feature not supported: {0}")]
    NotSupported(String),
}

/// A discovered service provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService {
    /// Service name
    pub name: String,

    /// Service endpoint (e.g., "http://192.168.1.100:9200")
    pub endpoint: String,

    /// Capabilities provided
    pub capabilities: Vec<String>,

    /// Metadata (custom properties)
    pub metadata: HashMap<String, String>,

    /// When this service was discovered
    pub discovered_at: SystemTime,

    /// How this service was discovered
    pub discovery_method: String,

    /// Health status (if known)
    pub healthy: Option<bool>,

    /// Priority/preference score (0-100, higher is better)
    pub priority: u8,
}

impl DiscoveredService {
    /// Check if service is still within cache TTL
    #[must_use]
    pub fn is_fresh(&self, ttl: Duration) -> bool {
        SystemTime::now()
            .duration_since(self.discovered_at)
            .map(|age| age < ttl)
            .unwrap_or(false)
    }

    /// Check if service provides a specific capability
    #[must_use]
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities
            .iter()
            .any(|c| c.eq_ignore_ascii_case(capability))
    }
}

/// Method used to discover a service
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// Environment variable (e.g., SQUIRREL_ENDPOINT)
    EnvironmentVariable,

    /// Multicast DNS (local network)
    Mdns,

    /// DNS Service Discovery
    DnsSd,

    /// Central service registry
    ServiceRegistry,

    /// Peer-to-peer multicast announcement
    P2pMulticast,

    /// Static configuration file
    StaticConfig,

    /// Manual override
    Manual,
}

impl fmt::Display for DiscoveryMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentVariable => write!(f, "Environment Variable"),
            Self::Mdns => write!(f, "mDNS"),
            Self::DnsSd => write!(f, "DNS-SD"),
            Self::ServiceRegistry => write!(f, "Service Registry"),
            Self::P2pMulticast => write!(f, "P2P Multicast"),
            Self::StaticConfig => write!(f, "Static Config"),
            Self::Manual => write!(f, "Manual"),
        }
    }
}

/// A capability discovery request
#[derive(Debug, Clone)]
pub struct CapabilityRequest {
    /// Required capability (e.g., "ai", "storage", "security")
    pub capability: String,

    /// Optional required features
    pub features: Vec<String>,

    /// Preference criteria ("performance", "cost", "local", etc.)
    pub preference: Option<String>,

    /// Discovery timeout
    pub timeout: Duration,

    /// Whether to use cache
    pub use_cache: bool,
}

impl CapabilityRequest {
    /// Create a new capability request
    #[must_use]
    pub fn new(capability: impl Into<String>) -> Self {
        Self {
            capability: capability.into(),
            features: Vec::new(),
            preference: None,
            timeout: Duration::from_secs(5),
            use_cache: true,
        }
    }

    /// Add required features
    #[must_use]
    pub fn with_features(mut self, features: &[impl ToString]) -> Self {
        self.features = features.iter().map(ToString::to_string).collect();
        self
    }

    /// Set preference criteria
    #[must_use]
    pub fn with_preference(mut self, preference: impl Into<String>) -> Self {
        self.preference = Some(preference.into());
        self
    }

    /// Set timeout
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Disable cache
    #[must_use]
    pub const fn no_cache(mut self) -> Self {
        self.use_cache = false;
        self
    }
}

/// Information about a discovered primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Primal name (e.g., "Songbird", "BearDog")
    pub name: String,

    /// Host address
    pub host: String,

    /// Port number
    pub port: u16,

    /// Capabilities provided by this primal
    pub capabilities: Vec<String>,

    /// How this primal was discovered
    pub discovery_method: String,
}

/// Identity of this primal (self-knowledge)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalIdentity {
    /// This primal's name
    pub name: String,

    /// This primal's capabilities
    pub capabilities: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_request_builder() {
        let request = CapabilityRequest::new("ai")
            .with_features(&["text-generation", "embeddings"])
            .with_preference("performance")
            .with_timeout(Duration::from_secs(10));

        assert_eq!(request.capability, "ai");
        assert_eq!(request.features.len(), 2);
        assert_eq!(request.preference, Some("performance".to_string()));
        assert_eq!(request.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_discovered_service_has_capability() {
        let service = DiscoveredService {
            name: "test-service".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec!["ai".to_string(), "storage".to_string()],
            metadata: HashMap::new(),
            discovered_at: SystemTime::now(),
            discovery_method: "environment_variable".to_string(),
            healthy: Some(true),
            priority: 50,
        };

        assert!(service.has_capability("ai"));
        assert!(service.has_capability("AI")); // Case insensitive
        assert!(!service.has_capability("compute"));
    }

    #[test]
    fn test_discovered_service_freshness() {
        let mut service = DiscoveredService {
            name: "test".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec![],
            metadata: HashMap::new(),
            discovered_at: SystemTime::now(),
            discovery_method: "mdns".to_string(),
            healthy: Some(true),
            priority: 50,
        };

        // Fresh service
        assert!(service.is_fresh(Duration::from_secs(60)));

        // Old service
        service.discovered_at = SystemTime::now() - Duration::from_secs(600);
        assert!(!service.is_fresh(Duration::from_secs(60)));
    }
}
