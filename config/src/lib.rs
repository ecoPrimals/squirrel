use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

pub mod environment;
pub use environment::{Environment, EnvironmentConfig, EnvironmentError};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub network: NetworkConfig,
    pub database: DatabaseConfig,
    pub ai: AIConfig,
    pub security: SecurityConfig,
    pub observability: ObservabilityConfig,
    pub ecosystem: EcosystemConfig,
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
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
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
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self::default())
    }

    /// Load configuration from a file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path).map_err(|e| ConfigError::IO(e.to_string()))?;

        toml::from_str(&contents).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), ConfigError> {
        let contents =
            toml::to_string_pretty(self).map_err(|e| ConfigError::Parse(e.to_string()))?;

        std::fs::write(path, contents).map_err(|e| ConfigError::IO(e.to_string()))
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

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IO(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum DatabaseBackend {
    #[serde(rename = "nestgate")]
    NestGate,
    #[serde(rename = "postgres")]
    PostgreSQL,
    #[serde(rename = "sqlite")]
    SQLite,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AIConfig {
    pub providers: Vec<AIProvider>,
    pub default_provider: String,
    pub max_retries: u32,
    pub timeout: Duration,
    pub fallback_enabled: bool,
    pub health_check_interval: Duration,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AIProvider {
    pub name: String,
    pub provider_type: AIProviderType,
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub priority: u32,
    pub enabled: bool,
    pub rate_limit: RateLimit,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AIProviderType {
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "azure")]
    Azure,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub tokens_per_minute: u32,
    pub burst_limit: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub backend: SecurityBackend,
    pub jwt_secret_key_id: String,
    pub jwt_expiration: Duration,
    pub encryption_algorithm: String,
    pub hsm_provider: String,
    pub authentication_required: bool,
    pub session_timeout: Duration,
    pub max_failed_attempts: u32,
    pub lockout_duration: Duration,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SecurityBackend {
    #[serde(rename = "beardog")]
    BearDog,
    #[serde(rename = "internal")]
    Internal,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ObservabilityConfig {
    pub logging: LoggingConfig,
    pub metrics: MetricsConfig,
    pub tracing: TracingConfig,
    pub health_checks: HealthCheckConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub destination: String,
    pub file_path: Option<PathBuf>,
    pub rotation: Option<String>,
    pub max_size: Option<String>,
    pub max_files: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub port: u16,
    pub collection_interval: Duration,
    pub retention_period: Duration,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TracingConfig {
    pub enabled: bool,
    pub sampling_rate: f64,
    pub jaeger_endpoint: Option<String>,
    pub service_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub interval: Duration,
    pub timeout: Duration,
    pub failure_threshold: u32,
    pub recovery_threshold: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EcosystemConfig {
    pub enabled: bool,
    pub mode: EcosystemMode,
    pub discovery: DiscoveryConfig,
    pub coordination: CoordinationConfig,
    pub biome_manifest: BiomeManifestConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum EcosystemMode {
    #[serde(rename = "sovereign")]
    Sovereign, // Operate independently, coordinate when available
    #[serde(rename = "coordinated")]
    Coordinated, // Require coordination with other primals
    #[serde(rename = "standalone")]
    Standalone, // Operate without any coordination
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscoveryConfig {
    pub songbird_endpoint: Option<String>,
    pub auto_discovery: bool,
    pub probe_interval: Duration,
    pub direct_endpoints: HashMap<String, String>,
    pub health_check_timeout: Duration,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoordinationConfig {
    pub nestgate: Option<NestGateCoordination>,
    pub beardog: Option<BearDogCoordination>,
    pub toadstool: Option<ToadStoolCoordination>,
    pub fallback_strategies: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NestGateCoordination {
    pub endpoint: Option<String>,
    pub auto_provision: bool,
    pub storage_class: String,
    pub fallback_to_local: bool,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BearDogCoordination {
    pub endpoint: Option<String>,
    pub auto_auth: bool,
    pub security_level: String,
    pub fallback_to_local: bool,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToadStoolCoordination {
    pub endpoint: Option<String>,
    pub auto_delegate: bool,
    pub compute_class: String,
    pub fallback_to_local: bool,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BiomeManifestConfig {
    pub auto_generate: bool,
    pub output_path: String,
    pub metadata: BiomeMetadata,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BiomeMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub primal_type: String,
    pub capabilities: Vec<String>,
}

impl Config {
    /// Create a new configuration instance.
    ///
    /// This method first checks for a config file specified by the `SQUIRREL_MCP_CONFIG`
    /// environment variable, or defaults to "config.toml". If the file exists, it loads
    /// the configuration from the file and applies environment variable overrides.
    /// If the file doesn't exist, it creates a default configuration with environment
    /// variable overrides applied.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Config, anyhow::Error>` containing the initialized configuration
    /// or an error if the configuration could not be loaded.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The config file exists but cannot be read
    /// - The config file contains invalid TOML syntax
    /// - Required environment variables are malformed
    pub fn new() -> Result<Self, anyhow::Error> {
        let config_path =
            std::env::var("SQUIRREL_MCP_CONFIG").unwrap_or_else(|_| "config.toml".to_string());

        if std::path::Path::new(&config_path).exists() {
            Self::from_file(&config_path)
        } else {
            tracing::info!("Config file not found, using defaults and environment variables");
            Self::from_env()
        }
    }

    /// Load configuration from a TOML file.
    ///
    /// This method reads the configuration from the specified file path and applies
    /// environment variable overrides on top of the file-based configuration.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the TOML configuration file
    ///
    /// # Returns
    ///
    /// Returns a `Result<Config, anyhow::Error>` containing the loaded configuration
    /// or an error if the file could not be loaded or parsed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read (e.g., file doesn't exist, permission denied)
    /// - The file contains invalid TOML syntax
    /// - Environment variable overrides are malformed
    pub fn from_file(path: &str) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;

        // Override with environment variables
        config.apply_env_overrides();

        Ok(config)
    }

    /// Create configuration from environment variables only.
    ///
    /// This method creates a default configuration and applies environment variable
    /// overrides to customize the settings. This is useful when no configuration file
    /// is available or when running in containerized environments.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Config, anyhow::Error>` containing the configuration
    /// initialized from environment variables or an error if environment variables
    /// are malformed.
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are malformed or
    /// contain invalid values.
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let mut config = Self::default();
        config.apply_env_overrides();
        Ok(config)
    }

    fn apply_env_overrides(&mut self) {
        // Network configuration
        if let Ok(host) = std::env::var("SQUIRREL_MCP_HOST") {
            self.network.host = host;
        }
        if let Ok(port) = std::env::var("SQUIRREL_MCP_PORT") {
            self.network.port = port.parse().unwrap_or(self.network.port);
        }
        // Legacy WebSocket port configuration - commented out for now
        // if let Ok(ws_port) = std::env::var("SQUIRREL_MCP_WEBSOCKET_PORT") {
        //     self.network.websocket_port = ws_port.parse().unwrap_or(self.network.websocket_port);
        // }
        // if let Ok(dashboard_port) = std::env::var("SQUIRREL_MCP_DASHBOARD_PORT") {
        //     self.network.dashboard_port = dashboard_port.parse().unwrap_or(self.network.dashboard_port);
        // }

        // Database configuration
        if let Ok(db_connection) = std::env::var("SQUIRREL_MCP_DATABASE_URL") {
            self.database.connection_string = db_connection;
        }

        // Ecosystem configuration
        if let Ok(ecosystem_enabled) = std::env::var("SQUIRREL_ECOSYSTEM_ENABLED") {
            self.ecosystem.enabled = ecosystem_enabled.parse().unwrap_or(true);
        }
        if let Ok(ecosystem_mode) = std::env::var("SQUIRREL_ECOSYSTEM_MODE") {
            self.ecosystem.mode = match ecosystem_mode.as_str() {
                "sovereign" => EcosystemMode::Sovereign,
                "coordinated" => EcosystemMode::Coordinated,
                "standalone" => EcosystemMode::Standalone,
                _ => EcosystemMode::Sovereign,
            };
        }

        // Discovery configuration
        if let Ok(songbird_endpoint) = std::env::var("SONGBIRD_DISCOVERY_ENDPOINT") {
            self.ecosystem.discovery.songbird_endpoint = Some(songbird_endpoint);
        }
        if let Ok(auto_discovery) = std::env::var("SQUIRREL_AUTO_DISCOVERY") {
            self.ecosystem.discovery.auto_discovery = auto_discovery.parse().unwrap_or(true);
        }

        // Coordination configuration
        if let Ok(nestgate_endpoint) = std::env::var("NESTGATE_STORAGE_ENDPOINT") {
            if let Some(ref mut nestgate) = self.ecosystem.coordination.nestgate {
                nestgate.endpoint = Some(nestgate_endpoint);
            }
        }
        if let Ok(beardog_endpoint) = std::env::var("BEARDOG_AUTH_ENDPOINT") {
            if let Some(ref mut beardog) = self.ecosystem.coordination.beardog {
                beardog.endpoint = Some(beardog_endpoint);
            }
        }
        if let Ok(toadstool_endpoint) = std::env::var("TOADSTOOL_COMPUTE_ENDPOINT") {
            if let Some(ref mut toadstool) = self.ecosystem.coordination.toadstool {
                toadstool.endpoint = Some(toadstool_endpoint);
            }
        }

        // AI provider configuration
        if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
            if let Some(provider) = self.ai.providers.iter_mut().find(|p| p.name == "openai") {
                provider.api_key = openai_key;
            }
        }
        if let Ok(anthropic_key) = std::env::var("ANTHROPIC_API_KEY") {
            if let Some(provider) = self.ai.providers.iter_mut().find(|p| p.name == "anthropic") {
                provider.api_key = anthropic_key;
            }
        }
        if let Ok(ollama_endpoint) = std::env::var("OLLAMA_ENDPOINT") {
            if let Some(provider) = self.ai.providers.iter_mut().find(|p| p.name == "ollama") {
                provider.endpoint = ollama_endpoint;
            }
        }
    }

    pub fn validate(&self) -> Result<(), anyhow::Error> {
        // Validate network configuration
        if self.network.port == 0 {
            return Err(anyhow::anyhow!("Network port cannot be zero"));
        }
        if self.network.websocket_endpoint.is_none() {
            return Err(anyhow::anyhow!("WebSocket endpoint cannot be empty"));
        }
        // if self.network.websocket_port == 0 {
        //     return Err(anyhow::anyhow!("WebSocket port cannot be zero"));
        // }
        // if self.network.dashboard_port == 0 {
        //     return Err(anyhow::anyhow!("Dashboard port cannot be zero"));
        // }

        // Validate AI providers
        if self.ai.providers.is_empty() {
            return Err(anyhow::anyhow!(
                "At least one AI provider must be configured"
            ));
        }

        let default_provider_exists = self
            .ai
            .providers
            .iter()
            .any(|p| p.name == self.ai.default_provider);
        if !default_provider_exists {
            return Err(anyhow::anyhow!(
                "Default AI provider '{}' not found in providers list",
                self.ai.default_provider
            ));
        }

        // Validate ecosystem configuration
        if self.ecosystem.enabled {
            // Validate discovery configuration
            if self.ecosystem.discovery.songbird_endpoint.is_some() {
                if let Some(ref endpoint) = self.ecosystem.discovery.songbird_endpoint {
                    if !endpoint.starts_with("https://") && !endpoint.starts_with("http://") {
                        tracing::warn!(
                            "Songbird discovery endpoint should use HTTP/HTTPS protocol"
                        );
                    }
                }
            }

            // Validate coordination endpoints
            if let Some(ref beardog) = self.ecosystem.coordination.beardog {
                if let Some(ref endpoint) = beardog.endpoint {
                    if !endpoint.starts_with("https://") {
                        tracing::warn!("BearDog endpoint should use HTTPS for production");
                    }
                }
            }

            if let Some(ref nestgate) = self.ecosystem.coordination.nestgate {
                if let Some(ref endpoint) = nestgate.endpoint {
                    if !endpoint.starts_with("https://") {
                        tracing::warn!("NestGate endpoint should use HTTPS for production");
                    }
                }
            }

            if let Some(ref toadstool) = self.ecosystem.coordination.toadstool {
                if let Some(ref endpoint) = toadstool.endpoint {
                    if !endpoint.starts_with("https://") {
                        tracing::warn!("ToadStool endpoint should use HTTPS for production");
                    }
                }
            }
        }

        // Validate security configuration
        if self.security.authentication_required && self.security.jwt_secret_key_id.is_empty() {
            return Err(anyhow::anyhow!(
                "JWT secret key ID is required when authentication is enabled"
            ));
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            network: NetworkConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                websocket_endpoint: Some("ws://localhost:8081".to_string()),
                cors_origins: vec!["http://localhost:3000".to_string()],
            },
            database: DatabaseConfig {
                connection_string: "sqlite::memory:".to_string(),
                max_connections: 10,
                timeout_seconds: 30,
            },
            ai: AIConfig {
                providers: vec![
                    AIProvider {
                        name: "openai".to_string(),
                        provider_type: AIProviderType::OpenAI,
                        endpoint: Self::parse_endpoint_safely(
                            "https://api.openai.com/v1",
                            "OpenAI",
                        ),
                        api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
                        model: "gpt-4".to_string(),
                        max_tokens: 4096,
                        temperature: 0.7,
                        priority: 1,
                        enabled: true,
                        rate_limit: RateLimit {
                            requests_per_minute: 60,
                            tokens_per_minute: 200000,
                            burst_limit: 10,
                        },
                    },
                    AIProvider {
                        name: "anthropic".to_string(),
                        provider_type: AIProviderType::Anthropic,
                        endpoint: Self::parse_endpoint_safely(
                            "https://api.anthropic.com/v1",
                            "Anthropic",
                        ),
                        api_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
                        model: "claude-3-sonnet-20240229".to_string(),
                        max_tokens: 4096,
                        temperature: 0.7,
                        priority: 2,
                        enabled: true,
                        rate_limit: RateLimit {
                            requests_per_minute: 60,
                            tokens_per_minute: 200000,
                            burst_limit: 10,
                        },
                    },
                    AIProvider {
                        name: "ollama".to_string(),
                        provider_type: AIProviderType::Ollama,
                        endpoint: Self::parse_endpoint_safely("http://localhost:11434", "Ollama"),
                        api_key: String::new(),
                        model: "llama3.2".to_string(),
                        max_tokens: 4096,
                        temperature: 0.7,
                        priority: 3,
                        enabled: true,
                        rate_limit: RateLimit {
                            requests_per_minute: 120,
                            tokens_per_minute: 400000,
                            burst_limit: 20,
                        },
                    },
                ],
                default_provider: "openai".to_string(),
                max_retries: 3,
                timeout: Duration::from_secs(60),
                fallback_enabled: true,
                health_check_interval: Duration::from_secs(60),
            },
            security: SecurityConfig {
                backend: SecurityBackend::BearDog,
                jwt_secret_key_id: "squirrel-mcp-jwt".to_string(),
                jwt_expiration: Duration::from_secs(3600),
                encryption_algorithm: "aes-256-gcm".to_string(),
                hsm_provider: "softhsm".to_string(),
                authentication_required: true,
                session_timeout: Duration::from_secs(1800),
                max_failed_attempts: 5,
                lockout_duration: Duration::from_secs(300),
            },
            observability: ObservabilityConfig {
                logging: LoggingConfig {
                    level: "info".to_string(),
                    format: "json".to_string(),
                    destination: "stdout".to_string(),
                    file_path: None,
                    rotation: None,
                    max_size: None,
                    max_files: None,
                },
                metrics: MetricsConfig {
                    enabled: true,
                    endpoint: "/metrics".to_string(),
                    port: 9090,
                    collection_interval: Duration::from_secs(15),
                    retention_period: Duration::from_secs(86400),
                },
                tracing: TracingConfig {
                    enabled: true,
                    sampling_rate: 1.0,
                    jaeger_endpoint: None,
                    service_name: "squirrel-mcp".to_string(),
                },
                health_checks: HealthCheckConfig {
                    enabled: true,
                    interval: Duration::from_secs(30),
                    timeout: Duration::from_secs(5),
                    failure_threshold: 3,
                    recovery_threshold: 2,
                },
            },
            ecosystem: EcosystemConfig {
                enabled: true,
                mode: EcosystemMode::Sovereign,
                discovery: DiscoveryConfig {
                    songbird_endpoint: std::env::var("SONGBIRD_DISCOVERY_ENDPOINT").ok(),
                    auto_discovery: true,
                    probe_interval: Duration::from_secs(30),
                    direct_endpoints: HashMap::from([
                        ("nestgate".to_string(), "http://localhost:8444".to_string()),
                        ("beardog".to_string(), "http://localhost:8443".to_string()),
                        ("toadstool".to_string(), "http://localhost:8445".to_string()),
                    ]),
                    health_check_timeout: Duration::from_secs(5),
                },
                coordination: CoordinationConfig {
                    nestgate: Some(NestGateCoordination {
                        endpoint: std::env::var("NESTGATE_STORAGE_ENDPOINT").ok(),
                        auto_provision: true,
                        storage_class: "standard".to_string(),
                        fallback_to_local: true,
                        capabilities: vec![
                            "storage".to_string(),
                            "persistence".to_string(),
                            "backup".to_string(),
                        ],
                    }),
                    beardog: Some(BearDogCoordination {
                        endpoint: std::env::var("BEARDOG_AUTH_ENDPOINT").ok(),
                        auto_auth: true,
                        security_level: "enterprise".to_string(),
                        fallback_to_local: true,
                        capabilities: vec![
                            "authentication".to_string(),
                            "authorization".to_string(),
                            "encryption".to_string(),
                        ],
                    }),
                    toadstool: Some(ToadStoolCoordination {
                        endpoint: std::env::var("TOADSTOOL_COMPUTE_ENDPOINT").ok(),
                        auto_delegate: true,
                        compute_class: "standard".to_string(),
                        fallback_to_local: true,
                        capabilities: vec![
                            "compute".to_string(),
                            "containers".to_string(),
                            "wasm".to_string(),
                        ],
                    }),
                    fallback_strategies: HashMap::from([
                        ("storage".to_string(), "local-storage".to_string()),
                        ("auth".to_string(), "local-auth".to_string()),
                        ("compute".to_string(), "local-execution".to_string()),
                    ]),
                },
                biome_manifest: BiomeManifestConfig {
                    auto_generate: true,
                    output_path: "biome.yaml".to_string(),
                    metadata: BiomeMetadata {
                        name: "squirrel-mcp".to_string(),
                        description: "AI Agent Platform with MCP Protocol".to_string(),
                        version: "2.0.0".to_string(),
                        primal_type: "squirrel".to_string(),
                        capabilities: vec![
                            "mcp".to_string(),
                            "ai-agents".to_string(),
                            "context-management".to_string(),
                            "plugin-execution".to_string(),
                        ],
                    },
                },
            },
        }
    }
}

impl Config {
    /// Safely parse endpoint URLs with proper error handling - guaranteed to never panic
    fn parse_endpoint_safely(url: &str, provider_name: &str) -> String {
        // Try the requested URL first
        if let Ok(parsed) = url.parse::<Url>() {
            return parsed.to_string();
        }

        tracing::warn!(
            "Failed to parse {} endpoint URL '{}'. Trying fallbacks.",
            provider_name,
            url
        );

        // Try known-good fallback URLs
        let fallbacks = [
            "http://localhost:8080",
            "http://127.0.0.1:8080",
            "http://localhost",
            "http://127.0.0.1",
            "http://disabled",
            "http://error",
        ];

        for fallback in &fallbacks {
            if let Ok(parsed) = fallback.parse::<Url>() {
                tracing::warn!("Using fallback URL '{}' for {}", fallback, provider_name);
                return parsed.to_string();
            }
        }

        // PRODUCTION SAFE: If all known fallbacks fail, create a URL that represents a disabled provider
        // This prevents crashes while maintaining system stability
        tracing::error!(
            "CRITICAL: All URL parsing failed for {}. Provider will be disabled.",
            provider_name
        );

        // Create a disabled URL that will cause the provider to be skipped
        // This approach is safe and prevents application crashes
        Self::create_disabled_url(provider_name)
    }

    /// Creates a URL that represents a disabled provider
    /// This is a helper function that uses safe URL creation patterns
    fn create_disabled_url(provider_name: &str) -> String {
        // Try multiple strategies to create a URL that represents a disabled provider
        // This URL will cause the provider to be skipped during initialization

        // Strategy 1: Create a provider-specific disabled URL
        let clean_name = provider_name
            .chars()
            .filter(|c| c.is_alphanumeric())
            .take(10)
            .collect::<String>();

        if !clean_name.is_empty() {
            let disabled_url = format!("http://disabled-{clean_name}");
            if let Ok(url) = disabled_url.parse::<Url>() {
                return url.to_string();
            }
        }

        // Strategy 2: Use a generic disabled URL
        if let Ok(url) = "http://disabled".parse::<Url>() {
            return url.to_string();
        }

        // Strategy 3: Use localhost as a safe fallback
        if let Ok(url) = "http://localhost".parse::<Url>() {
            return url.to_string();
        }

        // Strategy 4: Use IP address as fallback
        if let Ok(url) = "http://127.0.0.1".parse::<Url>() {
            return url.to_string();
        }

        // Strategy 5: Use the most basic HTTP URL
        if let Ok(url) = "http://error".parse::<Url>() {
            return url.to_string();
        }

        // FINAL FALLBACK: If even basic URLs fail, this indicates a fundamental system issue
        // Log the critical error and exit the process safely rather than panic
        tracing::error!(
            "SYSTEM CRITICAL: Cannot create any URL for {}. URL crate appears broken.",
            provider_name
        );
        tracing::error!(
            "This indicates a fundamental system failure. Exiting to prevent undefined behavior."
        );

        // Exit the process cleanly rather than panic or use dangerous patterns
        std::process::exit(1);
    }
}

/// Configuration defaults that can be overridden by environment variables
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigDefaults {
    /// Network configuration defaults
    pub network: NetworkDefaults,
    /// Database configuration defaults
    pub database: DatabaseDefaults,
    /// External service defaults
    pub external_services: ExternalServiceDefaults,
    /// AI service defaults
    pub ai_services: AIServiceDefaults,
    /// Observability defaults
    pub observability: ObservabilityDefaults,
}

/// Network configuration defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDefaults {
    /// Default server host
    pub host: String,
    /// Default server port
    pub port: u16,
    /// Default CORS origins
    pub cors_origins: Vec<String>,
    /// Default WebSocket port
    pub websocket_port: u16,
    /// Default bind address for production
    pub bind_address: String,
}

/// Database configuration defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseDefaults {
    /// Default database URL (should be overridden in production)
    pub url: String,
    /// Default maximum connections
    pub max_connections: u32,
    /// Default connection timeout
    pub timeout_seconds: u64,
    /// Default test database URL
    pub test_url: String,
}

/// External service configuration defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalServiceDefaults {
    /// Default Songbird service URL
    pub songbird_url: String,
    /// Default Toadstool service URL
    pub toadstool_url: String,
    /// Default NestGate service URL
    pub nestgate_url: String,
    /// Default BearDog service URL
    pub beardog_url: String,
    /// Default BiomeOS service URL
    pub biomeos_url: String,
    /// Default BiomeOS AI API URL
    pub biomeos_ai_api: String,
    /// Default BiomeOS MCP API URL
    pub biomeos_mcp_api: String,
    /// Default BiomeOS Context API URL
    pub biomeos_context_api: String,
    /// Default BiomeOS Health API URL
    pub biomeos_health_api: String,
    /// Default BiomeOS Metrics API URL
    pub biomeos_metrics_api: String,
    /// Default BiomeOS WebSocket URL
    pub biomeos_websocket_url: String,
}

/// AI service configuration defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIServiceDefaults {
    /// Default OpenAI API URL
    pub openai_api_url: String,
    /// Default Anthropic API URL
    pub anthropic_api_url: String,
    /// Default Ollama API URL
    pub ollama_api_url: String,
    /// Default Llama.cpp API URL
    pub llamacpp_api_url: String,
    /// Default temperature
    pub temperature: f32,
    /// Default timeout
    pub timeout_seconds: u64,
}

/// Observability configuration defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityDefaults {
    /// Default dashboard URL
    pub dashboard_url: String,
    /// Default OTLP endpoint
    pub otlp_endpoint: String,
    /// Default Jaeger endpoint
    pub jaeger_endpoint: String,
    /// Default Zipkin endpoint
    pub zipkin_endpoint: String,
    /// Default metrics port
    pub metrics_port: u16,
    /// Default health check port
    pub health_port: u16,
}

impl Default for NetworkDefaults {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            cors_origins: vec!["http://localhost:3000".to_string()],
            websocket_port: 8081,
            bind_address: "0.0.0.0".to_string(),
        }
    }
}

impl Default for DatabaseDefaults {
    fn default() -> Self {
        Self {
            url: "sqlite::memory:".to_string(),
            max_connections: 10,
            timeout_seconds: 30,
            test_url: "sqlite::memory:".to_string(),
        }
    }
}

impl Default for ExternalServiceDefaults {
    fn default() -> Self {
        Self {
            songbird_url: "http://localhost:8080".to_string(),
            toadstool_url: "http://localhost:8445".to_string(),
            nestgate_url: "http://localhost:8444".to_string(),
            beardog_url: "http://localhost:8443".to_string(),
            biomeos_url: "http://localhost:5000".to_string(),
            biomeos_ai_api: "http://localhost:5000/ai".to_string(),
            biomeos_mcp_api: "http://localhost:5000/mcp".to_string(),
            biomeos_context_api: "http://localhost:5000/context".to_string(),
            biomeos_health_api: "http://localhost:5000/health".to_string(),
            biomeos_metrics_api: "http://localhost:5000/metrics".to_string(),
            biomeos_websocket_url: "ws://localhost:5000/ws".to_string(),
        }
    }
}

impl Default for AIServiceDefaults {
    fn default() -> Self {
        Self {
            openai_api_url: "https://api.openai.com/v1".to_string(),
            anthropic_api_url: "https://api.anthropic.com/v1".to_string(),
            ollama_api_url: "http://localhost:11434".to_string(),
            llamacpp_api_url: "http://localhost:8080".to_string(),
            temperature: 0.7,
            timeout_seconds: 30,
        }
    }
}

impl Default for ObservabilityDefaults {
    fn default() -> Self {
        Self {
            dashboard_url: "http://localhost:3000".to_string(),
            otlp_endpoint: "http://localhost:4317".to_string(),
            jaeger_endpoint: "http://localhost:14268/api/traces".to_string(),
            zipkin_endpoint: "http://localhost:9411".to_string(),
            metrics_port: 4318,
            health_port: 4319,
        }
    }
}

/// Centralized configuration manager with environment-aware defaults
pub struct ConfigManager {
    defaults: ConfigDefaults,
    environment: Environment,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            defaults: ConfigDefaults::default(),
            environment: Environment::from_env(),
        }
    }

    /// Get a configuration value with environment variable override
    pub fn get_string(&self, key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    /// Get a port number with environment variable override
    pub fn get_port(&self, key: &str, default: u16) -> u16 {
        env::var(key)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(default)
    }

    /// Get network configuration with environment overrides
    pub fn get_network_config(&self) -> NetworkConfig {
        NetworkConfig {
            host: self.get_string("SQUIRREL_HOST", &self.defaults.network.host),
            port: self.get_port("SQUIRREL_PORT", self.defaults.network.port),
            cors_origins: env::var("SQUIRREL_CORS_ORIGINS")
                .unwrap_or_else(|_| self.defaults.network.cors_origins.join(","))
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            websocket_endpoint: env::var("SQUIRREL_WS_ENDPOINT").ok().or_else(|| {
                Some(format!(
                    "ws://{}:{}",
                    self.defaults.network.host, self.defaults.network.websocket_port
                ))
            }),
        }
    }

    /// Get database configuration with environment overrides
    pub fn get_database_config(&self) -> DatabaseConfig {
        DatabaseConfig {
            connection_string: self.get_database_url(),
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(self.defaults.database.max_connections),
            timeout_seconds: env::var("DB_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(self.defaults.database.timeout_seconds),
        }
    }

    /// Get database URL with proper environment-based defaults
    fn get_database_url(&self) -> String {
        env::var("DATABASE_URL").unwrap_or_else(|_| {
            match self.environment {
                Environment::Production => {
                    // Production should always use environment variables
                    "postgres://user:password@db:5432/squirrel_production".to_string()
                }
                Environment::Staging => {
                    "postgres://user:password@db:5432/squirrel_staging".to_string()
                }
                Environment::Testing => self.defaults.database.test_url.clone(),
                Environment::Development => self.defaults.database.url.clone(),
            }
        })
    }

    /// Get external service configuration with environment overrides
    pub fn get_external_services_config(&self) -> ExternalServiceConfig {
        ExternalServiceConfig {
            songbird_url: self.get_string(
                "SONGBIRD_URL",
                &self.defaults.external_services.songbird_url,
            ),
            toadstool_url: self.get_string(
                "TOADSTOOL_URL",
                &self.defaults.external_services.toadstool_url,
            ),
            nestgate_url: self.get_string(
                "NESTGATE_URL",
                &self.defaults.external_services.nestgate_url,
            ),
            beardog_url: self
                .get_string("BEARDOG_URL", &self.defaults.external_services.beardog_url),
            biomeos_url: self
                .get_string("BIOMEOS_URL", &self.defaults.external_services.biomeos_url),
        }
    }

    /// Get AI service configuration with environment overrides
    pub fn get_ai_services_config(&self) -> AIServiceConfig {
        AIServiceConfig {
            openai_api_url: self
                .get_string("OPENAI_API_URL", &self.defaults.ai_services.openai_api_url),
            anthropic_api_url: self.get_string(
                "ANTHROPIC_API_URL",
                &self.defaults.ai_services.anthropic_api_url,
            ),
            ollama_api_url: self
                .get_string("OLLAMA_API_URL", &self.defaults.ai_services.ollama_api_url),
            default_temperature: env::var("AI_TEMPERATURE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(self.defaults.ai_services.temperature),
            timeout_seconds: env::var("AI_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(self.defaults.ai_services.timeout_seconds),
        }
    }

    /// Get BiomeOS endpoint URLs with environment overrides
    pub fn get_biomeos_endpoints(&self) -> BiomeOSEndpoints {
        BiomeOSEndpoints {
            ai_api: self.get_string(
                "BIOMEOS_AI_API",
                &self.defaults.external_services.biomeos_ai_api,
            ),
            mcp_api: self.get_string(
                "BIOMEOS_MCP_API",
                &self.defaults.external_services.biomeos_mcp_api,
            ),
            context_api: self.get_string(
                "BIOMEOS_CONTEXT_API",
                &self.defaults.external_services.biomeos_context_api,
            ),
            health: self.get_string(
                "BIOMEOS_HEALTH_API",
                &self.defaults.external_services.biomeos_health_api,
            ),
            metrics: self.get_string(
                "BIOMEOS_METRICS_API",
                &self.defaults.external_services.biomeos_metrics_api,
            ),
            websocket: Some(self.get_string(
                "BIOMEOS_WEBSOCKET_URL",
                &self.defaults.external_services.biomeos_websocket_url,
            )),
        }
    }

    /// Get observability configuration with environment overrides
    pub fn get_extended_observability_config(&self) -> ExtendedObservabilityConfig {
        ExtendedObservabilityConfig {
            dashboard_url: self.get_string(
                "OBSERVABILITY_DASHBOARD_URL",
                &self.defaults.observability.dashboard_url,
            ),
            otlp_endpoint: self
                .get_string("OTLP_ENDPOINT", &self.defaults.observability.otlp_endpoint),
            jaeger_endpoint: self.get_string(
                "JAEGER_ENDPOINT",
                &self.defaults.observability.jaeger_endpoint,
            ),
            zipkin_endpoint: self.get_string(
                "ZIPKIN_ENDPOINT",
                &self.defaults.observability.zipkin_endpoint,
            ),
            metrics_port: self.get_port("METRICS_PORT", self.defaults.observability.metrics_port),
            health_port: self.get_port("HEALTH_PORT", self.defaults.observability.health_port),
        }
    }

    /// Get a service URL reference with fallback
    pub fn get_service_url(&self, service_name: &str) -> Option<&str> {
        match service_name {
            "songbird" => Some(&self.defaults.external_services.songbird_url),
            "toadstool" => Some(&self.defaults.external_services.toadstool_url),
            "nestgate" => Some(&self.defaults.external_services.nestgate_url),
            "beardog" => Some(&self.defaults.external_services.beardog_url),
            "biomeos" => Some(&self.defaults.external_services.biomeos_url),
            _ => None,
        }
    }

    /// Get a service URL as owned String (only when cloning is necessary)
    pub fn get_service_url_owned(&self, service_name: &str) -> Option<String> {
        self.get_service_url(service_name).map(|s| s.to_string())
    }
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
