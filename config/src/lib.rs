use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use std::collections::HashMap;
use url::Url;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub network: NetworkConfig,
    pub database: DatabaseConfig,
    pub ai: AIConfig,
    pub security: SecurityConfig,
    pub observability: ObservabilityConfig,
    pub ecosystem: EcosystemConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkConfig {
    pub host: String,
    pub port: u16,
    pub websocket_port: u16,
    pub dashboard_port: u16,
    pub api_port: u16,
    pub max_connections: usize,
    pub timeout: Duration,
    pub buffer_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub backend: DatabaseBackend,
    pub connection_string: String,
    pub max_connections: u32,
    pub timeout: Duration,
    pub encryption_enabled: bool,
    pub backup_enabled: bool,
    pub replication_factor: u32,
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
    pub endpoint: Url,
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
    Sovereign,      // Operate independently, coordinate when available
    #[serde(rename = "coordinated")]
    Coordinated,    // Require coordination with other primals
    #[serde(rename = "standalone")]
    Standalone,     // Operate without any coordination
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
    pub fn new() -> Result<Self, anyhow::Error> {
        let config_path = std::env::var("SQUIRREL_MCP_CONFIG")
            .unwrap_or_else(|_| "config.toml".to_string());

        if std::path::Path::new(&config_path).exists() {
            Self::from_file(&config_path)
        } else {
            tracing::info!("Config file not found, using defaults and environment variables");
            Self::from_env()
        }
    }

    pub fn from_file(path: &str) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;
        
        // Override with environment variables
        config.apply_env_overrides();
        
        Ok(config)
    }

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
        if let Ok(ws_port) = std::env::var("SQUIRREL_MCP_WEBSOCKET_PORT") {
            self.network.websocket_port = ws_port.parse().unwrap_or(self.network.websocket_port);
        }
        if let Ok(dashboard_port) = std::env::var("SQUIRREL_MCP_DASHBOARD_PORT") {
            self.network.dashboard_port = dashboard_port.parse().unwrap_or(self.network.dashboard_port);
        }

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
                if let Ok(url) = ollama_endpoint.parse() {
                    provider.endpoint = url;
                }
            }
        }
    }

    pub fn validate(&self) -> Result<(), anyhow::Error> {
        // Validate network configuration
        if self.network.port == 0 {
            return Err(anyhow::anyhow!("Network port cannot be zero"));
        }
        if self.network.websocket_port == 0 {
            return Err(anyhow::anyhow!("WebSocket port cannot be zero"));
        }
        if self.network.dashboard_port == 0 {
            return Err(anyhow::anyhow!("Dashboard port cannot be zero"));
        }

        // Validate AI providers
        if self.ai.providers.is_empty() {
            return Err(anyhow::anyhow!("At least one AI provider must be configured"));
        }

        let default_provider_exists = self.ai.providers
            .iter()
            .any(|p| p.name == self.ai.default_provider);
        if !default_provider_exists {
            return Err(anyhow::anyhow!("Default AI provider '{}' not found in providers list", self.ai.default_provider));
        }

        // Validate ecosystem configuration
        if self.ecosystem.enabled {
            // Validate discovery configuration
            if self.ecosystem.discovery.songbird_endpoint.is_some() {
                if let Some(ref endpoint) = self.ecosystem.discovery.songbird_endpoint {
                    if !endpoint.starts_with("https://") && !endpoint.starts_with("http://") {
                        tracing::warn!("Songbird discovery endpoint should use HTTP/HTTPS protocol");
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
            return Err(anyhow::anyhow!("JWT secret key ID is required when authentication is enabled"));
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
                websocket_port: 8081,
                dashboard_port: 8082,
                api_port: 8083,
                max_connections: 1000,
                timeout: Duration::from_secs(30),
                buffer_size: 8192,
            },
            database: DatabaseConfig {
                backend: DatabaseBackend::NestGate,
                connection_string: "nestgate://localhost:8444".to_string(),
                max_connections: 10,
                timeout: Duration::from_secs(30),
                encryption_enabled: true,
                backup_enabled: true,
                replication_factor: 1,
            },
            ai: AIConfig {
                providers: vec![
                    AIProvider {
                        name: "openai".to_string(),
                        provider_type: AIProviderType::OpenAI,
                        endpoint: Self::parse_endpoint_safely("https://api.openai.com/v1", "OpenAI"),
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
                        endpoint: Self::parse_endpoint_safely("https://api.anthropic.com/v1", "Anthropic"),
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
    fn parse_endpoint_safely(url: &str, provider_name: &str) -> Url {
        // Try the requested URL first
        if let Ok(parsed) = url.parse::<Url>() {
            return parsed;
        }
        
        tracing::warn!("Failed to parse {} endpoint URL '{}'. Trying fallbacks.", provider_name, url);
        
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
                return parsed;
            }
        }
        
        // PRODUCTION SAFE: If all known fallbacks fail, create a URL that represents a disabled provider
        // This prevents crashes while maintaining system stability
        tracing::error!("CRITICAL: All URL parsing failed for {}. Provider will be disabled.", provider_name);
        
        // Create a disabled URL that will cause the provider to be skipped
        // This approach is safe and prevents application crashes
        Self::create_disabled_url(provider_name)
    }
    
    /// Creates a URL that represents a disabled provider
    /// This is a helper function that uses safe URL creation patterns
    fn create_disabled_url(provider_name: &str) -> Url {
        // Try multiple strategies to create a URL that represents a disabled provider
        // This URL will cause the provider to be skipped during initialization
        
        // Strategy 1: Create a provider-specific disabled URL
        let clean_name = provider_name.chars()
            .filter(|c| c.is_alphanumeric())
            .take(10)
            .collect::<String>();
        
        if !clean_name.is_empty() {
            let disabled_url = format!("http://disabled-{}", clean_name);
            if let Ok(url) = disabled_url.parse::<Url>() {
                return url;
            }
        }
        
        // Strategy 2: Use a generic disabled URL
        if let Ok(url) = "http://disabled".parse::<Url>() {
            return url;
        }
        
        // Strategy 3: Use localhost as a safe fallback
        if let Ok(url) = "http://localhost".parse::<Url>() {
            return url;
        }
        
        // Strategy 4: Use IP address as fallback
        if let Ok(url) = "http://127.0.0.1".parse::<Url>() {
            return url;
        }
        
        // Strategy 5: Use the most basic HTTP URL
        if let Ok(url) = "http://error".parse::<Url>() {
            return url;
        }
        
        // FINAL FALLBACK: If even basic URLs fail, this indicates a fundamental system issue
        // Log the critical error and exit the process safely rather than panic
        tracing::error!("SYSTEM CRITICAL: Cannot create any URL for {}. URL crate appears broken.", provider_name);
        tracing::error!("This indicates a fundamental system failure. Exiting to prevent undefined behavior.");
        
        // Exit the process cleanly rather than panic or use dangerous patterns
        std::process::exit(1);
    }
} 