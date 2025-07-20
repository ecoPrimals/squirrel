//! Network configuration module
//!
//! This module provides centralized configuration for network addresses, ports,
//! and endpoints to replace hardcoded values throughout the codebase.

use serde::{Deserialize, Serialize};
use std::env;
use url::Url;

/// Default network configuration values
pub mod defaults {
    pub const DEFAULT_HOST: &str = "127.0.0.1";
    pub const DEFAULT_BIND_HOST: &str = "0.0.0.0";
    pub const DEFAULT_HTTP_PORT: u16 = 8080;
    pub const DEFAULT_HTTPS_PORT: u16 = 8443;
    pub const DEFAULT_MCP_PORT: u16 = 8444;
    pub const DEFAULT_BEARDOG_PORT: u16 = 8443;
    pub const DEFAULT_SONGBIRD_PORT: u16 = 8080;
    pub const DEFAULT_WEBSOCKET_PORT: u16 = 8080;
    pub const LOCALHOST: &str = "localhost";
}

/// Network endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Host address for binding services
    pub bind_host: String,

    /// Host address for connecting to services
    pub host: String,

    /// HTTP port
    pub http_port: u16,

    /// HTTPS port
    pub https_port: u16,

    /// MCP protocol port
    pub mcp_port: u16,

    /// WebSocket port
    pub websocket_port: u16,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_host: env::var("BIND_HOST")
                .unwrap_or_else(|_| defaults::DEFAULT_BIND_HOST.to_string()),
            host: env::var("HOST").unwrap_or_else(|_| defaults::DEFAULT_HOST.to_string()),
            http_port: env::var("HTTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(defaults::DEFAULT_HTTP_PORT),
            https_port: env::var("HTTPS_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(defaults::DEFAULT_HTTPS_PORT),
            mcp_port: env::var("MCP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(defaults::DEFAULT_MCP_PORT),
            websocket_port: env::var("WEBSOCKET_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(defaults::DEFAULT_WEBSOCKET_PORT),
        }
    }
}

/// Service endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    /// Songbird service endpoint
    pub songbird_endpoint: String,

    /// Beardog security service endpoint
    pub beardog_endpoint: String,

    /// BiomeOS endpoint
    pub biomeos_endpoint: String,

    /// Health check endpoint
    pub health_endpoint: String,

    /// Metrics endpoint
    pub metrics_endpoint: String,

    /// Admin endpoint
    pub admin_endpoint: String,

    /// WebSocket endpoint
    pub websocket_endpoint: Option<String>,
}

impl Default for ServiceEndpoints {
    fn default() -> Self {
        let network = NetworkConfig::default();
        let base_url = format!("http://{}:{}", network.host, network.http_port);
        let secure_url = format!("https://{}:{}", network.host, network.https_port);
        let websocket_url = format!("ws://{}:{}/ws", network.host, network.websocket_port);

        Self {
            songbird_endpoint: env::var("SONGBIRD_ENDPOINT").unwrap_or_else(|_| base_url.clone()),
            beardog_endpoint: env::var("BEARDOG_ENDPOINT").unwrap_or_else(|_| secure_url.clone()),
            biomeos_endpoint: env::var("BIOMEOS_ENDPOINT").unwrap_or_else(|_| base_url.clone()),
            health_endpoint: env::var("HEALTH_ENDPOINT")
                .unwrap_or_else(|_| format!("{}/health", base_url)),
            metrics_endpoint: env::var("METRICS_ENDPOINT")
                .unwrap_or_else(|_| format!("{}/metrics", base_url)),
            admin_endpoint: env::var("ADMIN_ENDPOINT")
                .unwrap_or_else(|_| format!("{}/admin", base_url)),
            websocket_endpoint: Some(
                env::var("WEBSOCKET_ENDPOINT").unwrap_or_else(|_| websocket_url),
            ),
        }
    }
}

/// Development environment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentConfig {
    /// Use localhost instead of bind addresses
    pub use_localhost: bool,

    /// Enable mock services
    pub enable_mocks: bool,

    /// Development ports offset
    pub port_offset: u16,
}

impl Default for DevelopmentConfig {
    fn default() -> Self {
        Self {
            use_localhost: env::var("DEV_USE_LOCALHOST")
                .map(|v| v == "true")
                .unwrap_or(true),
            enable_mocks: env::var("DEV_ENABLE_MOCKS")
                .map(|v| v == "true")
                .unwrap_or(false),
            port_offset: env::var("DEV_PORT_OFFSET")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(0),
        }
    }
}

impl NetworkConfig {
    /// Get the appropriate host for the environment
    pub fn get_effective_host(&self, dev_config: &DevelopmentConfig) -> String {
        if dev_config.use_localhost {
            defaults::LOCALHOST.to_string()
        } else {
            self.host.clone()
        }
    }

    /// Get the HTTP URL for the service
    pub fn get_http_url(&self, dev_config: &DevelopmentConfig) -> String {
        let host = self.get_effective_host(dev_config);
        let port = self.http_port + dev_config.port_offset;
        format!("http://{}:{}", host, port)
    }

    /// Get the HTTPS URL for the service
    pub fn get_https_url(&self, dev_config: &DevelopmentConfig) -> String {
        let host = self.get_effective_host(dev_config);
        let port = self.https_port + dev_config.port_offset;
        format!("https://{}:{}", host, port)
    }

    /// Get the WebSocket URL for the service
    pub fn get_websocket_url(&self, dev_config: &DevelopmentConfig) -> String {
        let host = self.get_effective_host(dev_config);
        let port = self.websocket_port + dev_config.port_offset;
        format!("ws://{}:{}/ws", host, port)
    }
}

impl ServiceEndpoints {
    /// Create endpoints for development environment
    pub fn for_development() -> Self {
        let network = NetworkConfig::default();
        let dev_config = DevelopmentConfig::default();

        let base_url = network.get_http_url(&dev_config);
        let secure_url = network.get_https_url(&dev_config);
        let websocket_url = network.get_websocket_url(&dev_config);

        Self {
            songbird_endpoint: base_url.clone(),
            beardog_endpoint: secure_url,
            biomeos_endpoint: base_url.clone(),
            health_endpoint: format!("{}/health", base_url),
            metrics_endpoint: format!("{}/metrics", base_url),
            admin_endpoint: format!("{}/admin", base_url),
            websocket_endpoint: Some(websocket_url),
        }
    }

    /// Create endpoints for production environment
    pub fn for_production() -> Self {
        Self::default()
    }

    /// Validate all endpoints
    pub fn validate(&self) -> Result<(), String> {
        let endpoints = vec![
            ("songbird", &self.songbird_endpoint),
            ("beardog", &self.beardog_endpoint),
            ("biomeos", &self.biomeos_endpoint),
            ("health", &self.health_endpoint),
            ("metrics", &self.metrics_endpoint),
            ("admin", &self.admin_endpoint),
        ];

        for (name, endpoint) in endpoints {
            if let Err(e) = Url::parse(endpoint) {
                return Err(format!("Invalid {} endpoint '{}': {}", name, endpoint, e));
            }
        }

        if let Some(ws_endpoint) = &self.websocket_endpoint {
            if !ws_endpoint.starts_with("ws://") && !ws_endpoint.starts_with("wss://") {
                return Err(format!(
                    "Invalid WebSocket endpoint '{}': must start with ws:// or wss://",
                    ws_endpoint
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.http_port, defaults::DEFAULT_HTTP_PORT);
        assert_eq!(config.https_port, defaults::DEFAULT_HTTPS_PORT);
    }

    #[test]
    fn test_service_endpoints_validation() {
        let endpoints = ServiceEndpoints::for_development();
        assert!(endpoints.validate().is_ok());
    }

    #[test]
    fn test_development_config() {
        let network = NetworkConfig::default();
        let dev_config = DevelopmentConfig::default();

        let url = network.get_http_url(&dev_config);
        assert!(url.contains("localhost") || url.contains("127.0.0.1"));
    }
}
