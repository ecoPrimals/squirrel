---
title: "Galaxy MCP Adapter Configuration"
description: "Configuration standards for the Galaxy MCP adapter crate"
version: "0.1.0"
last_updated: "2025-03-27"
status: "draft"
owners:
  primary: ["DataScienceBioLab", "mcp-team"]
  reviewers: ["core-team", "devops-team"]
---

# Galaxy MCP Adapter Configuration

## 1. Overview

This specification defines the configuration approach for the Galaxy MCP adapter crate. It covers adapter-specific configuration options, integration with the existing MCP configuration system, and runtime management of configuration settings.

## 2. Configuration Integration

The Galaxy adapter crate will leverage the existing configuration infrastructure from the MCP project while adding Galaxy-specific options:

```rust
use mcp::config::{Config as McpConfig, ConfigBuilder};
use context::config::ContextConfig;

pub struct GalaxyAdapterConfig {
    // Galaxy-specific configuration
    pub galaxy_url: String,
    pub api_key: Option<String>,
    pub timeout: std::time::Duration,
    pub max_retries: u32,
    pub default_history_id: Option<String>,
    
    // References to parent configurations
    pub mcp_config: McpConfig,
    pub context_config: ContextConfig,
}

impl GalaxyAdapterConfig {
    pub fn new(
        galaxy_url: String,
        api_key: Option<String>,
        mcp_config: McpConfig,
        context_config: ContextConfig
    ) -> Self {
        Self {
            galaxy_url,
            api_key,
            timeout: std::time::Duration::from_secs(30),
            max_retries: 3,
            default_history_id: None,
            mcp_config,
            context_config,
        }
    }
    
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    // Other builder methods...
}
```

## 3. Configuration Sources

The adapter will support configuration from multiple sources, with the following precedence (highest to lowest):

1. Programmatic configuration via the builder API
2. Environment variables
3. Configuration files (shared with the main MCP configuration)
4. Default values

### 3.1 Environment Variables

Galaxy adapter-specific environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `GALAXY_MCP_URL` | Galaxy API URL | `https://usegalaxy.org/api` |
| `GALAXY_MCP_API_KEY` | Galaxy API key | None |
| `GALAXY_MCP_TIMEOUT` | API timeout in seconds | 30 |
| `GALAXY_MCP_MAX_RETRIES` | Maximum retry attempts | 3 |
| `GALAXY_MCP_DEFAULT_HISTORY` | Default history ID | None |

### 3.2 Configuration File Integration

The adapter will use a `galaxy` section within the MCP configuration file:

```toml
# Example configuration in the shared config file

[mcp]
# Existing MCP configuration...

[context]
# Existing context configuration...

[galaxy]
url = "https://usegalaxy.org/api"
# API key should be provided via environment variable
timeout = 30
max_retries = 3
default_history_id = ""
```

## 4. Adapter Configuration Options

### 4.1 Galaxy API Connection

```toml
[galaxy]
# Galaxy API URL
url = "https://usegalaxy.org/api"
# API key (prefer environment variable GALAXY_MCP_API_KEY)
api_key = ""
# Connection timeout in seconds
timeout = 30
# Maximum number of retries for failed requests
max_retries = 3
# Retry backoff factor in seconds
retry_backoff = 1.5
# Default history ID (optional)
default_history_id = ""
```

### 4.2 Caching Options

```toml
[galaxy.cache]
# Enable tool caching
enable_tool_cache = true
# Tool cache TTL in seconds
tool_cache_ttl = 3600
# Enable workflow caching
enable_workflow_cache = true
# Workflow cache TTL in seconds
workflow_cache_ttl = 3600
```

### 4.3 Performance Tuning

```toml
[galaxy.performance]
# Enable keepalive connections
keepalive = true
# Keepalive timeout in seconds
keepalive_timeout = 30
# Enable connection pooling
connection_pooling = true
# Maximum connections per pool
max_connections_per_pool = 10
```

## 5. Configuration Loading

The adapter will integrate with the MCP project's configuration system:

```rust
pub fn create_adapter_from_config() -> Result<GalaxyAdapter, ConfigError> {
    // Load shared MCP configuration
    let mcp_config = mcp::config::load_config()?;
    
    // Load shared context configuration
    let context_config = context::config::load_config()?;
    
    // Extract Galaxy-specific configuration
    let galaxy_config = if let Some(galaxy_section) = mcp_config.get_section("galaxy") {
        GalaxyAdapterConfig::from_config_section(galaxy_section, mcp_config.clone(), context_config.clone())?
    } else {
        // Create from environment variables if section not found
        GalaxyAdapterConfig::from_env(mcp_config.clone(), context_config.clone())?
    };
    
    // Create adapter with configuration
    Ok(GalaxyAdapter::new(galaxy_config))
}
```

## 6. Configuration Validation

The adapter will perform validation of Galaxy-specific configuration:

```rust
impl GalaxyAdapterConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate Galaxy URL
        if self.galaxy_url.is_empty() {
            return Err(ConfigError::InvalidValue("galaxy_url cannot be empty".into()));
        }
        
        if !self.galaxy_url.starts_with("http://") && !self.galaxy_url.starts_with("https://") {
            return Err(ConfigError::InvalidValue("galaxy_url must start with http:// or https://".into()));
        }
        
        // Validate timeout
        if self.timeout.as_secs() == 0 {
            return Err(ConfigError::InvalidValue("timeout must be greater than zero".into()));
        }
        
        // Validate max retries
        if self.max_retries > 10 {
            return Err(ConfigError::InvalidValue("max_retries must be 10 or less".into()));
        }
        
        Ok(())
    }
}
```

## 7. Usage Examples

### 7.1 Programmatic Configuration

```rust
use galaxy_mcp::{GalaxyAdapter, GalaxyAdapterConfig};
use mcp::config::ConfigBuilder as McpConfigBuilder;
use context::config::ConfigBuilder as ContextConfigBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create MCP config
    let mcp_config = McpConfigBuilder::new()
        .with_log_level("debug")
        .build()?;
    
    // Create context config
    let context_config = ContextConfigBuilder::new()
        .build()?;
    
    // Create Galaxy adapter config
    let galaxy_config = GalaxyAdapterConfig::new(
        "https://usegalaxy.org/api".to_string(),
        Some("your-api-key".to_string()),
        mcp_config,
        context_config
    )
    .with_timeout(std::time::Duration::from_secs(60))
    .with_max_retries(5);
    
    // Create adapter
    let adapter = GalaxyAdapter::new(galaxy_config);
    
    // Use adapter...
    
    Ok(())
}
```

### 7.2 Environment Variables

```bash
# Set Galaxy adapter configuration
export GALAXY_MCP_URL="https://usegalaxy.org/api"
export GALAXY_MCP_API_KEY="your-api-key"
export GALAXY_MCP_TIMEOUT=60
export GALAXY_MCP_MAX_RETRIES=5

# Run application
cargo run --example galaxy_workflow
```

### 7.3 Configuration File

```toml
# .mcp/config.toml
[mcp]
log_level = "info"

[context]
data_dir = "./data"

[galaxy]
url = "https://usegalaxy.org/api"
timeout = 60
max_retries = 5
default_history_id = "f2db41e1fa331b3e"
```

```rust
// Load from file
let adapter = galaxy_mcp::create_adapter_from_config()?;
```

## 8. Implementation Example

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyAdapterConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub timeout: u64,
    pub max_retries: u32,
    pub default_history_id: Option<String>,
    
    #[serde(default)]
    pub cache: CacheConfig,
    
    #[serde(default)]
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheConfig {
    pub enable_tool_cache: bool,
    pub tool_cache_ttl: u64,
    pub enable_workflow_cache: bool,
    pub workflow_cache_ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceConfig {
    pub keepalive: bool,
    pub keepalive_timeout: u64,
    pub connection_pooling: bool,
    pub max_connections_per_pool: u32,
}

impl Default for GalaxyAdapterConfig {
    fn default() -> Self {
        Self {
            url: "https://usegalaxy.org/api".to_string(),
            api_key: None,
            timeout: 30,
            max_retries: 3,
            default_history_id: None,
            cache: CacheConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl GalaxyAdapterConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(url) = std::env::var("GALAXY_MCP_URL") {
            config.url = url;
        }
        
        if let Ok(api_key) = std::env::var("GALAXY_MCP_API_KEY") {
            config.api_key = Some(api_key);
        }
        
        if let Ok(timeout) = std::env::var("GALAXY_MCP_TIMEOUT") {
            if let Ok(timeout) = timeout.parse() {
                config.timeout = timeout;
            }
        }
        
        if let Ok(max_retries) = std::env::var("GALAXY_MCP_MAX_RETRIES") {
            if let Ok(max_retries) = max_retries.parse() {
                config.max_retries = max_retries;
            }
        }
        
        if let Ok(default_history_id) = std::env::var("GALAXY_MCP_DEFAULT_HISTORY") {
            config.default_history_id = Some(default_history_id);
        }
        
        config
    }
}
```

## 9. Related Specifications

- [Galaxy MCP Integration Plan](galaxy-mcp-integration.md)
- [API Mapping](api-mapping.md)
- [Security Model](security-model.md)

<version>0.1.0</version> 