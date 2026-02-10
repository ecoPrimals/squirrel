// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Modern Network Configuration Module
//!
//! This module provides a modernized, testable network configuration with:
//! - Builder pattern for easy construction
//! - Type-safe port validation
//! - Sensible defaults
//! - Optional fields for testing
//!
//! # Example
//!
//! ```rust
//! use squirrel_mcp_config::unified::network::{NetworkConfig, Port};
//!
//! // Simple construction with defaults
//! let config = NetworkConfig::default();
//!
//! // Builder pattern
//! let config = NetworkConfig::builder()
//!     .http_port(8080)
//!     .websocket_port(8081)
//!     .enable_tls(true)
//!     .build();
//!
//! // For testing - minimal config
//! let config = NetworkConfig::testing();
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use universal_constants::{limits, network};

/// Validated port number (1-65535)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct Port(u16);

impl Port {
    /// Create a new validated port
    pub fn new(port: u16) -> Result<Self, PortError> {
        if port == 0 {
            return Err(PortError::Zero);
        }
        Ok(Port(port))
    }

    /// Get the port value
    pub fn get(&self) -> u16 {
        self.0
    }

    /// Check if this is a privileged port (<1024)
    pub fn is_privileged(&self) -> bool {
        self.0 < 1024
    }
}

impl TryFrom<u16> for Port {
    type Error = PortError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Port::new(value)
    }
}

impl From<Port> for u16 {
    fn from(port: Port) -> u16 {
        port.0
    }
}

/// Port validation errors
#[derive(Debug, thiserror::Error)]
pub enum PortError {
    #[error("Port number cannot be zero")]
    Zero,
}

/// Modern network configuration with builder pattern
///
/// This replaces the old monolithic NetworkConfig with a more testable,
/// ergonomic design.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Bind address for all services
    #[serde(default = "default_bind_address")]
    bind_address: String,

    /// HTTP API port
    #[serde(default = "default_http_port")]
    http_port: Port,

    /// WebSocket port
    #[serde(default = "default_websocket_port")]
    websocket_port: Port,

    /// gRPC port
    #[serde(default = "default_grpc_port")]
    grpc_port: Port,

    /// Maximum concurrent connections
    #[serde(default = "default_max_connections")]
    max_connections: u32,

    /// Enable TLS/SSL
    #[serde(default)]
    enable_tls: bool,

    /// Path to TLS certificate (if TLS enabled)
    #[serde(default)]
    tls_cert_path: Option<PathBuf>,

    /// Path to TLS private key (if TLS enabled)
    #[serde(default)]
    tls_key_path: Option<PathBuf>,
}

// Default value functions
fn default_bind_address() -> String {
    network::DEFAULT_BIND_ADDRESS.to_string()
}

fn default_http_port() -> Port {
    Port(network::get_service_port("http"))
}

fn default_websocket_port() -> Port {
    Port(network::get_service_port("websocket"))
}

fn default_grpc_port() -> Port {
    Port(network::DEFAULT_GRPC_PORT)
}

fn default_max_connections() -> u32 {
    limits::DEFAULT_MAX_CONNECTIONS as u32
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: default_bind_address(),
            http_port: default_http_port(),
            websocket_port: default_websocket_port(),
            grpc_port: default_grpc_port(),
            max_connections: default_max_connections(),
            enable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

impl NetworkConfig {
    /// Create a new builder for NetworkConfig
    pub fn builder() -> NetworkConfigBuilder {
        NetworkConfigBuilder::default()
    }

    /// Create a minimal config for testing
    ///
    /// Uses high port numbers to avoid conflicts and requires no special privileges.
    pub fn testing() -> Self {
        Self {
            bind_address: network::DEFAULT_BIND_ADDRESS.to_string(),
            http_port: Port(18080), // High ports for testing
            websocket_port: Port(18081),
            grpc_port: Port(19090),
            max_connections: 10, // Low for tests
            enable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }

    /// Create a production-ready config
    pub fn production() -> Self {
        Self {
            bind_address: network::BIND_ALL_INTERFACES.to_string(), // Bind to all interfaces
            http_port: Port(443),                                    // HTTPS
            websocket_port: Port(network::get_service_port("websocket")),
            grpc_port: Port(network::DEFAULT_GRPC_PORT),
            max_connections: 10000, // High for production
            enable_tls: true,
            tls_cert_path: Some(PathBuf::from("/etc/squirrel/cert.pem")),
            tls_key_path: Some(PathBuf::from("/etc/squirrel/key.pem")),
        }
    }

    /// Create a development config
    pub fn development() -> Self {
        Self::default()
    }

    // Getters with clear ownership
    pub fn bind_address(&self) -> &str {
        &self.bind_address
    }

    pub fn http_port(&self) -> Port {
        self.http_port
    }

    pub fn websocket_port(&self) -> Port {
        self.websocket_port
    }

    pub fn grpc_port(&self) -> Port {
        self.grpc_port
    }

    pub fn max_connections(&self) -> u32 {
        self.max_connections
    }

    pub fn is_tls_enabled(&self) -> bool {
        self.enable_tls
    }

    pub fn tls_cert_path(&self) -> Option<&PathBuf> {
        self.tls_cert_path.as_ref()
    }

    pub fn tls_key_path(&self) -> Option<&PathBuf> {
        self.tls_key_path.as_ref()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), NetworkConfigError> {
        // Check TLS consistency
        if self.enable_tls {
            if self.tls_cert_path.is_none() {
                return Err(NetworkConfigError::TlsCertMissing);
            }
            if self.tls_key_path.is_none() {
                return Err(NetworkConfigError::TlsKeyMissing);
            }
        }

        // Check port conflicts
        if self.http_port == self.websocket_port {
            return Err(NetworkConfigError::PortConflict {
                port1: "http_port".to_string(),
                port2: "websocket_port".to_string(),
                value: self.http_port.get(),
            });
        }

        if self.http_port == self.grpc_port {
            return Err(NetworkConfigError::PortConflict {
                port1: "http_port".to_string(),
                port2: "grpc_port".to_string(),
                value: self.http_port.get(),
            });
        }

        if self.websocket_port == self.grpc_port {
            return Err(NetworkConfigError::PortConflict {
                port1: "websocket_port".to_string(),
                port2: "grpc_port".to_string(),
                value: self.websocket_port.get(),
            });
        }

        // Check max connections is reasonable
        if self.max_connections == 0 {
            return Err(NetworkConfigError::InvalidMaxConnections);
        }

        Ok(())
    }
}

/// Builder for NetworkConfig
#[derive(Debug, Default)]
pub struct NetworkConfigBuilder {
    bind_address: Option<String>,
    http_port: Option<Port>,
    websocket_port: Option<Port>,
    grpc_port: Option<Port>,
    max_connections: Option<u32>,
    enable_tls: Option<bool>,
    tls_cert_path: Option<PathBuf>,
    tls_key_path: Option<PathBuf>,
}

impl NetworkConfigBuilder {
    /// Set the bind address
    pub fn bind_address(mut self, addr: impl Into<String>) -> Self {
        self.bind_address = Some(addr.into());
        self
    }

    /// Set the HTTP port
    pub fn http_port(mut self, port: u16) -> Self {
        if let Ok(p) = Port::new(port) {
            self.http_port = Some(p);
        }
        self
    }

    /// Set the WebSocket port
    pub fn websocket_port(mut self, port: u16) -> Self {
        if let Ok(p) = Port::new(port) {
            self.websocket_port = Some(p);
        }
        self
    }

    /// Set the gRPC port
    pub fn grpc_port(mut self, port: u16) -> Self {
        if let Ok(p) = Port::new(port) {
            self.grpc_port = Some(p);
        }
        self
    }

    /// Set the maximum connections
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = Some(max);
        self
    }

    /// Enable or disable TLS
    pub fn enable_tls(mut self, enable: bool) -> Self {
        self.enable_tls = Some(enable);
        self
    }

    /// Set the TLS certificate path
    pub fn tls_cert_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.tls_cert_path = Some(path.into());
        self
    }

    /// Set the TLS key path
    pub fn tls_key_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.tls_key_path = Some(path.into());
        self
    }

    /// Build the NetworkConfig
    pub fn build(self) -> NetworkConfig {
        NetworkConfig {
            bind_address: self.bind_address.unwrap_or_else(default_bind_address),
            http_port: self.http_port.unwrap_or_else(default_http_port),
            websocket_port: self.websocket_port.unwrap_or_else(default_websocket_port),
            grpc_port: self.grpc_port.unwrap_or_else(default_grpc_port),
            max_connections: self.max_connections.unwrap_or_else(default_max_connections),
            enable_tls: self.enable_tls.unwrap_or(false),
            tls_cert_path: self.tls_cert_path,
            tls_key_path: self.tls_key_path,
        }
    }
}

/// Network configuration errors
#[derive(Debug, thiserror::Error)]
pub enum NetworkConfigError {
    #[error("TLS is enabled but certificate path is missing")]
    TlsCertMissing,

    #[error("TLS is enabled but key path is missing")]
    TlsKeyMissing,

    #[error("Port conflict: {port1} and {port2} both use port {value}")]
    PortConflict {
        port1: String,
        port2: String,
        value: u16,
    },

    #[error("max_connections must be greater than 0")]
    InvalidMaxConnections,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_validation() {
        assert!(Port::new(0).is_err());
        assert!(Port::new(1).is_ok());
        assert!(Port::new(8080).is_ok());
        assert!(Port::new(65535).is_ok());
    }

    #[test]
    fn test_port_privileged() {
        let port = Port::new(80).expect("test: should succeed");
        assert!(port.is_privileged());

        let port = Port::new(8080).expect("test: should succeed");
        assert!(!port.is_privileged());
    }

    #[test]
    fn test_default_config() {
        let config = NetworkConfig::default();
        assert_eq!(config.bind_address(), network::DEFAULT_BIND_ADDRESS);
        assert_eq!(config.http_port().get(), network::DEFAULT_HTTP_PORT);
        assert_eq!(config.websocket_port().get(), network::DEFAULT_WEBSOCKET_PORT);
        assert_eq!(config.grpc_port().get(), network::DEFAULT_GRPC_PORT);
        assert_eq!(config.max_connections(), limits::DEFAULT_MAX_CONNECTIONS as u32);
        assert!(!config.is_tls_enabled());
    }

    #[test]
    fn test_testing_config() {
        let config = NetworkConfig::testing();
        assert_eq!(config.bind_address(), network::DEFAULT_BIND_ADDRESS);
        assert_eq!(config.http_port().get(), 18080);
        assert_eq!(config.max_connections(), 10);
        assert!(!config.is_tls_enabled());
    }

    #[test]
    fn test_production_config() {
        let config = NetworkConfig::production();
        assert_eq!(config.bind_address(), network::BIND_ALL_INTERFACES);
        assert_eq!(config.http_port().get(), 443);
        assert!(config.is_tls_enabled());
        assert!(config.tls_cert_path().is_some());
        assert!(config.tls_key_path().is_some());
    }

    #[test]
    fn test_builder() {
        let config = NetworkConfig::builder()
            .bind_address(network::BIND_ALL_INTERFACES)
            .http_port(9000)
            .websocket_port(9001)
            .max_connections(500)
            .build();

        assert_eq!(config.bind_address(), network::BIND_ALL_INTERFACES);
        assert_eq!(config.http_port().get(), 9000);
        assert_eq!(config.websocket_port().get(), 9001);
        assert_eq!(config.max_connections(), 500);
    }

    #[test]
    fn test_builder_with_defaults() {
        let config = NetworkConfig::builder().http_port(9000).build();

        // Should use defaults for unspecified fields
        assert_eq!(config.bind_address(), network::DEFAULT_BIND_ADDRESS);
        assert_eq!(config.http_port().get(), 9000);
        assert_eq!(config.websocket_port().get(), network::DEFAULT_WEBSOCKET_PORT); // default
    }

    #[test]
    fn test_validation_tls_cert_missing() {
        let config = NetworkConfig {
            enable_tls: true,
            tls_cert_path: None,
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            NetworkConfigError::TlsCertMissing
        ));
    }

    #[test]
    fn test_validation_port_conflict() {
        let config = NetworkConfig {
            http_port: Port(8080),
            websocket_port: Port(8080), // Same port!
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            NetworkConfigError::PortConflict {
                port1,
                port2,
                value,
            } => {
                assert_eq!(value, 8080);
                assert!(port1 == "http_port" || port1 == "websocket_port");
                assert!(port2 == "http_port" || port2 == "websocket_port");
            }
            _ => panic!("Expected PortConflict error"),
        }
    }

    #[test]
    fn test_validation_success() {
        let config = NetworkConfig::default();
        assert!(config.validate().is_ok());

        let config = NetworkConfig::testing();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = NetworkConfig::builder()
            .http_port(9000)
            .enable_tls(true)
            .tls_cert_path("/path/to/cert.pem")
            .tls_key_path("/path/to/key.pem")
            .build();

        let toml = toml::to_string(&config).expect("test: should succeed");
        let parsed: NetworkConfig = toml::from_str(&toml).expect("test: should succeed");

        assert_eq!(parsed.http_port().get(), 9000);
        assert!(parsed.is_tls_enabled());
    }
}
