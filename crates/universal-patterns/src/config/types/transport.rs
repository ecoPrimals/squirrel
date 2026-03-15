// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Transport and network configuration types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Network configuration for primal communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Primary listening address
    pub bind_address: String,

    /// Primary listening port
    pub port: u16,

    /// External/public address (for service discovery)
    pub public_address: Option<String>,

    /// TLS configuration
    pub tls: Option<TlsConfig>,

    /// Timeout settings
    pub timeouts: TimeoutConfig,

    /// Connection limits
    pub limits: ConnectionLimits,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Certificate file path
    pub cert_file: PathBuf,

    /// Private key file path
    pub key_file: PathBuf,

    /// CA certificate file path (for mutual TLS)
    pub ca_file: Option<PathBuf>,

    /// Require client certificates
    pub require_client_cert: bool,
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Connection timeout (seconds)
    pub connect: u64,

    /// Request timeout (seconds)
    pub request: u64,

    /// Keep-alive timeout (seconds)
    pub keep_alive: u64,

    /// Idle timeout (seconds)
    pub idle: u64,
}

/// Connection limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionLimits {
    /// Maximum concurrent connections
    pub max_connections: usize,

    /// Maximum requests per connection
    pub max_requests_per_connection: usize,

    /// Rate limiting (requests per second)
    pub rate_limit: Option<f64>,
}
