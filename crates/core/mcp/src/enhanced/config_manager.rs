// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration Management for Enhanced MCP Platform
//!
//! This module provides centralized configuration management to replace
//! hardcoded values with environment-aware configuration.
//! 
//! Now integrated with the unified configuration system for all timeout values.

use std::collections::HashMap;
use std::env;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::fs;
// Removed: use squirrel_mcp_config::get_service_endpoints;

// Import unified config for timeout management
use squirrel_mcp_config::unified::{ConfigLoader, SquirrelUnifiedConfig};
use std::sync::Arc;

use super::error_types::{EnhancedMCPError, EnhancedResult};

/// Environment types for configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Testing => write!(f, "testing"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
        }
    }
}

impl std::str::FromStr for Environment {
    type Err = EnhancedMCPError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "testing" | "test" => Ok(Environment::Testing),
            "staging" | "stage" => Ok(Environment::Staging),
            "production" | "prod" => Ok(Environment::Production),
            _ => Err(EnhancedMCPError::config_validation(
                "environment",
                format!("Invalid environment: {}", s),
                Some(s),
            )),
        }
    }
}

/// Network configuration with environment-aware defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub host: IpAddr,
    pub port: u16,
    pub bind_address: SocketAddr,
    pub external_url: String,
    pub max_connections: usize,
    pub keep_alive: Duration,
    pub read_timeout: Duration,
    pub write_timeout: Duration,
    pub enable_compression: bool,
    pub enable_tls: bool,
    pub tls_cert_path: Option<PathBuf>,
    pub tls_key_path: Option<PathBuf>,
}

impl NetworkConfig {
    /// Create network configuration for specific environment using unified config
    /// 
    /// Timeouts now come from the unified configuration system with environment-specific scaling:
    /// - Development: 1x base timeouts
    /// - Testing: 0.5x base timeouts (faster feedback)
    /// - Staging: 2x base timeouts
    /// - Production: 4x base timeouts (more conservative)
    pub fn for_environment(env: Environment) -> Self {
        // Load unified config for base timeout values
        let unified_config = ConfigLoader::load()
            .map(|c| c.into_config())
            .ok();
        
        // Get environment-specific timeout multipliers
        let (timeout_multiplier, (host, port, external_url, max_conn, tls, cert, key)) = match env {
            Environment::Development => {
                // Multi-tier development server configuration
                let port = std::env::var("MCP_DEV_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(8080);  // Default development port
                let base_url = format!("http://localhost:{}", port);
                
                (
                    1.0,
                    (
                        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                        port,
                        base_url,
                        100,
                        false,
                        None,
                        None,
                    )
                )
            },
            Environment::Testing => {
                // Multi-tier testing server configuration  
                let port = std::env::var("MCP_TEST_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(8081);  // Default testing port
                let base_url = format!("http://localhost:{}", port);
                
                (
                    0.5,
                    (
                        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                        port,
                        base_url,
                        50,
                        false,
                        None,
                        None,
                    )
                )
            },
            Environment::Staging => (
                2.0,
                (
                    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                    8080,
                    "https://staging.example.com".to_string(),
                    500,
                    true,
                    Some(PathBuf::from("/etc/ssl/certs/staging.crt")),
                    Some(PathBuf::from("/etc/ssl/private/staging.key")),
                )
            ),
            Environment::Production => (
                4.0,
                (
                    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                    8443,
                    "https://api.example.com".to_string(),
                    1000,
                    true,
                    Some(PathBuf::from("/etc/ssl/certs/production.crt")),
                    Some(PathBuf::from("/etc/ssl/private/production.key")),
                )
            ),
        };
        
        // Calculate timeouts from unified config with environment multiplier
        let base_heartbeat = unified_config.as_ref()
            .map(|c| c.timeouts.heartbeat_interval())
            .unwrap_or(Duration::from_secs(30));
        let keep_alive = Duration::from_secs((base_heartbeat.as_secs() as f64 * timeout_multiplier) as u64);
        
        let base_request = unified_config.as_ref()
            .map(|c| c.timeouts.request_timeout())
            .unwrap_or(Duration::from_secs(30));
        let read_timeout = Duration::from_secs((base_request.as_secs() as f64 * timeout_multiplier) as u64);
        let write_timeout = read_timeout;
        
        Self {
            host,
            port,
            bind_address: SocketAddr::new(host, port),
            external_url,
            max_connections: max_conn,
            keep_alive,
            read_timeout,
            write_timeout,
            enable_compression: env != Environment::Testing,
            enable_tls: tls,
            tls_cert_path: cert,
            tls_key_path: key,
        }
    }

    /// Override with environment variables
    pub fn with_env_overrides(mut self) -> EnhancedResult<Self> {
        if let Ok(host) = env::var("MCP_HOST") {
            self.host = host.parse()
                .map_err(|e| EnhancedMCPError::config_validation("host", e, Some(&host)))?;
        }

        if let Ok(port) = env::var("MCP_PORT") {
            self.port = port.parse()
                .map_err(|e| EnhancedMCPError::config_validation("port", e, Some(&port)))?;
        }

        if let Ok(url) = env::var("MCP_EXTERNAL_URL") {
            self.external_url = url;
        }

        if let Ok(max_conn) = env::var("MCP_MAX_CONNECTIONS") {
            self.max_connections = max_conn.parse()
                .map_err(|e| EnhancedMCPError::config_validation("max_connections", e, Some(&max_conn)))?;
        }

        // Update bind address based on host/port changes
        self.bind_address = SocketAddr::new(self.host, self.port);

        Ok(self)
    }

    /// Validate configuration
    pub fn validate(&self) -> EnhancedResult<()> {
        if self.port == 0 {
            return Err(EnhancedMCPError::config_validation(
                "port",
                "Port cannot be zero",
                Some("0"),
            ));
        }

        if self.max_connections == 0 {
            return Err(EnhancedMCPError::config_validation(
                "max_connections",
                "Max connections must be greater than zero",
                Some("0"),
            ));
        }

        if self.enable_tls && (self.tls_cert_path.is_none() || self.tls_key_path.is_none()) {
            return Err(EnhancedMCPError::config_validation(
                "tls",
                "TLS enabled but certificate or key path not provided",
                None,
            ));
        }

        Ok(())
    }
}

/// Database configuration with environment-aware defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub enable_logging: bool,
    pub enable_migrations: bool,
}

impl DatabaseConfig {
    /// Create database configuration for specific environment using unified config
    /// 
    /// Timeouts now come from the unified configuration system with environment-specific scaling
    pub fn for_environment(env: Environment) -> Self {
        // Load unified config for base timeout values
        let unified_config = ConfigLoader::load()
            .map(|c| c.into_config())
            .ok();
        
        let (url, max_conn, min_conn, logging, migrations, timeout_multiplier) = match env {
            Environment::Development => (
                "sqlite:./data/dev.db".to_string(),
                5, 1, true, true, 1.0
            ),
            Environment::Testing => (
                "sqlite::memory:".to_string(),
                1, 1, false, true, 0.2
            ),
            Environment::Staging => (
                "postgres://user:pass@localhost:5432/staging".to_string(),
                20, 5, true, true, 1.0
            ),
            Environment::Production => (
                "postgres://user:pass@db.example.com:5432/production".to_string(),
                50, 10, false, false, 1.0
            ),
        };
        
        // Calculate timeouts from unified config
        let base_db_timeout = unified_config.as_ref()
            .map(|c| c.timeouts.database_timeout())
            .unwrap_or(Duration::from_secs(30));
        let connection_timeout = Duration::from_secs((base_db_timeout.as_secs() as f64 * timeout_multiplier) as u64);
        
        let base_session = unified_config.as_ref()
            .map(|c| c.timeouts.session_timeout())
            .unwrap_or(Duration::from_secs(3600));
        let idle_timeout = Duration::from_secs((base_session.as_secs() as f64 * 0.17 * timeout_multiplier) as u64); // ~10 min for dev/prod
        let max_lifetime = Duration::from_secs((base_session.as_secs() as f64 * 0.5 * timeout_multiplier) as u64); // ~30 min for dev/prod
        
        Self {
            url,
            max_connections: max_conn,
            min_connections: min_conn,
            connection_timeout,
            idle_timeout,
            max_lifetime,
            enable_logging: logging,
            enable_migrations: migrations,
        }
    }

    /// Override with environment variables
    pub fn with_env_overrides(mut self) -> EnhancedResult<Self> {
        if let Ok(url) = env::var("DATABASE_URL") {
            self.url = url;
        }

        if let Ok(max_conn) = env::var("DATABASE_MAX_CONNECTIONS") {
            self.max_connections = max_conn.parse()
                .map_err(|e| EnhancedMCPError::config_validation("max_connections", e, Some(&max_conn)))?;
        }

        if let Ok(min_conn) = env::var("DATABASE_MIN_CONNECTIONS") {
            self.min_connections = min_conn.parse()
                .map_err(|e| EnhancedMCPError::config_validation("min_connections", e, Some(&min_conn)))?;
        }

        Ok(self)
    }

    /// Validate configuration
    pub fn validate(&self) -> EnhancedResult<()> {
        if self.url.is_empty() {
            return Err(EnhancedMCPError::config_validation(
                "database_url",
                "Database URL cannot be empty",
                Some(""),
            ));
        }

        if self.max_connections < self.min_connections {
            return Err(EnhancedMCPError::config_validation(
                "database_connections",
                "Max connections must be greater than or equal to min connections",
                Some(&format!("max: {}, min: {}", self.max_connections, self.min_connections)),
            ));
        }

        Ok(())
    }
}

/// Security configuration with environment-aware defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration: Duration,
    pub api_key_length: usize,
    pub rate_limit_requests: usize,
    pub rate_limit_window: Duration,
    pub enable_cors: bool,
    pub cors_origins: Vec<String>,
    pub enable_csrf: bool,
    pub session_timeout: Duration,
    pub max_login_attempts: usize,
    pub lockout_duration: Duration,
}

impl SecurityConfig {
    /// Create security configuration for specific environment using unified config
    /// 
    /// Timeouts now come from the unified configuration system with environment-specific scaling
    pub fn for_environment(env: Environment) -> Self {
        // Load unified config for base timeout values
        let unified_config = ConfigLoader::load()
            .map(|c| c.into_config())
            .ok();
        
        let (secret, api_len, rate_reqs, cors_origins, csrf, max_attempts, timeout_multiplier) = match env {
            Environment::Development => {
                // Multi-tier CORS origins for development
                let cors_origin = std::env::var("CORS_ORIGINS")
                    .or_else(|_| std::env::var("WEB_UI_URL"))
                    .unwrap_or_else(|_| {
                        let port = std::env::var("WEB_UI_PORT")
                            .ok()
                            .and_then(|p| p.parse::<u16>().ok())
                            .unwrap_or(3000);  // Default Web UI port
                        format!("http://localhost:{}", port)
                    });
                
                (
                    "dev-secret-key-must-be-at-least-32-characters-long".to_string(),
                    32, 1000, vec![cors_origin], false, 10, 1.0
                )
            },
            Environment::Testing => (
                "test-secret-key-must-be-at-least-32-characters-long".to_string(),
                16, 100, vec!["*".to_string()], false, 5, 0.25
            ),
            Environment::Staging => (
                "staging-secret-key-must-be-at-least-32-characters-long".to_string(),
                64, 500, vec!["https://staging.example.com".to_string()], true, 5, 0.5
            ),
            Environment::Production => (
                "production-secret-key-must-be-at-least-32-characters-long".to_string(),
                128, 100, vec!["https://api.example.com".to_string()], true, 3, 0.5
            ),
        };
        
        // Calculate timeouts from unified config
        let base_session = unified_config.as_ref()
            .map(|c| c.timeouts.session_timeout())
            .unwrap_or(Duration::from_secs(3600));
        let jwt_expiration = Duration::from_secs((base_session.as_secs() as f64 * 0.25 * timeout_multiplier) as u64); // 15 min base
        let session_timeout = Duration::from_secs((base_session.as_secs() as f64 * 2.0 * timeout_multiplier) as u64); // 2h base
        let lockout_duration = Duration::from_secs((base_session.as_secs() as f64 * 0.083 * timeout_multiplier) as u64); // 5 min base
        
        let rate_limit_window = Duration::from_secs(60); // Always 60s for rate limiting
        
        Self {
            jwt_secret: secret,
            jwt_expiration,
            api_key_length: api_len,
            rate_limit_requests: rate_reqs,
            rate_limit_window,
            enable_cors: true,
            cors_origins,
            enable_csrf: csrf,
            session_timeout,
            max_login_attempts: max_attempts,
            lockout_duration,
        }
    }

    /// Override with environment variables
    pub fn with_env_overrides(mut self) -> EnhancedResult<Self> {
        if let Ok(secret) = env::var("JWT_SECRET") {
            self.jwt_secret = secret;
        }

        if let Ok(expiration) = env::var("JWT_EXPIRATION_SECONDS") {
            let seconds: u64 = expiration.parse()
                .map_err(|e| EnhancedMCPError::config_validation("jwt_expiration", e, Some(&expiration)))?;
            self.jwt_expiration = Duration::from_secs(seconds);
        }

        if let Ok(requests) = env::var("RATE_LIMIT_REQUESTS") {
            self.rate_limit_requests = requests.parse()
                .map_err(|e| EnhancedMCPError::config_validation("rate_limit_requests", e, Some(&requests)))?;
        }

        if let Ok(origins) = env::var("CORS_ORIGINS") {
            self.cors_origins = origins.split(',').map(|s| s.trim().to_string()).collect();
        }

        Ok(self)
    }

    /// Validate configuration
    pub fn validate(&self) -> EnhancedResult<()> {
        if self.jwt_secret.len() < 32 {
            return Err(EnhancedMCPError::config_validation(
                "jwt_secret",
                "JWT secret must be at least 32 characters long",
                Some(&format!("length: {}", self.jwt_secret.len())),
            ));
        }

        if self.api_key_length < 16 {
            return Err(EnhancedMCPError::config_validation(
                "api_key_length",
                "API key length must be at least 16 characters",
                Some(&self.api_key_length.to_string()),
            ));
        }

        if self.rate_limit_requests == 0 {
            return Err(EnhancedMCPError::config_validation(
                "rate_limit_requests",
                "Rate limit requests must be greater than zero",
                Some("0"),
            ));
        }

        Ok(())
    }
}

/// Complete Enhanced MCP Platform Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMCPConfig {
    pub environment: Environment,
    pub network: NetworkConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub features: HashMap<String, bool>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl EnhancedMCPConfig {
    /// Create configuration for specific environment
    pub fn for_environment(env: Environment) -> Self {
        let mut features = HashMap::new();
        features.insert("streaming".to_string(), true);
        features.insert("websockets".to_string(), true);
        features.insert("metrics".to_string(), true);
        features.insert("health_checks".to_string(), true);

        match env {
            Environment::Development => {
                features.insert("debug_logging".to_string(), true);
                features.insert("hot_reload".to_string(), true);
            }
            Environment::Testing => {
                features.insert("mock_providers".to_string(), true);
                features.insert("test_endpoints".to_string(), true);
            }
            Environment::Production => {
                features.insert("performance_monitoring".to_string(), true);
                features.insert("security_headers".to_string(), true);
            }
            _ => {}
        }

        Self {
            environment: env,
            network: NetworkConfig::for_environment(env),
            database: DatabaseConfig::for_environment(env),
            security: SecurityConfig::for_environment(env),
            features,
            metadata: HashMap::new(),
        }
    }

    /// Load configuration from environment and file
    pub async fn load() -> EnhancedResult<Self> {
        let env_str = env::var("MCP_ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let environment: Environment = env_str.parse()?;

        let mut config = Self::for_environment(environment);

        // Override with environment variables
        config.network = config.network.with_env_overrides()?;
        config.database = config.database.with_env_overrides()?;
        config.security = config.security.with_env_overrides()?;

        // Load from config file if exists
        if let Ok(config_path) = env::var("MCP_CONFIG_FILE") {
            config = config.load_from_file(&config_path).await?;
        }

        // Validate complete configuration
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from file
    pub async fn load_from_file(mut self, path: &str) -> EnhancedResult<Self> {
        let contents = fs::read_to_string(path).await
            .map_err(|e| EnhancedMCPError::config_validation("config_file", e, Some(path)))?;

        let file_config: EnhancedMCPConfig = toml::from_str(&contents)
            .map_err(|e| EnhancedMCPError::config_validation("config_parse", e, Some(path)))?;

        // Merge configurations (file overrides defaults)
        self.merge_from(file_config);

        Ok(self)
    }

    /// Merge configuration from another config
    pub fn merge_from(&mut self, other: EnhancedMCPConfig) {
        // Merge features
        for (key, value) in other.features {
            self.features.insert(key, value);
        }

        // Merge metadata
        for (key, value) in other.metadata {
            self.metadata.insert(key, value);
        }

        // Override network, database, and security if provided
        self.network = other.network;
        self.database = other.database;
        self.security = other.security;
    }

    /// Validate complete configuration
    pub fn validate(&self) -> EnhancedResult<()> {
        self.network.validate()?;
        self.database.validate()?;
        self.security.validate()?;
        Ok(())
    }

    /// Get feature flag value
    pub fn get_feature(&self, name: &str) -> bool {
        self.features.get(name).copied().unwrap_or(false)
    }

    /// Set feature flag value
    pub fn set_feature(&mut self, name: &str, value: bool) {
        self.features.insert(name.to_string(), value);
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Set metadata value
    pub fn set_metadata(&mut self, key: &str, value: serde_json::Value) {
        self.metadata.insert(key.to_string(), value);
    }

    /// Get display-friendly configuration summary
    pub fn summary(&self) -> String {
        format!(
            "Enhanced MCP Platform Configuration\n\
            Environment: {}\n\
            Network: {}:{}\n\
            Database: {}\n\
            Features: {} enabled\n\
            Security: {} enabled",
            self.environment,
            self.network.host,
            self.network.port,
            if self.database.url.contains("memory") { "In-Memory" } else { "Persistent" },
            self.features.len(),
            if self.security.enable_cors { "CORS" } else { "Basic" }
        )
    }
}

/// Configuration manager for runtime configuration access
pub struct ConfigManager {
    config: EnhancedMCPConfig,
}

impl ConfigManager {
    /// Create new configuration manager
    pub async fn new() -> EnhancedResult<Self> {
        let config = EnhancedMCPConfig::load().await?;
        Ok(Self { config })
    }

    /// Get current configuration
    pub fn get_config(&self) -> &EnhancedMCPConfig {
        &self.config
    }

    /// Reload configuration
    pub async fn reload(&mut self) -> EnhancedResult<()> {
        self.config = EnhancedMCPConfig::load().await?;
        Ok(())
    }

    /// Get network configuration
    pub fn network(&self) -> &NetworkConfig {
        &self.config.network
    }

    /// Get database configuration
    pub fn database(&self) -> &DatabaseConfig {
        &self.config.database
    }

    /// Get security configuration
    pub fn security(&self) -> &SecurityConfig {
        &self.config.security
    }

    /// Check if feature is enabled
    pub fn feature_enabled(&self, name: &str) -> bool {
        self.config.get_feature(name)
    }

    /// Get configuration summary
    pub fn summary(&self) -> String {
        self.config.summary()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_parsing() {
        use crate::error_handling::safe_operations::SafeOps;
        
        // Test development environment parsing with safe operations
        let dev_result = SafeOps::safe_parse::<Environment>("development", "environment_parsing_test");
        assert_eq!(
            dev_result.unwrap_or_default(), 
            Environment::Development,
            "Should parse 'development' environment correctly"
        );
        
        // Test production environment parsing with safe operations  
        let prod_result = SafeOps::safe_parse::<Environment>("prod", "environment_parsing_test");
        assert_eq!(
            prod_result.unwrap_or_default(),
            Environment::Production,
            "Should parse 'prod' environment correctly"
        );
    }

    #[test]
    fn test_network_config_validation() {
        let mut config = NetworkConfig::for_environment(Environment::Development);
        assert!(config.validate().is_ok());

        config.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_database_config_validation() {
        let mut config = DatabaseConfig::for_environment(Environment::Development);
        assert!(config.validate().is_ok());

        config.url = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_security_config_validation() {
        let mut config = SecurityConfig::for_environment(Environment::Development);
        assert!(config.validate().is_ok());

        config.jwt_secret = "short".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_feature_flags() {
        let mut config = EnhancedMCPConfig::for_environment(Environment::Development);
        assert!(config.get_feature("streaming"));
        assert!(!config.get_feature("nonexistent"));

        config.set_feature("custom_feature", true);
        assert!(config.get_feature("custom_feature"));
    }

    #[test]
    fn test_environment_specific_defaults() {
        let dev_config = EnhancedMCPConfig::for_environment(Environment::Development);
        let prod_config = EnhancedMCPConfig::for_environment(Environment::Production);

        assert_eq!(dev_config.network.port, 8080);
        assert_eq!(prod_config.network.port, 443);
        assert!(dev_config.get_feature("debug_logging"));
        assert!(!prod_config.get_feature("debug_logging"));
    }

    #[tokio::test]
    async fn test_config_manager_creation() {
        // This test requires proper environment setup
        // In a real scenario, you'd set up test environment variables
        let result = ConfigManager::new().await;
        // We expect this to work with default development settings
        assert!(result.is_ok());
    }
} 