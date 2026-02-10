// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Modern Database Configuration Module
//!
//! This module provides a modernized, testable database configuration with:
//! - Type-safe DatabaseBackend enum (already exists!)
//! - Connection string validation
//! - Builder pattern for easy construction
//! - Database-specific presets
//! - Sensible defaults per backend
//!
//! # Example
//!
//! ```rust
//! use squirrel_mcp_config::unified::database::{DatabaseConfig, DatabaseBackend};
//!
//! // For testing - in-memory SQLite
//! let config = DatabaseConfig::testing();
//!
//! // Builder pattern
//! let config = DatabaseConfig::builder()
//!     .backend(DatabaseBackend::PostgreSQL)
//!     .host("localhost")
//!     .database("mydb")
//!     .build();
//!
//! // PostgreSQL preset
//! let config = DatabaseConfig::postgres("localhost", "mydb");
//! ```

use serde::{Deserialize, Serialize};
use universal_constants::deployment;

// Re-export DatabaseBackend from types (already exists and is good!)
pub use super::types::DatabaseBackend;

// Note: Only PostgreSQL, SQLite, Memory, and NestGate are supported
// MySQL is not in the DatabaseBackend enum

/// Modern database configuration with builder pattern
///
/// This replaces the old monolithic DatabaseConfig with a more testable,
/// ergonomic design with backend-specific helpers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database backend
    #[serde(default)]
    backend: DatabaseBackend,

    /// Database host
    #[serde(default = "default_host")]
    host: String,

    /// Database port
    #[serde(default)]
    port: Option<u16>,

    /// Database name
    #[serde(default = "default_database")]
    database: String,

    /// Username (optional for SQLite)
    #[serde(default)]
    username: Option<String>,

    /// Password (optional for SQLite)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    password: Option<String>,

    /// Maximum connections in pool
    #[serde(default = "default_max_connections")]
    max_connections: u32,

    /// Minimum idle connections
    #[serde(default = "default_min_connections")]
    min_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    connection_timeout: u64,

    /// Enable TLS/SSL
    #[serde(default)]
    enable_tls: bool,
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_database() -> String {
    "squirrel".to_string()
}

fn default_max_connections() -> u32 {
    20
}

fn default_min_connections() -> u32 {
    2
}

fn default_connection_timeout() -> u64 {
    30
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            backend: DatabaseBackend::SQLite,
            host: default_host(),
            port: None,
            database: default_database(),
            username: None,
            password: None,
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connection_timeout: default_connection_timeout(),
            enable_tls: false,
        }
    }
}

impl DatabaseConfig {
    /// Create a new builder for DatabaseConfig
    pub fn builder() -> DatabaseConfigBuilder {
        DatabaseConfigBuilder::default()
    }

    /// Create a config for testing (in-memory SQLite)
    pub fn testing() -> Self {
        Self {
            backend: DatabaseBackend::SQLite,
            host: ":memory:".to_string(),
            port: None,
            database: ":memory:".to_string(),
            username: None,
            password: None,
            max_connections: 1, // SQLite in-memory is single connection
            min_connections: 1,
            connection_timeout: 5,
            enable_tls: false,
        }
    }

    /// Create a PostgreSQL config with sensible defaults
    pub fn postgres(host: impl Into<String>, database: impl Into<String>) -> Self {
        Self {
            backend: DatabaseBackend::PostgreSQL,
            host: host.into(),
            port: Some(deployment::ports::postgres()), // Default PostgreSQL port
            database: database.into(),
            username: Some("postgres".to_string()),
            password: None, // Must be set explicitly
            max_connections: 20,
            min_connections: 2,
            connection_timeout: 30,
            enable_tls: false,
        }
    }

    // MySQL removed - not in DatabaseBackend enum

    /// Create a SQLite config (file-based)
    pub fn sqlite(path: impl Into<String>) -> Self {
        Self {
            backend: DatabaseBackend::SQLite,
            host: "localhost".to_string(), // Not used for SQLite
            port: None,
            database: path.into(),
            username: None,
            password: None,
            max_connections: 1, // SQLite typically single connection
            min_connections: 1,
            connection_timeout: 5,
            enable_tls: false,
        }
    }

    // Getters with clear ownership
    pub fn backend(&self) -> DatabaseBackend {
        self.backend
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn database(&self) -> &str {
        &self.database
    }

    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    pub fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    pub fn max_connections(&self) -> u32 {
        self.max_connections
    }

    pub fn min_connections(&self) -> u32 {
        self.min_connections
    }

    pub fn connection_timeout(&self) -> u64 {
        self.connection_timeout
    }

    pub fn is_tls_enabled(&self) -> bool {
        self.enable_tls
    }

    /// Build a connection string for this database
    pub fn connection_string(&self) -> String {
        match self.backend {
            DatabaseBackend::PostgreSQL => {
                let mut parts = vec![format!("postgresql://{}", self.host)];

                if let Some(port) = self.port {
                    parts[0] = format!("postgresql://{}:{}", self.host, port);
                }

                if let Some(username) = &self.username {
                    let password = self.password.as_deref().unwrap_or("");
                    parts[0] = format!(
                        "postgresql://{}:{}@{}{}",
                        username,
                        password,
                        self.host,
                        self.port.map(|p| format!(":{}", p)).unwrap_or_default()
                    );
                }

                format!("{}/{}", parts[0], self.database)
            }
            // MySQL is not in the backend enum, removed for now
            DatabaseBackend::SQLite => {
                format!("sqlite://{}", self.database)
            }
            DatabaseBackend::Memory => "sqlite://:memory:".to_string(),
            DatabaseBackend::NestGate => {
                // NestGate uses custom protocol
                format!(
                    "nestgate://{}:{}/{}",
                    self.host,
                    self.port.unwrap_or(9000),
                    self.database
                )
            }
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), DatabaseConfigError> {
        // Check database name is not empty
        if self.database.is_empty() {
            return Err(DatabaseConfigError::EmptyDatabase);
        }

        // PostgreSQL requires username in production scenarios
        if matches!(self.backend, DatabaseBackend::PostgreSQL) {
            if self.username.is_none() {
                return Err(DatabaseConfigError::UsernameRequired {
                    backend: self.backend,
                });
            }
        }

        // Check pool sizes make sense
        if self.max_connections == 0 {
            return Err(DatabaseConfigError::InvalidPoolSize {
                max: self.max_connections,
                min: self.min_connections,
            });
        }

        if self.min_connections > self.max_connections {
            return Err(DatabaseConfigError::InvalidPoolSize {
                max: self.max_connections,
                min: self.min_connections,
            });
        }

        // Check timeout is reasonable
        if self.connection_timeout == 0 {
            return Err(DatabaseConfigError::InvalidTimeout(self.connection_timeout));
        }

        Ok(())
    }
}

/// Builder for DatabaseConfig
#[derive(Debug, Default)]
pub struct DatabaseConfigBuilder {
    backend: Option<DatabaseBackend>,
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    max_connections: Option<u32>,
    min_connections: Option<u32>,
    connection_timeout: Option<u64>,
    enable_tls: Option<bool>,
}

impl DatabaseConfigBuilder {
    /// Set the database backend
    pub fn backend(mut self, backend: DatabaseBackend) -> Self {
        self.backend = Some(backend);
        self
    }

    /// Set the host
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Set the port
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Set the database name
    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    /// Set the username
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Set the password
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Set the maximum connections
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = Some(max);
        self
    }

    /// Set the minimum connections
    pub fn min_connections(mut self, min: u32) -> Self {
        self.min_connections = Some(min);
        self
    }

    /// Set the connection timeout
    pub fn connection_timeout(mut self, timeout: u64) -> Self {
        self.connection_timeout = Some(timeout);
        self
    }

    /// Enable or disable TLS
    pub fn enable_tls(mut self, enable: bool) -> Self {
        self.enable_tls = Some(enable);
        self
    }

    /// Build the DatabaseConfig
    pub fn build(self) -> DatabaseConfig {
        DatabaseConfig {
            backend: self.backend.unwrap_or_default(),
            host: self.host.unwrap_or_else(default_host),
            port: self.port,
            database: self.database.unwrap_or_else(default_database),
            username: self.username,
            password: self.password,
            max_connections: self.max_connections.unwrap_or_else(default_max_connections),
            min_connections: self.min_connections.unwrap_or_else(default_min_connections),
            connection_timeout: self
                .connection_timeout
                .unwrap_or_else(default_connection_timeout),
            enable_tls: self.enable_tls.unwrap_or(false),
        }
    }
}

/// Database configuration errors
#[derive(Debug, thiserror::Error)]
pub enum DatabaseConfigError {
    #[error("Database name cannot be empty")]
    EmptyDatabase,

    #[error("Username is required for {backend:?}")]
    UsernameRequired { backend: DatabaseBackend },

    #[error("Invalid pool size: max={max}, min={min} (min must be <= max and max must be > 0)")]
    InvalidPoolSize { max: u32, min: u32 },

    #[error("Invalid connection timeout: {0} (must be > 0)")]
    InvalidTimeout(u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== DatabaseConfig Tests ==========

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert_eq!(config.backend(), DatabaseBackend::SQLite);
        assert_eq!(config.host(), "localhost");
        assert_eq!(config.database(), "squirrel");
    }

    #[test]
    fn test_testing_config() {
        let config = DatabaseConfig::testing();
        assert_eq!(config.backend(), DatabaseBackend::SQLite);
        assert_eq!(config.host(), ":memory:");
        assert_eq!(config.database(), ":memory:");
        assert_eq!(config.max_connections(), 1);
    }

    #[test]
    fn test_postgres_preset() {
        let config = DatabaseConfig::postgres("db.example.com", "mydb");
        assert_eq!(config.backend(), DatabaseBackend::PostgreSQL);
        assert_eq!(config.host(), "db.example.com");
        assert_eq!(config.database(), "mydb");
        assert_eq!(config.port(), Some(5432));
        assert_eq!(config.username(), Some("postgres"));
    }

    // MySQL test removed - not in DatabaseBackend enum

    #[test]
    fn test_sqlite_preset() {
        let config = DatabaseConfig::sqlite("/var/data/squirrel.db");
        assert_eq!(config.backend(), DatabaseBackend::SQLite);
        assert_eq!(config.database(), "/var/data/squirrel.db");
        assert!(config.username().is_none());
    }

    #[test]
    fn test_builder() {
        let config = DatabaseConfig::builder()
            .backend(DatabaseBackend::PostgreSQL)
            .host("localhost")
            .port(5433)
            .database("testdb")
            .username("user")
            .password("pass")
            .max_connections(10)
            .build();

        assert_eq!(config.backend(), DatabaseBackend::PostgreSQL);
        assert_eq!(config.host(), "localhost");
        assert_eq!(config.port(), Some(5433));
        assert_eq!(config.database(), "testdb");
        assert_eq!(config.username(), Some("user"));
        assert_eq!(config.password(), Some("pass"));
        assert_eq!(config.max_connections(), 10);
    }

    #[test]
    fn test_builder_with_defaults() {
        let config = DatabaseConfig::builder()
            .backend(DatabaseBackend::PostgreSQL)
            .username("testuser") // PostgreSQL requires username
            .build();

        // Should use defaults for unspecified fields
        assert_eq!(config.backend(), DatabaseBackend::PostgreSQL);
        assert_eq!(config.host(), "localhost");
        assert_eq!(config.max_connections(), 20);
    }

    #[test]
    fn test_connection_string_postgres() {
        let config = DatabaseConfig::builder()
            .backend(DatabaseBackend::PostgreSQL)
            .host("localhost")
            .port(5432)
            .database("mydb")
            .username("user")
            .password("pass")
            .build();

        let conn_str = config.connection_string();
        assert!(conn_str.starts_with("postgresql://"));
        assert!(conn_str.contains("user:pass"));
        assert!(conn_str.contains("localhost:5432"));
        assert!(conn_str.ends_with("/mydb"));
    }

    // MySQL connection string test removed - not in DatabaseBackend enum

    #[test]
    fn test_connection_string_sqlite() {
        let config = DatabaseConfig::sqlite("/tmp/test.db");
        let conn_str = config.connection_string();
        assert_eq!(conn_str, "sqlite:///tmp/test.db");
    }

    #[test]
    fn test_validation_empty_database() {
        let mut config = DatabaseConfig::testing();
        config.database = String::new();

        let result = config.validate();
        assert!(matches!(result, Err(DatabaseConfigError::EmptyDatabase)));
    }

    #[test]
    fn test_validation_username_required() {
        let mut config = DatabaseConfig::postgres("localhost", "mydb");
        config.username = None;

        let result = config.validate();
        assert!(matches!(
            result,
            Err(DatabaseConfigError::UsernameRequired { .. })
        ));
    }

    #[test]
    fn test_validation_invalid_pool_size() {
        let mut config = DatabaseConfig::testing();
        config.max_connections = 0;

        let result = config.validate();
        assert!(matches!(
            result,
            Err(DatabaseConfigError::InvalidPoolSize { .. })
        ));

        // Min > Max
        config.max_connections = 10;
        config.min_connections = 20;
        let result = config.validate();
        assert!(matches!(
            result,
            Err(DatabaseConfigError::InvalidPoolSize { .. })
        ));
    }

    #[test]
    fn test_validation_invalid_timeout() {
        let mut config = DatabaseConfig::testing();
        config.connection_timeout = 0;

        let result = config.validate();
        assert!(matches!(
            result,
            Err(DatabaseConfigError::InvalidTimeout(_))
        ));
    }

    #[test]
    fn test_validation_success() {
        let config = DatabaseConfig::testing();
        assert!(config.validate().is_ok());

        let config = DatabaseConfig::postgres("localhost", "db");
        assert!(config.validate().is_ok());

        // MySQL test removed - not in DatabaseBackend enum
    }
}
