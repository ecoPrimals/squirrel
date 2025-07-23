use serde::{Deserialize, Serialize};
use std::env;
use std::str::FromStr;
use thiserror::Error;

/// Environment configuration errors
#[derive(Debug, Error)]
pub enum EnvironmentError {
    #[error("Missing required environment variable: {0}")]
    MissingVariable(String),

    #[error("Invalid environment variable value: {variable} = {value}")]
    InvalidValue { variable: String, value: String },

    #[error("Parse error for variable {variable}: {error}")]
    ParseError { variable: String, error: String },
}

/// Application environment types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

impl Environment {
    /// Get environment from MCP_ENV variable
    pub fn from_env() -> Self {
        match env::var("MCP_ENV")
            .unwrap_or_else(|_| "development".to_string())
            .as_str()
        {
            "production" => Environment::Production,
            "staging" => Environment::Staging,
            "testing" => Environment::Testing,
            _ => Environment::Development,
        }
    }

    /// Check if running in development mode
    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Development)
    }

    /// Check if running in production mode
    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }

    /// Get configuration file suffix
    pub fn config_suffix(&self) -> &str {
        match self {
            Environment::Development => "dev",
            Environment::Testing => "test",
            Environment::Staging => "staging",
            Environment::Production => "prod",
        }
    }
}

impl FromStr for Environment {
    type Err = EnvironmentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "development" | "dev" => Ok(Environment::Development),
            "testing" | "test" => Ok(Environment::Testing),
            "staging" => Ok(Environment::Staging),
            "production" | "prod" => Ok(Environment::Production),
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
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub request_timeout_ms: u64,
    pub max_connections: u32,
}

impl NetworkConfig {
    /// Load network configuration from environment variables
    pub fn from_env() -> Result<Self, EnvironmentError> {
        // Network configuration with environment-aware defaults
        let host = env::var("MCP_HOST").unwrap_or_else(|_| {
            if env::var("MCP_ENVIRONMENT").unwrap_or_default() == "production" {
                "0.0.0.0".to_string() // Bind to all interfaces in production
            } else {
                "127.0.0.1".to_string() // Localhost for development
            }
        });

        let port = env::var("MCP_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap_or(8080);

        // Web UI configuration with environment awareness
        let _web_ui_url = env::var("WEB_UI_URL").unwrap_or_else(|_| {
            if env::var("MCP_ENVIRONMENT").unwrap_or_default() == "production" {
                "http://biomeos-ui:3000".to_string() // Production service name
            } else {
                "http://localhost:3000".to_string() // Development fallback
            }
        });

        let cors_origins = env::var("MCP_CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000".to_string())
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

        Ok(NetworkConfig {
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
    /// Database connection string (env: DATABASE_URL)
    pub connection_string: String,
    /// Maximum number of connections (env: DATABASE_MAX_CONNECTIONS)
    pub max_connections: u32,
    /// Connection timeout in seconds (env: DATABASE_TIMEOUT)
    pub timeout_seconds: u64,
}

impl DatabaseConfig {
    /// Load database configuration from environment variables
    pub fn from_env() -> Result<Self, EnvironmentError> {
        let env = Environment::from_env();
        Ok(env.get_database_config())
    }
}

impl Environment {
    /// Get database configuration with environment overrides
    pub fn get_database_config(&self) -> DatabaseConfig {
        let database_url = match std::env::var("DATABASE_URL") {
            Ok(url) => {
                // Validate URL format for security
                if *self == Environment::Production && url.contains("password") {
                    eprintln!("⚠️  WARNING: Production DATABASE_URL appears to contain hardcoded password");
                }
                url
            }
            Err(_) => {
                match self {
                    Environment::Production => {
                        eprintln!("🚨 FATAL SECURITY ERROR: DATABASE_URL environment variable is required in production");
                        eprintln!(
                            "   Production deployment blocked to prevent security vulnerability"
                        );
                        eprintln!("   Please set DATABASE_URL environment variable with secure credentials");
                        std::process::exit(1);
                    }
                    Environment::Staging => {
                        eprintln!("⚠️  WARNING: DATABASE_URL not set in staging, using fallback");
                        // Use environment variable or fail
                        match std::env::var("DATABASE_URL_STAGING") {
                            Ok(url) => url,
                            Err(_) => {
                                eprintln!("🚨 ERROR: Neither DATABASE_URL nor DATABASE_URL_STAGING is set");
                                std::process::exit(1);
                            }
                        }
                    }
                    Environment::Testing => "sqlite::memory:".to_string(),
                    Environment::Development => {
                        // Allow fallback for development
                        std::env::var("DATABASE_URL_DEV")
                            .unwrap_or_else(|_| "sqlite::memory:".to_string())
                    }
                }
            }
        };

        let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .unwrap_or(10);

        let timeout_seconds = crate::constants::env_helpers::get_database_timeout();

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
    pub openai_api_key: Option<String>,
    pub openai_endpoint: String,
    pub anthropic_api_key: Option<String>,
    pub anthropic_endpoint: String,
    pub ollama_endpoint: String,
    pub default_model: String,
    pub request_timeout_ms: u64,
}

impl AIProviderConfig {
    /// Load AI provider configuration from environment variables
    pub fn from_env() -> Result<Self, EnvironmentError> {
        let openai_api_key = env::var("OPENAI_API_KEY").ok();
        let openai_endpoint =
            env::var("OPENAI_ENDPOINT").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        let anthropic_api_key = env::var("ANTHROPIC_API_KEY").ok();
        let anthropic_endpoint = env::var("ANTHROPIC_ENDPOINT")
            .unwrap_or_else(|_| "https://api.anthropic.com/v1".to_string());

        let ollama_endpoint =
            env::var("OLLAMA_ENDPOINT").unwrap_or_else(|_| "http://localhost:11434".to_string());

        let default_model =
            env::var("MCP_DEFAULT_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());

        let request_timeout_ms = env::var("AI_REQUEST_TIMEOUT_MS")
            .unwrap_or_else(|_| "30000".to_string())
            .parse::<u64>()
            .map_err(|e| EnvironmentError::ParseError {
                variable: "AI_REQUEST_TIMEOUT_MS".to_string(),
                error: e.to_string(),
            })?;

        Ok(AIProviderConfig {
            openai_api_key,
            openai_endpoint,
            anthropic_api_key,
            anthropic_endpoint,
            ollama_endpoint,
            default_model,
            request_timeout_ms,
        })
    }
}

/// Ecosystem service configuration from environment variables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemConfig {
    pub nestgate_endpoint: String,
    pub beardog_endpoint: String,
    pub toadstool_endpoint: String,
    pub songbird_endpoint: String,
    pub service_timeout_ms: u64,
}

impl EcosystemConfig {
    /// Load ecosystem configuration from environment variables
    pub fn from_env() -> Result<Self, EnvironmentError> {
        // Primal endpoints with service discovery
        let nestgate_endpoint = env::var("NESTGATE_ENDPOINT").unwrap_or_else(|_| {
            if env::var("MCP_ENVIRONMENT").unwrap_or_default() == "production" {
                "http://nestgate:8444".to_string()
            } else {
                "http://localhost:8444".to_string()
            }
        });

        let beardog_endpoint = env::var("BEARDOG_ENDPOINT").unwrap_or_else(|_| {
            if env::var("MCP_ENVIRONMENT").unwrap_or_default() == "production" {
                "http://beardog:8443".to_string()
            } else {
                "http://localhost:8443".to_string()
            }
        });

        let toadstool_endpoint = env::var("TOADSTOOL_ENDPOINT").unwrap_or_else(|_| {
            if env::var("MCP_ENVIRONMENT").unwrap_or_default() == "production" {
                "http://toadstool:8445".to_string()
            } else {
                "http://localhost:8445".to_string()
            }
        });

        let biomeos_endpoint = env::var("BIOMEOS_ENDPOINT").unwrap_or_else(|_| {
            if env::var("MCP_ENVIRONMENT").unwrap_or_default() == "production" {
                "http://biomeos:8446".to_string()
            } else {
                "http://localhost:8446".to_string()
            }
        });

        let service_timeout_ms = crate::constants::env_helpers::parse_timeout(
            "ECOSYSTEM_SERVICE_TIMEOUT_MS",
            crate::constants::timeouts::DEFAULT_OPERATION_TIMEOUT,
        );

        Ok(EcosystemConfig {
            nestgate_endpoint,
            beardog_endpoint,
            toadstool_endpoint,
            songbird_endpoint: biomeos_endpoint, // Assuming songbird_endpoint is the same as biomeos_endpoint
            service_timeout_ms,
        })
    }
}

/// Complete environment-based configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub environment: Environment,
    pub network: NetworkConfig,
    pub database: DatabaseConfig,
    pub ai_providers: AIProviderConfig,
    pub ecosystem: EcosystemConfig,
}

impl EnvironmentConfig {
    /// Load complete configuration from environment variables
    pub fn from_env() -> Result<Self, EnvironmentError> {
        let environment = Environment::from_env();
        let network = NetworkConfig::from_env()?;
        let database = DatabaseConfig::from_env()?;
        let ai_providers = AIProviderConfig::from_env()?;
        let ecosystem = EcosystemConfig::from_env()?;

        Ok(EnvironmentConfig {
            environment,
            network,
            database,
            ai_providers,
            ecosystem,
        })
    }

    /// Load configuration with validation
    pub fn load_and_validate() -> Result<Self, EnvironmentError> {
        let config = Self::from_env()?;
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration values
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
mod tests {
    use super::*;

    #[test]
    fn test_environment_from_string() {
        assert_eq!(
            Environment::from_str("development").unwrap(),
            Environment::Development
        );
        assert_eq!(
            Environment::from_str("production").unwrap(),
            Environment::Production
        );
        assert!(Environment::from_str("invalid").is_err());
    }

    #[test]
    fn test_environment_config_validation() {
        let mut config = EnvironmentConfig::from_env().unwrap();

        // Test invalid port
        config.network.port = 0;
        assert!(config.validate().is_err());

        // Test invalid timeout
        config.network.port = 8080;
        config.network.request_timeout_ms = 0;
        assert!(config.validate().is_err());
    }
}
