//! Configuration manager for Squirrel MCP
//!
//! This module provides the main configuration management functionality
//! including loading, validation, and environment-aware configuration.

use super::ai::AIConfig;
use super::types::{Config, CoreConfigError, DatabaseConfig, NetworkConfig};
// Removed unused ConfigDefaults import
use crate::core::ecosystem::EcosystemConfig;
use tracing::{debug, info, warn};
// Removed unused Environment imports

/// Configuration manager trait
pub trait ConfigManager {
    type Error;

    fn load_config(&self) -> Result<Config, Self::Error>;
    fn validate_config(&self, config: &Config) -> Result<(), Self::Error>;
}

/// Default configuration manager implementation
#[derive(Debug, Clone)]
pub struct DefaultConfigManager {
    config: Config,
}

impl DefaultConfigManager {
    /// Create a new default configuration manager
    pub fn new() -> Self {
        // Load configuration from environment variables with defaults
        let config = Config {
            network: NetworkConfig::default(),
            database: DatabaseConfig::default(),
            ai: AIConfig::default(),
            security: crate::core::security::SecurityConfig::default(),
            observability: crate::core::observability::ObservabilityConfig::default(),
            ecosystem: EcosystemConfig::default(),
        };

        Self { config }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self, CoreConfigError> {
        // This is a simplified implementation - in a real system you'd load from env vars
        Ok(Self::new())
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// Get biomeos endpoints configuration
    pub fn get_biomeos_endpoints(&self) -> std::collections::HashMap<String, String> {
        let mut endpoints = std::collections::HashMap::new();
        endpoints.insert("discovery".to_string(), "http://localhost:8080".to_string());
        endpoints.insert("registry".to_string(), "http://localhost:8081".to_string());
        endpoints.insert("health".to_string(), "http://localhost:8082".to_string());
        endpoints
    }

    /// Get external services configuration
    pub fn get_external_services_config(&self) -> std::collections::HashMap<String, String> {
        let mut services = std::collections::HashMap::new();
        services.insert("nestgate".to_string(), "http://localhost:9000".to_string());
        services.insert("toadstool".to_string(), "http://localhost:9001".to_string());
        services.insert("beardog".to_string(), "http://localhost:9002".to_string());
        services
    }
}

impl Default for DefaultConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigManager for DefaultConfigManager {
    type Error = CoreConfigError;

    fn load_config(&self) -> Result<Config, Self::Error> {
        Ok(self.config.clone())
    }

    fn validate_config(&self, config: &Config) -> Result<(), Self::Error> {
        // Enhanced MCP/AI-focused configuration validation
        debug!("🐿️ Validating configuration for MCP coordination");
        
        // Validate MCP protocol settings
        if let Some(mcp_config) = &config.mcp {
            if mcp_config.max_connections == 0 {
                return Err(CoreConfigError::Validation(
                    "MCP max_connections must be greater than 0 for AI coordination".to_string()
                ));
            }
            
            if mcp_config.message_timeout.as_millis() < 100 {
                return Err(CoreConfigError::Validation(
                    "MCP message_timeout too low for reliable AI coordination".to_string()
                ));
            }
            
            debug!("✅ MCP configuration validation passed");
        }
        
        // Validate AI coordination settings
        if let Some(ai_config) = &config.ai {
            if ai_config.context_window_size == 0 {
                return Err(CoreConfigError::Validation(
                    "AI context_window_size must be greater than 0 for context management".to_string()
                ));
            }
            
            if ai_config.max_concurrent_requests == 0 {
                return Err(CoreConfigError::Validation(
                    "AI max_concurrent_requests must be greater than 0 for coordination".to_string()
                ));
            }
            
            debug!("✅ AI configuration validation passed");
        }
        
        // Validate security settings for ecosystem participation
        if let Some(security_config) = &config.security {
            if security_config.session_timeout.as_secs() < 60 {
                return Err(CoreConfigError::Validation(
                    "Security session_timeout too short for reliable MCP sessions".to_string()
                ));
            }
            
            if security_config.max_sessions == 0 {
                return Err(CoreConfigError::Validation(
                    "Security max_sessions must be greater than 0 for coordination".to_string()
                ));
            }
            
            debug!("✅ Security configuration validation passed");
        }
        
        // Validate universal primal settings
        if let Some(universal_config) = &config.universal {
            if universal_config.service_discovery_interval.as_secs() < 10 {
                warn!("⚠️ Service discovery interval is very short, may impact performance");
            }
            
            if universal_config.capability_cache_size == 0 {
                return Err(CoreConfigError::Validation(
                    "Universal capability_cache_size must be greater than 0 for efficient discovery".to_string()
                ));
            }
            
            debug!("✅ Universal configuration validation passed");
        }
        
        // Validate configuration completeness for MCP coordination
        let missing_sections = self.check_required_config_sections(config);
        if !missing_sections.is_empty() {
            return Err(CoreConfigError::Validation(
                format!("Missing required configuration sections for MCP coordination: {:?}", missing_sections)
            ));
        }
        
        info!("✅ Configuration validation completed successfully for MCP/AI coordination");
        Ok(())
    }
    
    /// Check for required configuration sections for MCP/AI coordination
    fn check_required_config_sections(&self, config: &Config) -> Vec<String> {
        let mut missing = Vec::new();
        
        if config.mcp.is_none() {
            missing.push("mcp".to_string());
        }
        if config.ai.is_none() {
            missing.push("ai".to_string());
        }
        if config.security.is_none() {
            missing.push("security".to_string());
        }
        
        missing
    }
}
