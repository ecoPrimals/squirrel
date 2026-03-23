// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Context types for primal operations
//!
//! This module defines context structures that carry information about
//! the execution environment, user, device, and security requirements.

use serde::{Deserialize, Serialize};

/// Context for primal operations with user/device awareness
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PrimalContext {
    /// User identifier
    pub user_id: String,
    /// Device identifier
    pub device_id: String,
    /// Optional session identifier
    pub session_id: Option<String>,
    /// Optional biome identifier
    pub biome_id: Option<String>,
    /// Network location for routing
    pub network_location: NetworkLocation,
    /// Required security level
    pub security_level: SecurityLevel,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Network location information for context-aware routing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkLocation {
    /// Geographic or logical region
    pub region: String,
    /// Data center identifier
    pub data_center: Option<String>,
    /// Availability zone within the region
    pub availability_zone: Option<String>,
    /// Client IP address
    pub ip_address: Option<String>,
    /// Subnet identifier
    pub subnet: Option<String>,
    /// Network identifier
    pub network_id: Option<String>,
    /// Geographic coordinates or location string
    pub geo_location: Option<String>,
}

impl Default for NetworkLocation {
    fn default() -> Self {
        Self {
            region: std::env::var("DEPLOYMENT_REGION").unwrap_or_else(|_| "us-west-2".to_string()),
            data_center: std::env::var("DATA_CENTER").ok(),
            availability_zone: std::env::var("AVAILABILITY_ZONE").ok(),
            ip_address: None,
            subnet: None,
            network_id: None,
            geo_location: None,
        }
    }
}

/// Security level requirements for operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SecurityLevel {
    /// Basic security - standard operations
    #[default]
    Basic,
    /// Standard security - normal operations
    Standard,
    /// Public security - public-facing operations
    Public,
    /// Enhanced security - sensitive data
    Enhanced,
    /// Advanced security - advanced operations
    Advanced,
    /// High security - high-value operations
    High,
    /// Critical security - critical operations
    Critical,
    /// Administrative security - admin operations
    Administrative,
    /// Internal security - internal system operations
    Internal,
    /// Maximum security - highly sensitive operations
    Maximum,
    /// Custom security level
    Custom(String),
}

impl std::fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic => write!(f, "Basic"),
            Self::Standard => write!(f, "Standard"),
            Self::Public => write!(f, "Public"),
            Self::Enhanced => write!(f, "Enhanced"),
            Self::Advanced => write!(f, "Advanced"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
            Self::Administrative => write!(f, "Administrative"),
            Self::Internal => write!(f, "Internal"),
            Self::Maximum => write!(f, "Maximum"),
            Self::Custom(level) => write!(f, "Custom({level})"),
        }
    }
}

/// Universal security context for authentication and authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalSecurityContext {
    /// User identifier
    pub user_id: String,
    /// Optional session identifier
    pub session_id: Option<String>,
    /// Security level for the context
    pub security_level: SecurityLevel,
    /// List of granted permissions
    pub permissions: Vec<String>,
}

impl Default for UniversalSecurityContext {
    fn default() -> Self {
        Self {
            user_id: "anonymous".to_string(),
            session_id: None,
            security_level: SecurityLevel::Basic,
            permissions: vec![],
        }
    }
}

/// Create a default primal context
#[must_use]
pub fn create_default_context(user_id: &str, device_id: &str) -> PrimalContext {
    PrimalContext {
        user_id: user_id.to_string(),
        device_id: device_id.to_string(),
        session_id: None,
        biome_id: None,
        network_location: NetworkLocation::default(),
        security_level: SecurityLevel::Basic,
        metadata: std::collections::HashMap::new(),
    }
}

/// Create a default security context
#[must_use]
pub fn create_default_security_context(user_id: &str) -> UniversalSecurityContext {
    UniversalSecurityContext {
        user_id: user_id.to_string(),
        session_id: None,
        security_level: SecurityLevel::Basic,
        permissions: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn security_level_display_covers_all_variants() {
        let levels = [
            SecurityLevel::Basic,
            SecurityLevel::Standard,
            SecurityLevel::Public,
            SecurityLevel::Enhanced,
            SecurityLevel::Advanced,
            SecurityLevel::High,
            SecurityLevel::Critical,
            SecurityLevel::Administrative,
            SecurityLevel::Internal,
            SecurityLevel::Maximum,
            SecurityLevel::Custom("x".into()),
        ];
        for level in levels {
            let s = level.to_string();
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn create_default_context_and_security() {
        let c = create_default_context("u", "d");
        assert_eq!(c.user_id, "u");
        assert_eq!(c.device_id, "d");
        let s = create_default_security_context("alice");
        assert_eq!(s.user_id, "alice");
    }

    #[test]
    fn primal_context_and_network_location_serde() {
        let mut c = create_default_context("a", "b");
        c.network_location = NetworkLocation {
            region: "r".into(),
            data_center: Some("dc".into()),
            availability_zone: None,
            ip_address: None,
            subnet: None,
            network_id: None,
            geo_location: None,
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: PrimalContext = serde_json::from_str(&json).unwrap();
        assert_eq!(back.network_location.region, "r");
    }

    #[test]
    fn universal_security_context_default() {
        let d = UniversalSecurityContext::default();
        assert_eq!(d.user_id, "anonymous");
        assert!(d.permissions.is_empty());
    }
}
