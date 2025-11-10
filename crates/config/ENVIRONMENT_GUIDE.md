# Environment Configuration Guide

**Date**: November 10, 2025  
**Status**: Active  
**Related ADR**: [ADR-008](../../docs/adr/ADR-008-configuration-standardization.md)

---

## Overview

Squirrel follows the [12-Factor App](https://12factor.net/) methodology for configuration management, using environment variables as the primary configuration mechanism. This guide establishes naming conventions, usage patterns, and best practices for environment-based configuration.

## Quick Start

```rust
use squirrel_mcp_config::Environment;

// Detect current environment
let env = Environment::from_env(); // Reads MCP_ENV

// Check environment
if env.is_production() {
    // Production-specific logic
}

// Load environment-aware configuration
use squirrel_mcp_config::unified::ConfigLoader;
let config = ConfigLoader::load()?; // Auto-loads from environment
```

## Environment Detection

### Environment Types

Squirrel recognizes four environment types:

```rust
pub enum Environment {
    Development,  // Local development
    Testing,      // Automated testing
    Staging,      // Pre-production staging
    Production,   // Live production
}
```

### Setting the Environment

Set via the `MCP_ENV` environment variable:

```bash
# Development (default)
export MCP_ENV=development

# Testing
export MCP_ENV=testing

# Staging
export MCP_ENV=staging

# Production
export MCP_ENV=production
```

**Aliases**: `dev`, `test`, `stage`, `prod` are also recognized.

### Environment Detection API

```rust
use squirrel_mcp_config::Environment;

// Get current environment
let env = Environment::from_env();

// Check environment type
match env {
    Environment::Development => println!("Running in development"),
    Environment::Testing => println!("Running in tests"),
    Environment::Staging => println!("Running in staging"),
    Environment::Production => println!("Running in production"),
}

// Convenience methods
if env.is_development() { /* ... */ }
if env.is_production() { /* ... */ }

// Get config file suffix
let suffix = env.config_suffix(); // "dev", "test", "staging", "prod"
```

---

## Environment Variable Naming Conventions

### Prefix Standards

All Squirrel environment variables use consistent prefixes:

| Prefix | Purpose | Example |
|--------|---------|---------|
| `SQUIRREL_` | Core Squirrel configuration | `SQUIRREL_HTTP_PORT` |
| `MCP_` | MCP protocol configuration | `MCP_TIMEOUT_SECS` |
| `DATABASE_` | Database configuration | `DATABASE_URL` |
| `OPENAI_` | OpenAI provider | `OPENAI_API_KEY` |
| `ANTHROPIC_` | Anthropic provider | `ANTHROPIC_API_KEY` |
| `GEMINI_` | Gemini provider | `GEMINI_API_KEY` |

### Naming Rules

1. **ALL_UPPERCASE**: Use uppercase with underscores
   - ✅ `SQUIRREL_HTTP_PORT`
   - ❌ `squirrel_http_port`

2. **Hierarchical**: Use underscores to indicate hierarchy
   - ✅ `SQUIRREL_DATABASE_CONNECTION_TIMEOUT`
   - ❌ `SQUIRREL_DB_CONN_TIMEOUT`

3. **Descriptive**: Use full words, avoid abbreviations
   - ✅ `SQUIRREL_MAX_CONNECTIONS`
   - ❌ `SQUIRREL_MAX_CONN`

4. **Typed Suffixes**: Use suffixes to indicate type
   - `_PORT` for ports (e.g., `SQUIRREL_HTTP_PORT`)
   - `_URL` for URLs (e.g., `DATABASE_URL`)
   - `_PATH` for file paths (e.g., `SQUIRREL_CONFIG_PATH`)
   - `_SECS` for time in seconds (e.g., `MCP_TIMEOUT_SECS`)
   - `_MS` for time in milliseconds (e.g., `MCP_RETRY_DELAY_MS`)
   - `_ENABLED` for booleans (e.g., `SQUIRREL_TLS_ENABLED`)

### Standard Variables

#### Core Configuration

```bash
# Environment
MCP_ENV=development                      # Environment type

# Network
SQUIRREL_HTTP_PORT=8080                  # HTTP server port
SQUIRREL_WEBSOCKET_PORT=8081             # WebSocket server port
SQUIRREL_BIND_ADDRESS=0.0.0.0            # Bind address
SQUIRREL_PUBLIC_ADDRESS=example.com      # Public address

# Timeouts
MCP_CONNECTION_TIMEOUT_SECS=30           # Connection timeout
MCP_REQUEST_TIMEOUT_SECS=60              # Request timeout
MCP_HEALTH_CHECK_TIMEOUT_SECS=10         # Health check timeout
MCP_SESSION_TIMEOUT_SECS=3600            # Session timeout (1 hour)

# Security
SQUIRREL_TLS_ENABLED=true                # Enable TLS
SQUIRREL_TLS_CERT_PATH=/path/to/cert.pem
SQUIRREL_TLS_KEY_PATH=/path/to/key.pem
SQUIRREL_JWT_SECRET=your-secret-here     # JWT secret (32+ chars)
SQUIRREL_REQUIRE_AUTH=true               # Require authentication

# Monitoring
SQUIRREL_METRICS_ENABLED=true            # Enable metrics
SQUIRREL_PROMETHEUS_PORT=9090            # Prometheus metrics port
SQUIRREL_LOG_LEVEL=info                  # Log level
```

#### Database Configuration

```bash
DATABASE_URL=postgresql://user:pass@localhost/db   # Connection string
DATABASE_MAX_CONNECTIONS=100                        # Connection pool size
DATABASE_CONNECTION_TIMEOUT_SECS=30                 # Connection timeout
DATABASE_IDLE_TIMEOUT_SECS=600                      # Idle timeout (10 min)
```

#### AI Provider Configuration

```bash
# OpenAI
OPENAI_API_KEY=sk-...                    # API key
OPENAI_ENDPOINT=https://api.openai.com/v1 # Base URL
OPENAI_TIMEOUT_SECS=60                   # Request timeout

# Anthropic
ANTHROPIC_API_KEY=sk-ant-...
ANTHROPIC_ENDPOINT=https://api.anthropic.com
ANTHROPIC_TIMEOUT_SECS=60

# Gemini
GEMINI_API_KEY=...
GEMINI_ENDPOINT=https://generativelanguage.googleapis.com
```

---

## Configuration Precedence

Squirrel loads configuration with the following precedence (highest to lowest):

1. **Environment Variables** (highest priority)
2. **Configuration File** (`squirrel.toml` or `config/squirrel.toml`)
3. **Platform Defaults** (OS-specific)
4. **Secure Defaults** (lowest priority)

### Example: Port Configuration

```bash
# 1. Environment variable (highest)
export SQUIRREL_HTTP_PORT=8080

# 2. Configuration file
# squirrel.toml:
# [network]
# http_port = 3000

# 3. Platform default (varies by OS)

# 4. Secure default: 8080
```

**Result**: Port will be `8080` (from environment variable).

---

## Environment-Aware Defaults

Configuration defaults vary by environment:

### Development

```rust
// Development defaults (optimized for DX)
timeouts:
  connection: 60s   // Generous for debugging
  request: 120s     // Long timeouts
  
network:
  max_connections: 100
  compression: false // Faster, easier debugging
  
logging:
  level: debug
  pretty: true       // Human-readable logs
  
security:
  tls_required: false
  require_auth: false
```

### Production

```rust
// Production defaults (optimized for performance & security)
timeouts:
  connection: 30s   // Reasonable
  request: 60s      // Standard
  
network:
  max_connections: 1000
  compression: true  // Bandwidth optimization
  
logging:
  level: info
  pretty: false      // JSON logs for parsing
  
security:
  tls_required: true
  require_auth: true
```

### Testing

```rust
// Testing defaults (optimized for speed)
timeouts:
  connection: 5s    // Fast failures
  request: 10s      // Quick tests
  
network:
  max_connections: 10
  
logging:
  level: warn       // Quiet unless error
```

---

## Usage Patterns

### Pattern 1: Direct Environment Variables

Best for simple configuration:

```rust
use std::env;

let port = env::var("SQUIRREL_HTTP_PORT")
    .unwrap_or_else(|_| "8080".to_string())
    .parse::<u16>()
    .expect("Invalid port");
```

### Pattern 2: ConfigLoader (Recommended)

Best for complex configuration with validation:

```rust
use squirrel_mcp_config::unified::ConfigLoader;

// Load with full precedence
let loaded = ConfigLoader::load()?;
let config = loaded.into_config();

// Access configuration
println!("HTTP Port: {}", config.network.http_port);
println!("Timeout: {}s", config.timeouts.connection_timeout_secs);

// Check sources
println!("Loaded from: {:?}", loaded.sources());
```

### Pattern 3: Environment-Aware Configuration

Best for environment-specific logic:

```rust
use squirrel_mcp_config::{Environment, unified::ConfigLoader};

let env = Environment::from_env();
let config = ConfigLoader::load()?;

match env {
    Environment::Development => {
        // Development-specific setup
        println!("Debug mode enabled");
    }
    Environment::Production => {
        // Production-specific setup
        if !config.security.enabled {
            panic!("Security must be enabled in production!");
        }
    }
    _ => {}
}
```

### Pattern 4: Custom Environment Loading

Best for specialized needs:

```rust
use squirrel_mcp_config::unified::ConfigLoader;

let config = ConfigLoader::new()
    .with_file_if_exists("custom-config.toml")?
    .with_env_prefix("MYAPP_")?  // Custom prefix
    .validate()?
    .build()?;
```

---

## Environment Files

### .env File Support

While not directly supported by Squirrel, you can use `dotenv` or similar:

```bash
# .env (development)
MCP_ENV=development
SQUIRREL_HTTP_PORT=8080
DATABASE_URL=postgresql://localhost/squirrel_dev
SQUIRREL_LOG_LEVEL=debug
```

```rust
// Load .env file (if using dotenv crate)
#[cfg(debug_assertions)]
dotenv::dotenv().ok();

// Now load config
let config = ConfigLoader::load()?;
```

### Environment-Specific Config Files

Squirrel automatically tries to load environment-specific files:

```
config/
  squirrel.toml           # Base configuration
  squirrel.dev.toml       # Development overrides
  squirrel.test.toml      # Testing overrides
  squirrel.staging.toml   # Staging overrides
  squirrel.prod.toml      # Production overrides
```

**Loading order**:
1. Load `squirrel.toml` (base)
2. Load `squirrel.{env}.toml` (environment-specific)
3. Apply environment variables
4. Validate

---

## Docker & Container Deployment

### Docker Compose

```yaml
version: '3.8'
services:
  squirrel:
    image: squirrel:latest
    environment:
      MCP_ENV: production
      SQUIRREL_HTTP_PORT: 8080
      SQUIRREL_WEBSOCKET_PORT: 8081
      DATABASE_URL: postgresql://user:pass@db:5432/squirrel
      OPENAI_API_KEY: ${OPENAI_API_KEY}  # From host environment
      SQUIRREL_LOG_LEVEL: info
    ports:
      - "8080:8080"
      - "8081:8081"
```

### Kubernetes ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: squirrel-config
data:
  MCP_ENV: "production"
  SQUIRREL_HTTP_PORT: "8080"
  SQUIRREL_LOG_LEVEL: "info"
---
apiVersion: v1
kind: Secret
metadata:
  name: squirrel-secrets
type: Opaque
stringData:
  DATABASE_URL: "postgresql://user:pass@postgres:5432/squirrel"
  OPENAI_API_KEY: "sk-..."
  SQUIRREL_JWT_SECRET: "your-secret-here"
```

### Docker Build-time vs Runtime

```dockerfile
FROM rust:1.73 as builder

# Build-time environment (compilation)
ENV RUST_LOG=info

# ... build steps ...

FROM debian:bookworm-slim

# Runtime environment (configuration)
ENV MCP_ENV=production
ENV SQUIRREL_HTTP_PORT=8080

# These should come from docker-compose or k8s
# ENV DATABASE_URL=...
# ENV OPENAI_API_KEY=...

COPY --from=builder /app/target/release/squirrel /usr/local/bin/

CMD ["squirrel"]
```

---

## Best Practices

### 1. Never Commit Secrets

❌ **Bad**:
```bash
# .env.production (committed to git)
OPENAI_API_KEY=sk-real-key-here
```

✅ **Good**:
```bash
# .env.production.example (committed to git)
OPENAI_API_KEY=sk-...

# .env.production (in .gitignore)
OPENAI_API_KEY=sk-real-key-here
```

### 2. Use Environment-Specific Validation

```rust
let env = Environment::from_env();
let config = ConfigLoader::load()?;

if env.is_production() {
    // Strict validation in production
    assert!(config.security.enabled, "Security required in production");
    assert!(config.security.jwt_secret.is_some(), "JWT secret required");
}
```

### 3. Provide Clear Defaults

```rust
// ✅ Good: Clear default with fallback
let port = env::var("SQUIRREL_HTTP_PORT")
    .unwrap_or_else(|_| "8080".to_string())
    .parse::<u16>()
    .unwrap_or(8080);

// ❌ Bad: Fails silently
let port = env::var("SQUIRREL_HTTP_PORT")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap();
```

### 4. Document Required Variables

```rust
/// Configuration that requires environment variables
///
/// # Required Environment Variables
///
/// - `DATABASE_URL`: PostgreSQL connection string
/// - `OPENAI_API_KEY`: OpenAI API key (production only)
/// - `SQUIRREL_JWT_SECRET`: JWT secret (32+ characters)
///
/// # Optional Environment Variables
///
/// - `SQUIRREL_HTTP_PORT`: HTTP port (default: 8080)
/// - `MCP_TIMEOUT_SECS`: Timeout in seconds (default: 30)
pub fn load_config() -> Result<Config> {
    ConfigLoader::load()
}
```

### 5. Validate Early

```rust
fn main() -> Result<()> {
    // Load and validate configuration at startup
    let config = ConfigLoader::load()?;
    
    // Additional validation
    if config.network.http_port == config.network.websocket_port {
        return Err("HTTP and WebSocket ports must be different".into());
    }
    
    // Start server only after validation passes
    start_server(config)?;
    Ok(())
}
```

---

## Testing with Environments

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_development_config() {
        env::set_var("MCP_ENV", "development");
        let env = Environment::from_env();
        assert!(env.is_development());
    }

    #[test]
    fn test_custom_port() {
        env::set_var("SQUIRREL_HTTP_PORT", "9000");
        let config = ConfigLoader::load().unwrap();
        assert_eq!(config.network.http_port, 9000);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_production_config() {
    // Set up test environment
    env::set_var("MCP_ENV", "production");
    env::set_var("SQUIRREL_JWT_SECRET", "test-secret-with-32-characters!");
    env::set_var("DATABASE_URL", "postgresql://localhost/test");
    
    let config = ConfigLoader::load().unwrap();
    
    // Verify production settings
    assert_eq!(config.environment, Environment::Production);
    assert!(config.security.enabled);
}
```

---

## Troubleshooting

### Problem: Environment Variables Not Loading

**Solution**:
```bash
# Check if variable is set
echo $SQUIRREL_HTTP_PORT

# Check spelling (case-sensitive!)
export SQUIRREL_HTTP_PORT=8080  # ✅
export squirrel_http_port=8080  # ❌

# Check if using correct prefix
export SQUIRREL_HTTP_PORT=8080  # ✅ (ConfigLoader uses SQUIRREL_)
export MCP_HTTP_PORT=8080       # ❌ (wrong prefix)
```

### Problem: Configuration File Not Found

**Solution**:
```bash
# Check current directory
pwd

# Config loader looks in:
# - ./squirrel.toml
# - ./config/squirrel.toml

# Create config directory if needed
mkdir -p config
cp squirrel.toml config/
```

### Problem: Wrong Environment Detected

**Solution**:
```bash
# Check MCP_ENV
echo $MCP_ENV

# Valid values: development, testing, staging, production
# Or: dev, test, stage, prod

# Fix
export MCP_ENV=production
```

---

## Migration Guide

### From Hardcoded Values

**Before**:
```rust
let port = 8080;
let timeout = Duration::from_secs(30);
```

**After**:
```rust
let config = ConfigLoader::load()?;
let port = config.network.http_port;
let timeout = config.timeouts.connection_timeout();
```

### From Custom Environment Loading

**Before**:
```rust
let port = env::var("PORT")
    .unwrap_or("8080".to_string())
    .parse()?;
```

**After**:
```rust
// ConfigLoader handles this automatically
let config = ConfigLoader::load()?;
let port = config.network.http_port;

// Or use SQUIRREL_HTTP_PORT environment variable
```

---

## Further Reading

- [ADR-008: Configuration Standardization](../../docs/adr/ADR-008-configuration-standardization.md)
- [VALIDATION_GUIDE.md](./VALIDATION_GUIDE.md)
- [12-Factor App Methodology](https://12factor.net/config)
- [ConfigLoader API Docs](src/unified/loader.rs)

---

**Questions or issues?** Check the inline documentation or see the comprehensive test suite for examples.

