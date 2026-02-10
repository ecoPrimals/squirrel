// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Tests for network configuration types

use super::*;

#[test]
fn test_port_new_valid() {
    let port = Port::new(8080).expect("Failed to create port");
    assert_eq!(port.get(), 8080);
}

#[test]
fn test_port_new_zero() {
    let result = Port::new(0);
    assert!(result.is_err());
}

#[test]
fn test_port_get() {
    let port = Port::new(3000).expect("test: should succeed");
    assert_eq!(port.get(), 3000);
}

#[test]
fn test_port_is_privileged() {
    let port80 = Port::new(80).expect("test: should succeed");
    assert!(port80.is_privileged());
    
    let port8080 = Port::new(8080).expect("test: should succeed");
    assert!(!port8080.is_privileged());
}

#[test]
fn test_port_try_from_u16() {
    let port: Port = 9000.try_into().expect("Failed");
    assert_eq!(port.get(), 9000);
}

#[test]
fn test_port_try_from_zero() {
    let result: Result<Port, _> = 0.try_into();
    assert!(result.is_err());
}

#[test]
fn test_port_into_u16() {
    let port = Port::new(5000).expect("test: should succeed");
    let value: u16 = port.into();
    assert_eq!(value, 5000);
}

#[test]
fn test_port_clone() {
    let port = Port::new(7000).expect("test: should succeed");
    let cloned = port.clone();
    assert_eq!(cloned.get(), 7000);
}

#[test]
fn test_port_partial_eq() {
    let port1 = Port::new(8080).expect("test: should succeed");
    let port2 = Port::new(8080).expect("test: should succeed");
    let port3 = Port::new(8081).expect("test: should succeed");
    
    assert_eq!(port1, port2);
    assert_ne!(port1, port3);
}

#[test]
fn test_port_debug() {
    let port = Port::new(3000).expect("test: should succeed");
    let debug_str = format!("{:?}", port);
    assert!(debug_str.contains("Port"));
}

#[test]
fn test_port_serialization() {
    let port = Port::new(8080).expect("test: should succeed");
    let serialized = serde_json::to_string(&port).expect("Failed to serialize");
    assert_eq!(serialized, "8080");
}

#[test]
fn test_port_deserialization() {
    let json = "8080";
    let port: Port = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(port.get(), 8080);
}

#[test]
fn test_port_error_zero() {
    let err = PortError::Zero;
    let msg = format!("{}", err);
    assert!(msg.contains("zero"));
}

#[test]
fn test_port_boundary_values() {
    assert!(Port::new(1).is_ok());
    assert!(Port::new(65535).is_ok());
    assert!(Port::new(0).is_err());
}

#[test]
fn test_port_privileged_boundary() {
    let port1023 = Port::new(1023).expect("test: should succeed");
    assert!(port1023.is_privileged());
    
    let port1024 = Port::new(1024).expect("test: should succeed");
    assert!(!port1024.is_privileged());
}

#[test]
fn test_network_config_default() {
    let config = NetworkConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("NetworkConfig"));
}

#[test]
fn test_network_config_clone() {
    let config = NetworkConfig::default();
    let cloned = config.clone();
    let _ = format!("{:?}", cloned);
}

#[test]
fn test_network_config_serialization() {
    let config = NetworkConfig::default();
    let serialized = serde_json::to_string(&config).expect("Failed to serialize");
    let _deserialized: NetworkConfig = serde_json::from_str(&serialized).expect("Failed to deserialize");
}

#[test]
fn test_port_common_values() {
    let common_ports = vec![80, 443, 8080, 8443, 3000, 5000, 9000];
    for port_num in common_ports {
        let port = Port::new(port_num).expect(&format!("Failed to create port {}", port_num));
        assert_eq!(port.get(), port_num);
    }
}

#[test]
fn test_port_eq_trait() {
    let port1 = Port::new(8080).expect("test: should succeed");
    let port2 = Port::new(8080).expect("test: should succeed");
    assert!(port1 == port2);
}

