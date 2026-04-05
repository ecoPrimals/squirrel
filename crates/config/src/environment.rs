// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use serde::{Deserialize, Serialize};
use std::env;
use std::str::FromStr;
use thiserror::Error;
use tracing::warn;
use universal_constants::capabilities;
use universal_constants::config_helpers;
use universal_constants::deployment::ports;
use universal_constants::network::{BIND_ALL_INTERFACES, LOCALHOST_IPV4};
use universal_constants::timeouts;

/// Environment configuration errors
#[derive(Debug, Error)]
pub enum EnvironmentError {
    /// A required environment variable was not set.
    #[error("Missing required environment variable: {0}")]
    MissingVariable(String),

    /// An environment variable had an invalid or unexpected value.
    #[error("Invalid environment variable value: {variable} = {value}")]
    InvalidValue {
        /// Name of the environment variable.
        variable: String,
        /// The invalid value that was read.
        value: String,
    },

    /// Failed to parse an environment variable into the expected type.
    #[error("Parse error for variable {variable}: {error}")]
    ParseError {
        /// Name of the environment variable.
        variable: String,
        /// Parse error message.
        error: String,
    },
}

/// Application environment types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    /// Local development with relaxed validation and defaults.
    Development,
    /// Automated testing with in-memory backends where applicable.
    Testing,
    /// Pre-production staging with production-like validation.
    Staging,
    /// Production deployment with strict validation and required secrets.
    Production,
}

impl Environment {
    /// Get environment from `MCP_ENV` variable
    #[must_use]
    pub fn from_env() -> Self {
        match env::var("MCP_ENV")
            .unwrap_or_else(|_| "development".to_string())
            .as_str()
        {
            "production" => Self::Production,
            "staging" => Self::Staging,
            "testing" => Self::Testing,
            _ => Self::Development,
        }
    }

    /// Check if running in development mode
    #[must_use]
    pub const fn is_development(&self) -> bool {
        matches!(self, Self::Development)
    }

    /// Check if running in production mode
    #[must_use]
    pub const fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }

    /// Get configuration file suffix
    #[must_use]
    pub const fn config_suffix(&self) -> &str {
        match self {
            Self::Development => "dev",
            Self::Testing => "test",
            Self::Staging => "staging",
            Self::Production => "prod",
        }
    }
}

impl FromStr for Environment {
    type Err = EnvironmentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "development" | "dev" => Ok(Self::Development),
            "testing" | "test" => Ok(Self::Testing),
            "staging" => Ok(Self::Staging),
            "production" | "prod" => Ok(Self::Production),
            _ => Err(EnvironmentError::InvalidValue {
                variable: "MCP_ENV".to_string(),
                value: s.to_string(),
            }),
        }
    }
}

/// Network configuration from environment variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Bind address for the MCP server (e.g. `127.0.0.1` or `0.0.0.0`).
    pub host: String,
    /// Port to listen on.
    pub port: u16,
    /// Allowed CORS origins for cross-origin requests.
    pub cors_origins: Vec<String>,
    /// Request timeout in milliseconds.
    pub request_timeout_ms: u64,
    /// Maximum concurrent connections.
    pub max_connections: u32,
}

impl NetworkConfig {
    /// Load network configuration from environment variables
    ///
    /// # Errors
    /// Returns `EnvironmentError` if required environment variables have invalid values.
    pub fn from_env() -> Result<Self, EnvironmentError> {
        // Network configuration with environment-aware defaults
        let host = env::var("MCP_HOST").unwrap_or_else(|_| {
            if env::var("MCP_ENVIRONMENT").unwrap_or_default() == "production" {
                BIND_ALL_INTERFACES.to_string() // Bind to all interfaces in production
            } else {
                LOCALHOST_IPV4.to_string() // Localhost for development
            }
        });

        let port = config_helpers::get_port("MCP_PORT", ports::api_gateway());

        // Web UI configuration with environment awareness
        // Production: Resolved at runtime via ecosystem registry capability discovery.
        // Use WEB_UI_URL env var to override; otherwise discovered via ecosystem UI capability.
        let _web_ui_url = env::var("WEB_UI_URL").unwrap_or_else(|_| {
            if env::var("MCP_ENVIRONMENT").unwrap_or_default() == "production" {
                format!("discovered://{}", capabilities::ECOSYSTEM_CAPABILITY)
            } else {
                // Multi-tier dev UI resolution
                let port = env::var("WEB_UI_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or_else(ports::biomeos_ui);
                format!("http://localhost:{port}")
            }
        });

        let cors_origins = env::var("MCP_CORS_ORIGINS")
            .unwrap_or_else(|_| {
                // Multi-tier CORS origins resolution
                let port = env::var("WEB_UI_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or_else(ports::biomeos_ui);
                format!("http://localhost:{port}")
            })
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let request_timeout_ms = env::var("MCP_REQUEST_TIMEOUT_MS")
            .unwrap_or_else(|_| "30000".to_string())
            .parse::<u64>()
            .map_err(|e| EnvironmentError::ParseError {
                variable: "MCP_REQUEST_TIMEOUT_MS".to_string(),
                error: e.to_string(),
            })?;

        let max_connections = env::var("MCP_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u32>()
            .map_err(|e| EnvironmentError::ParseError {
                variable: "MCP_MAX_CONNECTIONS".to_string(),
                error: e.to_string(),
            })?;

        Ok(Self {
            host,
            port,
            cors_origins,
            request_timeout_ms,
            max_connections,
        })
    }
}

/// Database configuration with environment variable support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection string (env: `DATABASE_URL`)
    pub connection_string: String,
    /// Maximum number of connections (env: `DATABASE_MAX_CONNECTIONS`)
    pub max_connections: u32,
    /// Connection timeout in seconds (env: `DATABASE_TIMEOUT`)
    pub timeout_seconds: u64,
}

impl DatabaseConfig {
    /// Load database configuration from environment variables
    ///
    /// # Errors
    /// Returns `EnvironmentError` if required environment variables have invalid values.
    pub fn from_env() -> Result<Self, EnvironmentError> {
        let env = Environment::from_env();
        Ok(env.get_database_config())
    }
}

impl Environment {
    /// Get database configuration with environment overrides
    #[must_use]
    pub fn get_database_config(&self) -> DatabaseConfig {
        let database_url = std::env::var("DATABASE_URL").map_or_else(
            |_| {
                match self {
                    Self::Production => {
                        warn!("FATAL SECURITY ERROR: DATABASE_URL environment variable is required in production");
                        warn!("Production deployment blocked to prevent security vulnerability");
                        warn!("Please set DATABASE_URL environment variable with secure credentials");
                        std::process::exit(1);
                    }
                    Self::Staging => {
                        warn!("DATABASE_URL not set in staging, using fallback");
                        std::env::var("DATABASE_URL_STAGING").unwrap_or_else(|_| {
                            warn!("ERROR: Neither DATABASE_URL nor DATABASE_URL_STAGING is set");
                            std::process::exit(1);
                        })
                    }
                    Self::Testing => "sqlite::memory:".to_string(),
                    Self::Development => std::env::var("DATABASE_URL_DEV")
                        .unwrap_or_else(|_| "sqlite::memory:".to_string()),
                }
            },
            |url| {
                if *self == Self::Production && url.contains("password") {
                    warn!("Production DATABASE_URL appears to contain hardcoded password");
                }
                url
            },
        );

        let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .unwrap_or(10);

        let timeout_seconds = env::var("DATABASE_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or_else(|| timeouts::DEFAULT_DATABASE_TIMEOUT.as_secs());

        DatabaseConfig {
            connection_string: database_url,
            max_connections,
            timeout_seconds,
        }
    }
}

/// AI Provider configuration from environment variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProviderConfig {
    /// Optional OpenAI API key (env: `OPENAI_API_KEY`).
    pub openai_api_key: Option<String>,
    /// OpenAI API base URL (env: `OPENAI_ENDPOINT`).
    pub openai_endpoint: String,
    /// Optional Anthropic API key (env: `ANTHROPIC_API_KEY`).
    pub anthropic_api_key: Option<String>,
    /// Anthropic API base URL (env: `ANTHROPIC_ENDPOINT`).
    pub anthropic_endpoint: String,
    /// Local AI server endpoint (agnostic: works with Ollama, llama.cpp, vLLM, etc.)
    pub local_server_endpoint: String,
    /// Default model name for AI requests (env: `MCP_DEFAULT_MODEL`).
    pub default_model: String,
    /// AI request timeout in milliseconds (env: `AI_REQUEST_TIMEOUT_MS`).
    pub request_timeout_ms: u64,
}

impl AIProviderConfig {
    /// Load AI provider configuration from environment variables
    ///
    /// # Errors
    /// Returns `EnvironmentError` if required environment variables have invalid values.
    pub fn from_env() -> Result<Self, EnvironmentError> {
        let openai_api_key = env::var("OPENAI_API_KEY").ok();
        let openai_endpoint =
            env::var("OPENAI_ENDPOINT").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        let anthropic_api_key = env::var("ANTHROPIC_API_KEY").ok();
        let anthropic_endpoint = env::var("ANTHROPIC_ENDPOINT")
            .unwrap_or_else(|_| "https://api.anthropic.com/v1".to_string());

        // Multi-tier local AI server endpoint resolution (vendor-agnostic)
        // 1. LOCAL_AI_ENDPOINT (agnostic)
        // 2. OLLAMA_ENDPOINT (backward compat)
        // 3. Port override via LOCAL_AI_PORT / OLLAMA_PORT
        // 4. Default: http://localhost:11434
        let local_server_endpoint = env::var("LOCAL_AI_ENDPOINT")
            .or_else(|_| env::var("OLLAMA_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = env::var("LOCAL_AI_PORT")
                    .or_else(|_| env::var("OLLAMA_PORT"))
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or_else(ports::ollama);
                format!("http://localhost:{port}")
            });

        let default_model =
            env::var("MCP_DEFAULT_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());

        let request_timeout_ms = env::var("AI_REQUEST_TIMEOUT_MS")
            .unwrap_or_else(|_| "30000".to_string())
            .parse::<u64>()
            .map_err(|e| EnvironmentError::ParseError {
                variable: "AI_REQUEST_TIMEOUT_MS".to_string(),
                error: e.to_string(),
            })?;

        Ok(Self {
            openai_api_key,
            openai_endpoint,
            anthropic_api_key,
            anthropic_endpoint,
            local_server_endpoint,
            default_model,
            request_timeout_ms,
        })
    }
}

/// Ecosystem service configuration from environment variables
///
/// Uses capability-domain env vars as primary with primal-specific names as
/// legacy fallbacks (wateringHole `PRIMAL_SELF_KNOWLEDGE_STANDARD` §4).
/// Production should use capability-based discovery; env vars override for explicit config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemConfig {
    /// Storage capability endpoint (env: `STORAGE_ENDPOINT` > `NESTGATE_ENDPOINT`)
    #[serde(alias = "nestgate_endpoint")]
    pub storage_endpoint: String,
    /// Security capability endpoint (env: `SECURITY_ENDPOINT` > `BEARDOG_ENDPOINT`)
    #[serde(alias = "beardog_endpoint")]
    pub security_endpoint: String,
    /// Compute capability endpoint (env: `COMPUTE_ENDPOINT` > `TOADSTOOL_ENDPOINT`)
    #[serde(alias = "toadstool_endpoint")]
    pub compute_endpoint: String,
    /// Ecosystem registry / service mesh endpoint (env: `SERVICE_MESH_ENDPOINT` > `BIOMEOS_ENDPOINT`)
    pub service_mesh_endpoint: String,
    /// Timeout in milliseconds for ecosystem service calls.
    pub service_timeout_ms: u64,
}

impl Default for EcosystemConfig {
    fn default() -> Self {
        // Multi-tier ecosystem endpoint defaults with port-only overrides
        let storage_endpoint = std::env::var("STORAGE_ENDPOINT")
            .or_else(|_| std::env::var("NESTGATE_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = std::env::var("STORAGE_PORT")
                    .or_else(|_| std::env::var("NESTGATE_PORT"))
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or_else(|| {
                        config_helpers::get_port("STORAGE_SERVICE_PORT", ports::storage_service())
                    });
                format!("http://localhost:{port}")
            });

        let security_endpoint = std::env::var("SECURITY_ENDPOINT")
            .or_else(|_| std::env::var("BEARDOG_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = std::env::var("SECURITY_PORT")
                    .or_else(|_| std::env::var("SECURITY_AUTHENTICATION_PORT"))
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or_else(|| {
                        config_helpers::get_port("SECURITY_SERVICE_PORT", ports::security_service())
                    });
                format!("http://localhost:{port}")
            });

        let compute_endpoint = std::env::var("COMPUTE_ENDPOINT")
            .or_else(|_| std::env::var("TOADSTOOL_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = std::env::var("COMPUTE_PORT")
                    .or_else(|_| std::env::var("TOADSTOOL_PORT"))
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or_else(|| {
                        config_helpers::get_port("COMPUTE_SERVICE_PORT", ports::compute_service())
                    });
                format!("http://localhost:{port}")
            });

        let service_mesh_endpoint = std::env::var("SERVICE_MESH_ENDPOINT")
            .or_else(|_| std::env::var("BIOMEOS_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = std::env::var("BIOMEOS_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or_else(|| {
                        config_helpers::get_port("SERVICE_MESH_PORT", ports::service_mesh())
                    });
                format!("http://localhost:{port}")
            });

        Self {
            storage_endpoint,
            security_endpoint,
            compute_endpoint,
            service_mesh_endpoint,
            service_timeout_ms: 5000,
        }
    }
}

impl EcosystemConfig {
    /// Load ecosystem configuration from environment variables
    ///
    /// # Errors
    /// Returns `EnvironmentError` if required environment variables have invalid values.
    pub fn from_env() -> Result<Self, EnvironmentError> {
        // Capability-based endpoint resolution (TRUE PRIMAL pattern).
        // Production defaults use discovered://{capability} - actual endpoints are resolved
        // at runtime via ecosystem registry capability discovery.
        // Env vars (NESTGATE_ENDPOINT, etc.) are ecosystem role endpoints - override for explicit config.
        let storage_endpoint = env::var("STORAGE_ENDPOINT")
            .or_else(|_| env::var("NESTGATE_ENDPOINT"))
            .unwrap_or_else(|_| {
                if env::var("MCP_ENVIRONMENT").unwrap_or_default() == "production" {
                    format!("discovered://{}", capabilities::STORAGE_CAPABILITY)
                } else {
                    let port = env::var("STORAGE_PORT")
                        .or_else(|_| env::var("NESTGATE_PORT"))
                        .ok()
                        .and_then(|p| p.parse::<u16>().ok())
                        .unwrap_or_else(|| {
                            config_helpers::get_port(
                                "STORAGE_SERVICE_PORT",
                                ports::storage_service(),
                            )
                        });
                    format!("http://localhost:{port}")
                }
            });

        let security_endpoint = env::var("SECURITY_ENDPOINT")
            .or_else(|_| env::var("BEARDOG_ENDPOINT"))
            .unwrap_or_else(|_| {
                if env::var("MCP_ENVIRONMENT").unwrap_or_default() == "production" {
                    format!("discovered://{}", capabilities::SECURITY_CAPABILITY)
                } else {
                    let port = env::var("SECURITY_PORT")
                        .or_else(|_| env::var("SECURITY_AUTHENTICATION_PORT"))
                        .ok()
                        .and_then(|p| p.parse::<u16>().ok())
                        .unwrap_or_else(|| {
                            config_helpers::get_port(
                                "SECURITY_SERVICE_PORT",
                                ports::security_service(),
                            )
                        });
                    format!("http://localhost:{port}")
                }
            });

        let compute_endpoint = env::var("COMPUTE_ENDPOINT")
            .or_else(|_| env::var("TOADSTOOL_ENDPOINT"))
            .unwrap_or_else(|_| {
                if env::var("MCP_ENVIRONMENT").unwrap_or_default() == "production" {
                    format!("discovered://{}", capabilities::COMPUTE_CAPABILITY)
                } else {
                    let port = env::var("COMPUTE_PORT")
                        .or_else(|_| env::var("TOADSTOOL_PORT"))
                        .ok()
                        .and_then(|p| p.parse::<u16>().ok())
                        .unwrap_or_else(|| {
                            config_helpers::get_port(
                                "COMPUTE_SERVICE_PORT",
                                ports::compute_service(),
                            )
                        });
                    format!("http://localhost:{port}")
                }
            });

        let biomeos_endpoint = env::var("BIOMEOS_ENDPOINT")
            .or_else(|_| env::var("SERVICE_MESH_ENDPOINT"))
            .unwrap_or_else(|_| {
                if env::var("MCP_ENVIRONMENT").unwrap_or_default() == "production" {
                    format!("discovered://{}", capabilities::SERVICE_MESH_CAPABILITY)
                } else {
                    let port = env::var("BIOMEOS_PORT")
                        .ok()
                        .and_then(|p| p.parse::<u16>().ok())
                        .unwrap_or_else(|| {
                            config_helpers::get_port("SERVICE_MESH_PORT", ports::service_mesh())
                        });
                    format!("http://localhost:{port}")
                }
            });

        let service_timeout_ms = env::var("ECOSYSTEM_SERVICE_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or_else(|| {
                // 10_000 ms fits in u64; allow for const conversion
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "DEFAULT_OPERATION_TIMEOUT.as_millis() fits u64"
                )]
                {
                    timeouts::DEFAULT_OPERATION_TIMEOUT.as_millis() as u64
                }
            });

        Ok(Self {
            storage_endpoint,
            security_endpoint,
            compute_endpoint,
            service_mesh_endpoint: biomeos_endpoint,
            service_timeout_ms,
        })
    }
}

/// Complete environment-based configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Current deployment environment (development, testing, staging, production).
    pub environment: Environment,
    /// Network bind and CORS settings.
    pub network: NetworkConfig,
    /// Database connection and pool settings.
    pub database: DatabaseConfig,
    /// AI provider endpoints and API keys.
    pub ai_providers: AIProviderConfig,
    /// Ecosystem service endpoints (storage, security, compute, service mesh).
    pub ecosystem: EcosystemConfig,
}

impl EnvironmentConfig {
    /// Load complete configuration from environment variables
    ///
    /// # Errors
    /// Returns `EnvironmentError` if any required environment variables have invalid values.
    pub fn from_env() -> Result<Self, EnvironmentError> {
        let environment = Environment::from_env();
        let network = NetworkConfig::from_env()?;
        let database = DatabaseConfig::from_env()?;
        let ai_providers = AIProviderConfig::from_env()?;
        let ecosystem = EcosystemConfig::from_env()?;

        Ok(Self {
            environment,
            network,
            database,
            ai_providers,
            ecosystem,
        })
    }

    /// Load configuration with validation
    ///
    /// # Errors
    /// Returns `EnvironmentError` if configuration loading or validation fails.
    pub fn load_and_validate() -> Result<Self, EnvironmentError> {
        let config = Self::from_env()?;
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration values
    ///
    /// # Errors
    /// Returns `EnvironmentError` if any configuration value is invalid.
    pub fn validate(&self) -> Result<(), EnvironmentError> {
        // Validate port range
        if self.network.port == 0 {
            return Err(EnvironmentError::InvalidValue {
                variable: "MCP_PORT".to_string(),
                value: self.network.port.to_string(),
            });
        }

        // Validate timeout values
        if self.network.request_timeout_ms == 0 {
            return Err(EnvironmentError::InvalidValue {
                variable: "MCP_REQUEST_TIMEOUT_MS".to_string(),
                value: self.network.request_timeout_ms.to_string(),
            });
        }

        // Validate database configuration
        if self.database.connection_string.is_empty() {
            return Err(EnvironmentError::InvalidValue {
                variable: "DATABASE_URL".to_string(),
                value: "empty".to_string(),
            });
        }

        // Validate AI provider endpoints
        if self.ai_providers.openai_endpoint.is_empty() {
            return Err(EnvironmentError::InvalidValue {
                variable: "OPENAI_ENDPOINT".to_string(),
                value: "empty".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "environment_tests.rs"]
mod tests;
