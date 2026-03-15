// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive tests for port-related types
//!
//! Coverage goal: 90%+
//! Strategy: Test all enum variants, struct creation, serialization, edge cases

use chrono::{Duration, Utc};

use super::ports::{DynamicPortInfo, PortStatus, PortType};

#[cfg(test)]
mod port_type_tests {
    use super::*;

    #[test]
    fn test_port_type_variants() {
        let types = vec![
            PortType::Http,
            PortType::Https,
            PortType::WebSocket,
            PortType::Grpc,
            PortType::Custom("mqtt".to_string()),
        ];

        assert_eq!(types.len(), 5);
    }

    #[test]
    fn test_port_type_equality() {
        assert_eq!(PortType::Http, PortType::Http);
        assert_ne!(PortType::Http, PortType::Https);
        assert_eq!(
            PortType::Custom("test".to_string()),
            PortType::Custom("test".to_string())
        );
        assert_ne!(
            PortType::Custom("test1".to_string()),
            PortType::Custom("test2".to_string())
        );
    }

    #[test]
    fn test_port_type_clone() {
        let original = PortType::WebSocket;
        let cloned = original.clone();
        assert_eq!(original, cloned);

        let custom = PortType::Custom("custom".to_string());
        let custom_clone = custom.clone();
        assert_eq!(custom, custom_clone);
    }

    #[test]
    fn test_port_type_debug() {
        assert!(format!("{:?}", PortType::Http).contains("Http"));
        assert!(format!("{:?}", PortType::Https).contains("Https"));
        assert!(format!("{:?}", PortType::WebSocket).contains("WebSocket"));
    }

    #[test]
    fn test_port_type_serialization() {
        let port_type = PortType::Grpc;
        let json = serde_json::to_string(&port_type).unwrap();
        let deserialized: PortType = serde_json::from_str(&json).unwrap();
        assert_eq!(port_type, deserialized);

        let custom = PortType::Custom("mqtt".to_string());
        let json_custom = serde_json::to_string(&custom).unwrap();
        let deserialized_custom: PortType = serde_json::from_str(&json_custom).unwrap();
        assert_eq!(custom, deserialized_custom);
    }

    #[test]
    fn test_port_type_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(PortType::Http);
        set.insert(PortType::Https);
        set.insert(PortType::WebSocket);

        assert_eq!(set.len(), 3);
        assert!(set.contains(&PortType::Http));
    }
}

#[cfg(test)]
mod port_status_tests {
    use super::*;

    #[test]
    fn test_port_status_variants() {
        let statuses = vec![
            PortStatus::Active,
            PortStatus::Reserved,
            PortStatus::Releasing,
            PortStatus::Expired,
        ];

        assert_eq!(statuses.len(), 4);
    }

    #[test]
    fn test_port_status_equality() {
        assert_eq!(PortStatus::Active, PortStatus::Active);
        assert_ne!(PortStatus::Active, PortStatus::Reserved);
        assert_ne!(PortStatus::Releasing, PortStatus::Expired);
    }

    #[test]
    fn test_port_status_clone() {
        let status = PortStatus::Active;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_port_status_debug() {
        assert!(format!("{:?}", PortStatus::Active).contains("Active"));
        assert!(format!("{:?}", PortStatus::Reserved).contains("Reserved"));
    }

    #[test]
    fn test_port_status_serialization() {
        let status = PortStatus::Releasing;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: PortStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_port_status_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(PortStatus::Active);
        set.insert(PortStatus::Reserved);

        assert_eq!(set.len(), 2);
        assert!(set.contains(&PortStatus::Active));
    }

    #[test]
    fn test_port_status_lifecycle() {
        // Simulate a typical port lifecycle
        let lifecycle = vec![
            PortStatus::Reserved,
            PortStatus::Active,
            PortStatus::Releasing,
            PortStatus::Expired,
        ];

        assert_eq!(lifecycle.len(), 4);
        assert_eq!(lifecycle[0], PortStatus::Reserved);
        assert_eq!(lifecycle[1], PortStatus::Active);
    }
}

#[cfg(test)]
mod dynamic_port_info_tests {
    use super::*;

    #[test]
    fn test_dynamic_port_info_creation() {
        let now = Utc::now();
        let port_info = DynamicPortInfo {
            assigned_port: 8080,
            port_type: PortType::Http,
            status: PortStatus::Active,
            assigned_at: now,
            lease_duration: Duration::hours(24),
        };

        assert_eq!(port_info.assigned_port, 8080);
        assert_eq!(port_info.port_type, PortType::Http);
        assert_eq!(port_info.status, PortStatus::Active);
    }

    #[test]
    fn test_dynamic_port_info_various_ports() {
        let http_port = DynamicPortInfo {
            assigned_port: 80,
            port_type: PortType::Http,
            status: PortStatus::Active,
            assigned_at: Utc::now(),
            lease_duration: Duration::hours(1),
        };

        let https_port = DynamicPortInfo {
            assigned_port: 443,
            port_type: PortType::Https,
            status: PortStatus::Active,
            assigned_at: Utc::now(),
            lease_duration: Duration::hours(1),
        };

        let ws_port = DynamicPortInfo {
            assigned_port: 8080,
            port_type: PortType::WebSocket,
            status: PortStatus::Reserved,
            assigned_at: Utc::now(),
            lease_duration: Duration::hours(2),
        };

        assert_eq!(http_port.assigned_port, 80);
        assert_eq!(https_port.assigned_port, 443);
        assert_eq!(ws_port.assigned_port, 8080);
    }

    #[test]
    fn test_dynamic_port_info_lease_durations() {
        let short_lease = DynamicPortInfo {
            assigned_port: 9000,
            port_type: PortType::Grpc,
            status: PortStatus::Active,
            assigned_at: Utc::now(),
            lease_duration: Duration::minutes(30),
        };

        let long_lease = DynamicPortInfo {
            assigned_port: 9001,
            port_type: PortType::Grpc,
            status: PortStatus::Active,
            assigned_at: Utc::now(),
            lease_duration: Duration::days(7),
        };

        assert!(short_lease.lease_duration < long_lease.lease_duration);
    }

    #[test]
    fn test_dynamic_port_info_clone() {
        let original = DynamicPortInfo {
            assigned_port: 8080,
            port_type: PortType::Http,
            status: PortStatus::Active,
            assigned_at: Utc::now(),
            lease_duration: Duration::hours(24),
        };

        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_dynamic_port_info_equality() {
        let now = Utc::now();
        let lease = Duration::hours(24);

        let port1 = DynamicPortInfo {
            assigned_port: 8080,
            port_type: PortType::Http,
            status: PortStatus::Active,
            assigned_at: now,
            lease_duration: lease,
        };

        let port2 = DynamicPortInfo {
            assigned_port: 8080,
            port_type: PortType::Http,
            status: PortStatus::Active,
            assigned_at: now,
            lease_duration: lease,
        };

        assert_eq!(port1, port2);
    }

    #[test]
    fn test_dynamic_port_info_debug() {
        let port_info = DynamicPortInfo {
            assigned_port: 8080,
            port_type: PortType::Http,
            status: PortStatus::Active,
            assigned_at: Utc::now(),
            lease_duration: Duration::hours(24),
        };

        let debug_str = format!("{:?}", port_info);
        assert!(debug_str.contains("DynamicPortInfo"));
        assert!(debug_str.contains("8080"));
    }

    #[test]
    fn test_dynamic_port_info_serialization() {
        let port_info = DynamicPortInfo {
            assigned_port: 8080,
            port_type: PortType::Http,
            status: PortStatus::Active,
            assigned_at: Utc::now(),
            lease_duration: Duration::hours(24),
        };

        let json = serde_json::to_string(&port_info).unwrap();
        let deserialized: DynamicPortInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(port_info, deserialized);
    }

    #[test]
    fn test_dynamic_port_info_hash() {
        use std::collections::HashSet;

        let now = Utc::now();
        let port1 = DynamicPortInfo {
            assigned_port: 8080,
            port_type: PortType::Http,
            status: PortStatus::Active,
            assigned_at: now,
            lease_duration: Duration::hours(24),
        };

        let port2 = DynamicPortInfo {
            assigned_port: 8081,
            port_type: PortType::Https,
            status: PortStatus::Reserved,
            assigned_at: now,
            lease_duration: Duration::hours(12),
        };

        let mut set = HashSet::new();
        set.insert(port1.clone());
        set.insert(port2.clone());

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_dynamic_port_info_custom_port_type() {
        let custom_port = DynamicPortInfo {
            assigned_port: 1883,
            port_type: PortType::Custom("mqtt".to_string()),
            status: PortStatus::Active,
            assigned_at: Utc::now(),
            lease_duration: Duration::hours(48),
        };

        assert_eq!(custom_port.assigned_port, 1883); // MQTT default port
        assert_eq!(custom_port.port_type, PortType::Custom("mqtt".to_string()));
    }

    #[test]
    fn test_dynamic_port_info_high_ports() {
        let high_port = DynamicPortInfo {
            assigned_port: 65535, // Max port number
            port_type: PortType::Custom("test".to_string()),
            status: PortStatus::Active,
            assigned_at: Utc::now(),
            lease_duration: Duration::hours(1),
        };

        assert_eq!(high_port.assigned_port, 65535);
    }

    #[test]
    fn test_dynamic_port_info_status_transitions() {
        let mut port_info = DynamicPortInfo {
            assigned_port: 8080,
            port_type: PortType::Http,
            status: PortStatus::Reserved,
            assigned_at: Utc::now(),
            lease_duration: Duration::hours(24),
        };

        // Reserved → Active
        port_info.status = PortStatus::Active;
        assert_eq!(port_info.status, PortStatus::Active);

        // Active → Releasing
        port_info.status = PortStatus::Releasing;
        assert_eq!(port_info.status, PortStatus::Releasing);

        // Releasing → Expired
        port_info.status = PortStatus::Expired;
        assert_eq!(port_info.status, PortStatus::Expired);
    }
}
