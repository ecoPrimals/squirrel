//! Core configuration types for Squirrel MCP
//!
//! This module defines the fundamental configuration structures and types
//! used throughout the application.

use serde::{Deserialize, Serialize};
use std::env;
use super::service_endpoints::get_service_endpoints;

use crate::core::ai::AIConfig;
use crate::core::ecosystem::EcosystemConfig;
use crate::core::observability::ObservabilityConfig;
use crate::core::security::SecurityConfig;
use crate::environment::Environment;

/// Main configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub network: NetworkConfig,
    pub database: DatabaseConfig,
    pub ai: AIConfig,
    pub security: SecurityConfig,
    pub observability: ObservabilityConfig,
    pub ecosystem: EcosystemConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            database: DatabaseConfig::default(),
            ai: AIConfig::default(),
            security: SecurityConfig::default(),
            observability: ObservabilityConfig::default(),
            ecosystem: EcosystemConfig::default(),
        }
    }
}

/// Network configuration with environment variable support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Server host (default: "127.0.0.1", env: SQUIRREL_HOST)
    pub host: String,
    /// Server port (default: 8080, env: SQUIRREL_PORT)
    pub port: u16,
    /// CORS origins (default: "http://localhost:3000", env: SQUIRREL_CORS_ORIGINS)
    pub cors_origins: Vec<String>,
    /// WebSocket endpoint (env: SQUIRREL_WS_ENDPOINT)
    pub websocket_endpoint: Option<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            host: env::var("SQUIRREL_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("SQUIRREL_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            cors_origins: env::var("SQUIRREL_CORS_ORIGINS")
                .unwrap_or_else(|_| get_service_endpoints().ui_endpoint.clone())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            websocket_endpoint: env::var("SQUIRREL_WS_ENDPOINT").ok(),
        }
    }
}

/// Database configuration with environment variable support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection string (env: DATABASE_URL)
    pub connection_string: String,
    /// Maximum number of connections (env: DB_MAX_CONNECTIONS)
    pub max_connections: u32,
    /// Connection timeout in seconds (env: DB_TIMEOUT)
    pub timeout_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            connection_string: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite::memory:".to_string()),
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            timeout_seconds: env::var("DB_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
        }
    }
}

/// Database backend options
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DatabaseBackend {
    #[serde(rename = "nestgate")]
    NestGate,
    #[serde(rename = "postgres")]
    PostgreSQL,
    #[serde(rename = "sqlite")]
    SQLite,
}

/// External service configuration with environment variable support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalServiceConfig {
    /// Songbird service URL (env: SONGBIRD_URL)
    pub songbird_url: String,
    /// Toadstool service URL (env: TOADSTOOL_URL)
    pub toadstool_url: String,
    /// NestGate service URL (env: NESTGATE_URL)
    pub nestgate_url: String,
    /// BearDog service URL (env: BEARDOG_URL)
    pub beardog_url: String,
    /// BiomeOS service URL (env: BIOMEOS_URL)
    pub biomeos_url: String,
}

impl Default for ExternalServiceConfig {
    fn default() -> Self {
        Self {
            songbird_url: env::var("SONGBIRD_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            toadstool_url: env::var("TOADSTOOL_URL")
                .unwrap_or_else(|_| "http://localhost:8445".to_string()),
            nestgate_url: env::var("NESTGATE_URL")
                .unwrap_or_else(|_| "http://localhost:8444".to_string()),
            beardog_url: env::var("BEARDOG_URL")
                .unwrap_or_else(|_| "http://localhost:8443".to_string()),
            biomeos_url: env::var("BIOMEOS_URL")
                .unwrap_or_else(|_| "http://localhost:5000".to_string()),
        }
    }
}

/// AI service configuration with environment variable support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIServiceConfig {
    /// OpenAI API endpoint (env: OPENAI_API_URL)
    pub openai_api_url: String,
    /// Anthropic API endpoint (env: ANTHROPIC_API_URL)
    pub anthropic_api_url: String,
    /// Ollama API endpoint (env: OLLAMA_API_URL)
    pub ollama_api_url: String,
    /// Default AI model temperature (env: AI_TEMPERATURE)
    pub default_temperature: f32,
    /// Maximum request timeout in seconds (env: AI_TIMEOUT)
    pub timeout_seconds: u64,
}

impl Default for AIServiceConfig {
    fn default() -> Self {
        Self {
            openai_api_url: env::var("OPENAI_API_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            anthropic_api_url: env::var("ANTHROPIC_API_URL")
                .unwrap_or_else(|_| "https://api.anthropic.com/v1".to_string()),
            ollama_api_url: env::var("OLLAMA_API_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            default_temperature: env::var("AI_TEMPERATURE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.7),
            timeout_seconds: env::var("AI_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
        }
    }
}

/// Comprehensive application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub network: NetworkConfig,
    pub database: DatabaseConfig,
    pub external_services: ExternalServiceConfig,
    pub ai_services: AIServiceConfig,
    pub environment: Environment,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            database: DatabaseConfig::default(),
            external_services: ExternalServiceConfig::default(),
            ai_services: AIServiceConfig::default(),
            environment: Environment::from_env(),
        }
    }
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, CoreConfigError> {
        Ok(Self::default())
    }

    /// Load configuration from a file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, CoreConfigError> {
        let contents =
            std::fs::read_to_string(path).map_err(|e| CoreConfigError::IO(e.to_string()))?;

        toml::from_str(&contents).map_err(|e| CoreConfigError::Parse(e.to_string()))
    }

    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), CoreConfigError> {
        let contents =
            toml::to_string_pretty(self).map_err(|e| CoreConfigError::Parse(e.to_string()))?;

        std::fs::write(path, contents).map_err(|e| CoreConfigError::IO(e.to_string()))
    }

    /// Get a service URL with fallback
    pub fn get_service_url(&self, service_name: &str) -> Option<String> {
        match service_name {
            "songbird" => Some(self.external_services.songbird_url.clone()),
            "toadstool" => Some(self.external_services.toadstool_url.clone()),
            "nestgate" => Some(self.external_services.nestgate_url.clone()),
            "beardog" => Some(self.external_services.beardog_url.clone()),
            "biomeos" => Some(self.external_services.biomeos_url.clone()),
            _ => None,
        }
    }
}

/// Core configuration errors
#[derive(Debug, thiserror::Error)]
pub enum CoreConfigError {
    #[error("IO error: {0}")]
    IO(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

/// BiomeOS endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSEndpoints {
    pub ai_api: String,
    pub mcp_api: String,
    pub context_api: String,
    pub health: String,
    pub metrics: String,
    pub websocket: Option<String>,
}

/// Extended observability configuration with additional endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedObservabilityConfig {
    pub dashboard_url: String,
    pub otlp_endpoint: String,
    pub jaeger_endpoint: String,
    pub zipkin_endpoint: String,
    pub metrics_port: u16,
    pub health_port: u16,
}
