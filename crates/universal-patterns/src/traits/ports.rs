// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Dynamic port types.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Dynamic port information for songbird-managed ports
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DynamicPortInfo {
    /// Port assigned by songbird
    pub assigned_port: u16,
    /// Port type (HTTP, HTTPS, WebSocket, etc.)
    pub port_type: PortType,
    /// Port status
    pub status: PortStatus,
    /// Port assignment timestamp
    pub assigned_at: DateTime<Utc>,
    /// Port lease duration
    pub lease_duration: Duration,
}

/// Port type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortType {
    /// HTTP port
    Http,
    /// HTTPS port
    Https,
    /// WebSocket port
    WebSocket,
    /// gRPC port
    Grpc,
    /// Custom port type
    Custom(String),
}

/// Port status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortStatus {
    /// Port is active and available
    Active,
    /// Port is reserved but not yet active
    Reserved,
    /// Port is being releasing
    Releasing,
    /// Port is expired and should be cleaned up
    Expired,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_port_info_serde() {
        let info = DynamicPortInfo {
            assigned_port: 8080,
            port_type: PortType::Http,
            status: PortStatus::Active,
            assigned_at: Utc::now(),
            lease_duration: Duration::hours(1),
        };
        let json = serde_json::to_string(&info).expect("should succeed");
        let deserialized: DynamicPortInfo = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.assigned_port, 8080);
        assert_eq!(deserialized.port_type, PortType::Http);
        assert_eq!(deserialized.status, PortStatus::Active);
    }

    #[test]
    fn test_port_type_serde() {
        for pt in [
            PortType::Http,
            PortType::Https,
            PortType::WebSocket,
            PortType::Grpc,
        ] {
            let json = serde_json::to_string(&pt).expect("should succeed");
            let deserialized: PortType = serde_json::from_str(&json).expect("should succeed");
            assert_eq!(deserialized, pt);
        }
    }

    #[test]
    fn test_port_type_custom_serde() {
        let pt = PortType::Custom("mqtt".to_string());
        let json = serde_json::to_string(&pt).expect("should succeed");
        let deserialized: PortType = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized, pt);
    }

    #[test]
    fn test_port_status_serde() {
        for status in [
            PortStatus::Active,
            PortStatus::Reserved,
            PortStatus::Releasing,
            PortStatus::Expired,
        ] {
            let json = serde_json::to_string(&status).expect("should succeed");
            let deserialized: PortStatus = serde_json::from_str(&json).expect("should succeed");
            assert_eq!(deserialized, status);
        }
    }

    #[test]
    fn test_port_type_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(PortType::Http);
        set.insert(PortType::Https);
        set.insert(PortType::WebSocket);
        set.insert(PortType::Grpc);
        set.insert(PortType::Custom("quic".to_string()));
        assert_eq!(set.len(), 5);
    }

    #[test]
    fn test_dynamic_port_info_equality() {
        let now = Utc::now();
        let info1 = DynamicPortInfo {
            assigned_port: 9090,
            port_type: PortType::Grpc,
            status: PortStatus::Reserved,
            assigned_at: now,
            lease_duration: Duration::minutes(30),
        };
        let info2 = info1.clone();
        assert_eq!(info1, info2);
    }
}
