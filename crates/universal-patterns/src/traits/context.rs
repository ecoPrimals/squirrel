// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Primal context types for user/device-specific routing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

/// Context for user/device-specific primal routing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalContext {
    /// User identifier
    pub user_id: String,
    /// Device identifier
    pub device_id: String,
    /// Session identifier
    pub session_id: String,
    /// Network location (IP, subnet, etc.)
    pub network_location: NetworkLocation,
    /// Security level required
    pub security_level: SecurityLevel,
    /// Additional context metadata
    pub metadata: HashMap<String, String>,
}

/// Network location information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkLocation {
    /// IP address
    pub ip_address: String,
    /// Subnet
    pub subnet: Option<String>,
    /// Local network identifier
    pub network_id: Option<String>,
    /// Geographic location
    pub geo_location: Option<String>,
}

/// Security level requirements
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Basic security
    Basic,
    /// Standard security
    Standard,
    /// High security
    High,
    /// Maximum security
    Maximum,
}

impl Default for PrimalContext {
    fn default() -> Self {
        Self {
            user_id: "default".to_string(),
            device_id: "default".to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            network_location: NetworkLocation {
                ip_address: "127.0.0.1".to_string(),
                subnet: None,
                network_id: None,
                geo_location: None,
            },
            security_level: SecurityLevel::Standard,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_context_default() {
        let ctx = PrimalContext::default();
        assert_eq!(ctx.user_id, "default");
        assert_eq!(ctx.device_id, "default");
        assert!(!ctx.session_id.is_empty());
        assert_eq!(ctx.network_location.ip_address, "127.0.0.1");
        assert!(ctx.network_location.subnet.is_none());
        assert_eq!(ctx.security_level, SecurityLevel::Standard);
        assert!(ctx.metadata.is_empty());
    }

    #[test]
    fn test_primal_context_serde() {
        let ctx = PrimalContext {
            user_id: "user1".to_string(),
            device_id: "device1".to_string(),
            session_id: "session1".to_string(),
            network_location: NetworkLocation {
                ip_address: "10.0.0.1".to_string(),
                subnet: Some("10.0.0.0/24".to_string()),
                network_id: Some("net-1".to_string()),
                geo_location: Some("US-East".to_string()),
            },
            security_level: SecurityLevel::High,
            metadata: {
                let mut m = HashMap::new();
                m.insert("env".to_string(), "production".to_string());
                m
            },
        };
        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: PrimalContext = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.user_id, "user1");
        assert_eq!(deserialized.device_id, "device1");
        assert_eq!(
            deserialized.network_location.subnet,
            Some("10.0.0.0/24".to_string())
        );
        assert_eq!(deserialized.security_level, SecurityLevel::High);
        assert_eq!(deserialized.metadata.get("env").unwrap(), "production");
    }

    #[test]
    fn test_primal_context_equality() {
        let ctx1 = PrimalContext {
            user_id: "u1".to_string(),
            device_id: "d1".to_string(),
            session_id: "s1".to_string(),
            network_location: NetworkLocation {
                ip_address: "127.0.0.1".to_string(),
                subnet: None,
                network_id: None,
                geo_location: None,
            },
            security_level: SecurityLevel::Basic,
            metadata: HashMap::new(),
        };
        let ctx2 = ctx1.clone();
        assert_eq!(ctx1, ctx2);
    }

    #[test]
    fn test_network_location_serde() {
        let loc = NetworkLocation {
            ip_address: "192.168.1.1".to_string(),
            subnet: Some("192.168.1.0/24".to_string()),
            network_id: Some("home-net".to_string()),
            geo_location: Some("EU-West".to_string()),
        };
        let json = serde_json::to_string(&loc).unwrap();
        let deserialized: NetworkLocation = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.ip_address, "192.168.1.1");
        assert_eq!(deserialized.subnet.unwrap(), "192.168.1.0/24");
    }

    #[test]
    fn test_security_level_serde() {
        for level in [
            SecurityLevel::Basic,
            SecurityLevel::Standard,
            SecurityLevel::High,
            SecurityLevel::Maximum,
        ] {
            let json = serde_json::to_string(&level).unwrap();
            let deserialized: SecurityLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, level);
        }
    }

    #[test]
    fn test_security_level_equality_and_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(SecurityLevel::Basic);
        set.insert(SecurityLevel::Standard);
        set.insert(SecurityLevel::High);
        set.insert(SecurityLevel::Maximum);
        assert_eq!(set.len(), 4);
        assert!(set.contains(&SecurityLevel::Basic));
    }
}
