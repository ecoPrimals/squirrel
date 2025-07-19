//! Configuration Validation and Default Management
//!
//! This module removes hardcoded values from the codebase and provides:
//! - Configuration validation with proper error messages
//! - Environment-aware defaults
//! - Configuration transformation and migration
//! - Secure handling of sensitive values

use std::collections::HashMap;
use std::env;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug};

use crate::error::Result;
use super::coordinator::{AICoordinatorConfig, RoutingConfig, RoutingStrategy};
use super::events::EventBroadcasterConfig;
use super::streaming::StreamManagerConfig;
use super::server::EnhancedServerConfig;
use super::{EnhancedPlatformConfig, PlatformSettings};

/// Configuration validation errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigValidationError {
    #[error("Missing required configuration: {field}")]
    MissingRequired { field: String },
    
    #[error("Invalid value for {field}: {reason}")]
    InvalidValue { field: String, reason: String },
    
    #[error("Configuration conflict: {description}")]
    Conflict { description: String },
    
    #[error("Environment variable error: {var} - {reason}")]
    EnvironmentError { var: String, reason: String },
}

/// Configuration defaults based on environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDefaults {
    /// Environment type (development, testing, production)
    pub environment: Environment,
    
    /// Default timeouts by environment
    pub timeouts: TimeoutDefaults,
    
    /// Default network settings
    pub network: NetworkDefaults,
    
    /// Default resource limits
    pub resources: ResourceDefaults,
    
    /// Default provider settings
    pub providers: ProviderDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutDefaults {
    pub request_timeout: Duration,
    pub connection_timeout: Duration,
    pub health_check_interval: Duration,
    pub retry_delay: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDefaults {
    pub max_connections: usize,
    pub buffer_size: usize,
    pub keep_alive: bool,
    pub compression: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDefaults {
    pub max_memory_mb: usize,
    pub max_concurrent_requests: usize,
    pub cache_size: usize,
    pub worker_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDefaults {
    pub openai_base_url: String,
    pub anthropic_base_url: String,
    pub ollama_base_url: String,
    pub default_models: HashMap<String, Vec<String>>,
    pub cost_thresholds: CostThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostThresholds {
    pub warning_threshold: f64,
    pub limit_threshold: f64,
    pub daily_budget: f64,
}

/// Configuration validator and builder
pub struct ConfigValidator {
    environment: Environment,
    defaults: ConfigDefaults,
    validation_rules: ValidationRules,
}

#[derive(Debug)]
pub struct ValidationRules {
    pub require_api_keys: bool,
    pub validate_urls: bool,
    pub check_model_availability: bool,
    pub enforce_security: bool,
}

impl ConfigValidator {
    /// Create new validator for specified environment
    pub fn new(environment: Environment) -> Self {
        let defaults = ConfigDefaults::for_environment(&environment);
        let validation_rules = ValidationRules::for_environment(&environment);
        
        Self {
            environment,
            defaults,
            validation_rules,
        }
    }
    
    /// Create validator from environment variables
    pub fn from_env() -> Result<Self> {
        let env_str = env::var("MCP_ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());
            
        let environment = match env_str.to_lowercase().as_str() {
            "development" | "dev" => Environment::Development,
            "testing" | "test" => Environment::Testing,
            "staging" | "stage" => Environment::Staging,
            "production" | "prod" => Environment::Production,
            _ => {
                warn!("Unknown environment '{}', defaulting to development", env_str);
                Environment::Development
            }
        };
        
        info!("Initialized configuration validator for {:?} environment", environment);
        Ok(Self::new(environment))
    }
    
    /// Validate and build complete platform configuration
    pub fn build_platform_config(&self) -> Result<EnhancedPlatformConfig> {
        debug!("Building platform configuration for {:?} environment", self.environment);
        
        let server_config = self.build_server_config()?;
        let ai_coordinator_config = self.build_ai_coordinator_config()?;
        let event_broadcaster_config = self.build_event_broadcaster_config()?;
        let stream_manager_config = self.build_stream_manager_config()?;
        let tool_executor_config = self.build_tool_executor_config()?;
        let platform_settings = self.build_platform_settings()?;
        
        let config = EnhancedPlatformConfig {
            server: server_config,
            ai_coordinator: ai_coordinator_config,
            event_broadcaster: event_broadcaster_config,
            stream_manager: stream_manager_config,
            tool_executor: tool_executor_config,
            platform_settings,
        };
        
        self.validate_platform_config(&config)?;
        
        info!("Successfully built and validated platform configuration");
        Ok(config)
    }
    
    /// Build server configuration with validation
    fn build_server_config(&self) -> Result<EnhancedServerConfig> {
        let port = self.get_env_var_or_default("MCP_SERVER_PORT", 
            self.defaults.network.max_connections.to_string())?
            .parse::<u16>()
            .map_err(|_| ConfigValidationError::InvalidValue {
                field: "port".to_string(),
                reason: "Must be a valid port number".to_string(),
            })?;
            
        if port < 1024 && self.environment == Environment::Production {
            return Err(ConfigValidationError::InvalidValue {
                field: "port".to_string(),
                reason: "Port numbers below 1024 not recommended for production".to_string(),
            }.into());
        }
        
        Ok(EnhancedServerConfig {
            name: self.get_env_var_or_default("MCP_SERVER_NAME", 
                format!("Enhanced MCP Server ({:?})", self.environment))?,
            port,
            max_connections: self.defaults.network.max_connections,
            request_timeout: self.defaults.timeouts.request_timeout,
            enable_metrics: self.environment != Environment::Testing,
            plugin_config: super::server::PluginConfig {
                plugin_directory: self.get_env_var_or_default("MCP_PLUGIN_DIR", 
                    "./plugins".to_string())?,
                max_plugins: 100,
                plugin_timeout: self.defaults.timeouts.request_timeout,
            },
        })
    }
    
    /// Build AI coordinator configuration with validation
    fn build_ai_coordinator_config(&self) -> Result<AICoordinatorConfig> {
        // Get API keys with environment-specific validation
        let openai_key = self.get_optional_api_key("OPENAI_API_KEY")?;
        let anthropic_key = self.get_optional_api_key("ANTHROPIC_API_KEY")?;
        let gemini_key = self.get_optional_api_key("GEMINI_API_KEY")?;
        let openrouter_key = self.get_optional_api_key("OPENROUTER_API_KEY")?;
        let huggingface_token = self.get_optional_api_key("HUGGINGFACE_TOKEN")?;
        
        // Validate at least one provider is configured for production
        if self.environment == Environment::Production {
            if openai_key.is_none() && anthropic_key.is_none() && gemini_key.is_none() {
                return Err(ConfigValidationError::MissingRequired {
                    field: "At least one cloud API key (OpenAI, Anthropic, or Gemini)".to_string(),
                }.into());
            }
        }
        
        Ok(AICoordinatorConfig {
            openai_api_key: openai_key,
            anthropic_api_key: anthropic_key,
            gemini_api_key: gemini_key,
            openrouter_api_key: openrouter_key,
            
            enable_ollama: self.get_env_bool("MCP_ENABLE_OLLAMA", true),
            enable_llamacpp: self.get_env_bool("MCP_ENABLE_LLAMACPP", false),
            enable_native: self.get_env_bool("MCP_ENABLE_NATIVE", false),
            enable_huggingface: self.get_env_bool("MCP_ENABLE_HUGGINGFACE", true),
            
            ollama_config: super::coordinator::OllamaConfig {
                base_url: self.get_env_var_or_default("OLLAMA_BASE_URL", 
                    self.defaults.providers.ollama_base_url.clone())?,
                timeout: self.defaults.timeouts.request_timeout,
                models: self.get_default_models("ollama"),
            },
            
            llamacpp_config: super::coordinator::LlamaCppConfig {
                server_url: self.get_env_var_or_default("LLAMACPP_SERVER_URL", 
                    "http://localhost:8080".to_string())?,
                timeout: self.defaults.timeouts.request_timeout,
                models_path: self.get_env_var_or_default("LLAMACPP_MODELS_PATH", 
                    "./models".to_string())?,
            },
            
            native_config: super::coordinator::NativeConfig {
                models_directory: self.get_env_var_or_default("NATIVE_MODELS_DIR", 
                    "./models".to_string())?,
                max_loaded_models: 3,
                use_gpu: self.get_env_bool("MCP_USE_GPU", true),
            },
            
            huggingface_config: super::coordinator::HuggingFaceConfig {
                api_token: huggingface_token,
                cache_directory: self.get_env_var_or_default("HF_CACHE_DIR", 
                    "./hf_cache".to_string())?,
                use_local_cache: true,
            },
            
            custom_providers: HashMap::new(),
            
            routing: RoutingConfig {
                default_strategy: self.get_routing_strategy()?,
                fallback_enabled: true,
                cost_optimization: self.environment == Environment::Production,
                latency_optimization: self.environment != Environment::Production,
                quality_optimization: true,
            },
            
            max_concurrent_requests: self.defaults.resources.max_concurrent_requests,
            request_timeout: self.defaults.timeouts.request_timeout,
            retry_attempts: if self.environment == Environment::Production { 3 } else { 1 },
        })
    }
    
    /// Build event broadcaster configuration
    fn build_event_broadcaster_config(&self) -> Result<EventBroadcasterConfig> {
        Ok(EventBroadcasterConfig {
            max_event_types: 1000,
            channel_capacity: self.defaults.network.buffer_size,
            max_history_size: if self.environment == Environment::Production { 50000 } else { 10000 },
            max_history_age_seconds: 86400, // 24 hours
            enable_persistence: self.environment == Environment::Production,
        })
    }
    
    /// Build stream manager configuration
    fn build_stream_manager_config(&self) -> Result<StreamManagerConfig> {
        Ok(StreamManagerConfig {
            max_concurrent_streams: self.defaults.resources.max_concurrent_requests,
            default_buffer_size: self.defaults.network.buffer_size,
            default_timeout: self.defaults.timeouts.request_timeout,
            cleanup_interval: Duration::from_secs(60),
            enable_metrics: true,
        })
    }
    
    /// Build tool executor configuration
    fn build_tool_executor_config(&self) -> Result<super::coordinator::ToolExecutorConfig> {
        Ok(super::coordinator::ToolExecutorConfig {
            max_concurrent_executions: self.defaults.resources.max_concurrent_requests / 2,
            execution_timeout: Duration::from_secs(300),
            retry_failed_executions: self.environment != Environment::Testing,
        })
    }
    
    /// Build platform settings
    fn build_platform_settings(&self) -> Result<PlatformSettings> {
        Ok(PlatformSettings {
            name: "Squirrel Universal MCP".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            enable_experimental: self.environment != Environment::Production,
            max_concurrent_operations: self.defaults.resources.max_concurrent_requests,
            request_timeout: self.defaults.timeouts.request_timeout,
            health_check_interval: self.defaults.timeouts.health_check_interval,
            enable_metrics: self.environment != Environment::Testing,
            debug_mode: self.environment == Environment::Development,
        })
    }
    
    /// Validate complete platform configuration
    fn validate_platform_config(&self, config: &EnhancedPlatformConfig) -> Result<()> {
        // Validate resource constraints
        if config.ai_coordinator.max_concurrent_requests > config.server.max_connections {
            return Err(ConfigValidationError::Conflict {
                description: "AI coordinator concurrent requests cannot exceed server max connections".to_string(),
            }.into());
        }
        
        // Validate timeout consistency
        if config.ai_coordinator.request_timeout > config.platform_settings.request_timeout {
            warn!("AI coordinator timeout exceeds platform timeout, this may cause issues");
        }
        
        // Production-specific validations
        if self.environment == Environment::Production {
            if config.platform_settings.debug_mode {
                return Err(ConfigValidationError::InvalidValue {
                    field: "debug_mode".to_string(),
                    reason: "Debug mode should not be enabled in production".to_string(),
                }.into());
            }
            
            if config.server.port < 1024 {
                warn!("Using privileged port {} in production", config.server.port);
            }
        }
        
        Ok(())
    }
    
    /// Helper methods for configuration building
    fn get_env_var_or_default(&self, var: &str, default: String) -> Result<String> {
        Ok(env::var(var).unwrap_or(default))
    }
    
    fn get_env_bool(&self, var: &str, default: bool) -> bool {
        env::var(var)
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(default)
    }
    
    fn get_optional_api_key(&self, var: &str) -> Result<Option<String>> {
        match env::var(var) {
            Ok(key) => {
                if self.validation_rules.require_api_keys && key.len() < 10 {
                    return Err(ConfigValidationError::InvalidValue {
                        field: var.to_string(),
                        reason: "API key appears to be too short".to_string(),
                    }.into());
                }
                Ok(Some(key))
            }
            Err(_) => {
                if self.validation_rules.require_api_keys && self.environment == Environment::Production {
                    warn!("API key {} not found for production environment", var);
                }
                Ok(None)
            }
        }
    }
    
    fn get_routing_strategy(&self) -> Result<RoutingStrategy> {
        let strategy_str = env::var("MCP_ROUTING_STRATEGY")
            .unwrap_or_else(|_| match self.environment {
                Environment::Production => "cost_optimized".to_string(),
                Environment::Development => "round_robin".to_string(),
                _ => "best_fit".to_string(),
            });
            
        match strategy_str.to_lowercase().as_str() {
            "best_fit" => Ok(RoutingStrategy::BestFit),
            "cost_optimized" => Ok(RoutingStrategy::CostOptimized),
            "latency_optimized" => Ok(RoutingStrategy::LatencyOptimized),
            "round_robin" => Ok(RoutingStrategy::RoundRobin),
            "weighted_random" => Ok(RoutingStrategy::WeightedRandom),
            _ => Err(ConfigValidationError::InvalidValue {
                field: "routing_strategy".to_string(),
                reason: format!("Unknown strategy: {}", strategy_str),
            }.into()),
        }
    }
    
    fn get_default_models(&self, provider: &str) -> Vec<String> {
        self.defaults.providers.default_models
            .get(provider)
            .cloned()
            .unwrap_or_default()
    }
}

impl ConfigDefaults {
    pub fn for_environment(env: &Environment) -> Self {
        let base_timeout = match env {
            Environment::Development => Duration::from_secs(60),
            Environment::Testing => Duration::from_secs(5),
            Environment::Staging => Duration::from_secs(30),
            Environment::Production => Duration::from_secs(30),
        };
        
        Self {
            environment: env.clone(),
            timeouts: TimeoutDefaults {
                request_timeout: base_timeout,
                connection_timeout: base_timeout / 2,
                health_check_interval: base_timeout * 2,
                retry_delay: Duration::from_secs(1),
            },
            network: NetworkDefaults {
                max_connections: match env {
                    Environment::Development => 100,
                    Environment::Testing => 10,
                    Environment::Staging => 500,
                    Environment::Production => 1000,
                },
                buffer_size: 8192,
                keep_alive: true,
                compression: *env == Environment::Production,
            },
            resources: ResourceDefaults {
                max_memory_mb: match env {
                    Environment::Development => 2048,
                    Environment::Testing => 512,
                    Environment::Staging => 4096,
                    Environment::Production => 8192,
                },
                max_concurrent_requests: match env {
                    Environment::Development => 50,
                    Environment::Testing => 5,
                    Environment::Staging => 200,
                    Environment::Production => 500,
                },
                cache_size: 1000,
                worker_threads: num_cpus::get(),
            },
            providers: ProviderDefaults {
                openai_base_url: "https://api.openai.com/v1".to_string(),
                anthropic_base_url: "https://api.anthropic.com".to_string(),
                ollama_base_url: "http://localhost:11434".to_string(),
                default_models: Self::get_default_models_by_provider(),
                cost_thresholds: CostThresholds {
                    warning_threshold: 10.0,
                    limit_threshold: 50.0,
                    daily_budget: 100.0,
                },
            },
        }
    }
    
    fn get_default_models_by_provider() -> HashMap<String, Vec<String>> {
        let mut models = HashMap::new();
        
        models.insert("openai".to_string(), vec![
            "gpt-4".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-3.5-turbo".to_string(),
        ]);
        
        models.insert("anthropic".to_string(), vec![
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
        ]);
        
        models.insert("ollama".to_string(), vec![
            "llama2".to_string(),
            "codellama".to_string(),
            "mistral".to_string(),
        ]);
        
        models.insert("gemini".to_string(), vec![
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
        ]);
        
        models
    }
}

impl ValidationRules {
    pub fn for_environment(env: &Environment) -> Self {
        Self {
            require_api_keys: *env == Environment::Production,
            validate_urls: true,
            check_model_availability: *env != Environment::Testing,
            enforce_security: *env == Environment::Production,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_validation_development() {
        let validator = ConfigValidator::new(Environment::Development);
        let config = validator.build_platform_config().unwrap();
        
        assert_eq!(config.platform_settings.debug_mode, true);
        assert_eq!(config.platform_settings.enable_experimental, true);
    }
    
    #[test]
    fn test_config_validation_production() {
        let validator = ConfigValidator::new(Environment::Production);
        let config = validator.build_platform_config().unwrap();
        
        assert_eq!(config.platform_settings.debug_mode, false);
        assert_eq!(config.platform_settings.enable_experimental, false);
        assert_eq!(config.ai_coordinator.routing.cost_optimization, true);
    }
    
    #[test]
    fn test_environment_from_string() {
        std::env::set_var("MCP_ENVIRONMENT", "production");
        let validator = ConfigValidator::from_env().unwrap();
        assert_eq!(validator.environment, Environment::Production);
    }
} 