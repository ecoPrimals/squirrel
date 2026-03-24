// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration helpers for modern, env-driven configuration
//!
//! This module provides helpers to replace hardcoded values with environment-driven config.

use std::env;

/// Get port from environment variable with fallback to default.
///
/// # Examples
///
/// ```
/// use universal_constants::config_helpers::get_port;
///
/// // Will use PORT env var if set, otherwise 8080
/// let port = get_port("PORT", 8080);
/// assert!(port > 0);
/// ```
#[must_use]
pub fn get_port(env_var: &str, default: u16) -> u16 {
    env::var(env_var)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// Get host from environment variable with fallback to default.
///
/// # Examples
///
/// ```
/// use universal_constants::config_helpers::get_host;
///
/// let host = get_host("HOST", "localhost");
/// assert!(!host.is_empty());
/// ```
#[must_use]
pub fn get_host(env_var: &str, default: &str) -> String {
    env::var(env_var).unwrap_or_else(|_| default.to_string())
}

/// Read a configuration value from environment, using a custom reader function.
///
/// Pattern absorbed from rhizoCrypt v0.13: `from_env_reader(F)` enables
/// tests to inject deterministic environment values without mutating
/// process state.
///
/// # Example
///
/// ```rust
/// use universal_constants::config_helpers::from_env_reader;
///
/// // Production: reads real env
/// let val = from_env_reader("MY_KEY", "default", |k| std::env::var(k));
///
/// // Test: inject a fake reader
/// let val = from_env_reader("MY_KEY", "default", |_| Ok("test-value".to_string()));
/// assert_eq!(val, "test-value");
/// ```
pub fn from_env_reader<F>(key: &str, default: &str, reader: F) -> String
where
    F: FnOnce(&str) -> Result<String, std::env::VarError>,
{
    reader(key).unwrap_or_else(|_| default.to_string())
}

/// Build URL from environment-driven host and port.
///
/// # Examples
///
/// ```
/// use universal_constants::config_helpers::build_url;
///
/// let url = build_url("http", "localhost", 8080, "/api");
/// assert_eq!(url, "http://localhost:8080/api");
/// ```
#[must_use]
pub fn build_url(scheme: &str, host: &str, port: u16, path: &str) -> String {
    format!("{scheme}://{host}:{port}{path}")
}

/// Get timeout duration from environment variable in seconds.
///
/// # Examples
///
/// ```
/// use universal_constants::config_helpers::get_timeout_secs;
/// use std::time::Duration;
///
/// let timeout = get_timeout_secs("REQUEST_TIMEOUT", 30);
/// assert_eq!(timeout, Duration::from_secs(30));
/// ```
#[must_use]
pub fn get_timeout_secs(env_var: &str, default_secs: u64) -> std::time::Duration {
    let secs = env::var(env_var)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default_secs);
    std::time::Duration::from_secs(secs)
}

/// Configuration builder for services
///
/// # Examples
///
/// ```
/// use universal_constants::config_helpers::ServiceConfig;
///
/// let config = ServiceConfig::builder()
///     .with_env_prefix("MYSERVICE")
///     .with_default_port(8080)
///     .with_default_host("localhost")
///     .build();
///
/// assert_eq!(config.port(), 8080);
/// assert_eq!(config.host(), "localhost");
/// ```
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    host: String,
    port: u16,
    timeout_secs: u64,
}

impl ServiceConfig {
    /// Create a new service config builder.
    #[must_use]
    pub fn builder() -> ServiceConfigBuilder {
        ServiceConfigBuilder::default()
    }

    /// Get the host.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Get the port
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Get the timeout duration
    #[must_use]
    pub const fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_secs)
    }

    /// Build HTTP URL.
    #[must_use]
    pub fn http_url(&self, path: &str) -> String {
        build_url("http", &self.host, self.port, path)
    }

    /// Build HTTPS URL.
    #[must_use]
    pub fn https_url(&self, path: &str) -> String {
        build_url("https", &self.host, self.port, path)
    }
}

/// Builder for `ServiceConfig`
#[derive(Debug, Default)]
pub struct ServiceConfigBuilder {
    env_prefix: Option<String>,
    default_host: String,
    default_port: u16,
    default_timeout_secs: u64,
}

impl ServiceConfigBuilder {
    /// Set environment variable prefix (e.g., `MYSERVICE` will look for `MYSERVICE_HOST`, `MYSERVICE_PORT`)
    #[must_use]
    pub fn with_env_prefix(mut self, prefix: &str) -> Self {
        self.env_prefix = Some(prefix.to_string());
        self
    }

    /// Set default host
    #[must_use]
    pub fn with_default_host(mut self, host: &str) -> Self {
        self.default_host = host.to_string();
        self
    }

    /// Set default port
    #[must_use]
    pub const fn with_default_port(mut self, port: u16) -> Self {
        self.default_port = port;
        self
    }

    /// Set default timeout in seconds
    #[must_use]
    pub const fn with_default_timeout_secs(mut self, secs: u64) -> Self {
        self.default_timeout_secs = secs;
        self
    }

    /// Build the configuration, resolving env vars for host, port, and timeout.
    #[must_use]
    pub fn build(self) -> ServiceConfig {
        let prefix = self.env_prefix.unwrap_or_default();

        let host_var = if prefix.is_empty() {
            "HOST".to_string()
        } else {
            format!("{prefix}_HOST")
        };

        let port_var = if prefix.is_empty() {
            "PORT".to_string()
        } else {
            format!("{prefix}_PORT")
        };

        let timeout_var = if prefix.is_empty() {
            "TIMEOUT".to_string()
        } else {
            format!("{prefix}_TIMEOUT")
        };

        let host = get_host(&host_var, &self.default_host);
        let port = get_port(&port_var, self.default_port);
        let timeout_secs = env::var(&timeout_var)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(self.default_timeout_secs);

        ServiceConfig {
            host,
            port,
            timeout_secs,
        }
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            timeout_secs: 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_port_with_default() {
        let port = get_port("NONEXISTENT_PORT_VAR", 3000);
        assert_eq!(port, 3000);
    }

    #[test]
    fn test_get_host_with_default() {
        let host = get_host("NONEXISTENT_HOST_VAR", "example.com");
        assert_eq!(host, "example.com");
    }

    #[test]
    fn test_build_url() {
        let url = build_url("https", "api.example.com", 443, "/v1/endpoint");
        assert_eq!(url, "https://api.example.com:443/v1/endpoint");
    }

    #[test]
    fn test_get_timeout_secs() {
        let timeout = get_timeout_secs("NONEXISTENT_TIMEOUT_VAR", 60);
        assert_eq!(timeout, std::time::Duration::from_secs(60));
    }

    #[test]
    fn test_service_config_builder() {
        // Use unique prefix to avoid environment variable conflicts
        let config = ServiceConfig::builder()
            .with_env_prefix("TESTCFGBUILDER")
            .with_default_host("testhost")
            .with_default_port(9000)
            .with_default_timeout_secs(45)
            .build();

        // May use env vars if set, otherwise defaults
        assert!(!config.host().is_empty());
        assert!(config.port() > 0);
        assert!(config.timeout().as_secs() > 0);
    }

    #[test]
    fn test_service_config_http_url() {
        let config = ServiceConfig::builder()
            .with_default_host("api.service.com")
            .with_default_port(8080)
            .build();

        let url = config.http_url("/health");
        assert_eq!(url, "http://api.service.com:8080/health");
    }

    #[test]
    fn test_service_config_https_url() {
        let config = ServiceConfig::builder()
            .with_default_host("secure.service.com")
            .with_default_port(443)
            .build();

        let url = config.https_url("/api/v1");
        assert_eq!(url, "https://secure.service.com:443/api/v1");
    }

    #[test]
    fn test_service_config_default() {
        let config = ServiceConfig::default();
        assert_eq!(config.host(), "localhost");
        assert_eq!(config.port(), 8080);
        assert_eq!(config.timeout(), std::time::Duration::from_secs(30));
    }
}
