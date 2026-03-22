# Configuration Validation Guide

**Date**: November 10, 2025 (updated March 22, 2026)  
**Status**: Active

---

## Overview

The Squirrel configuration system now includes a unified validation module that provides reusable validation functions for common configuration patterns. This guide explains how to use the validation system effectively.

## Quick Start

```rust
use squirrel_mcp_config::unified::validation::{Validator, ValidationResult};

// Validate a port
let port = Validator::validate_port(8080)?;

// Validate a timeout
let timeout = Validator::validate_timeout_secs(30, "connection_timeout")?;

// Validate a hostname
Validator::validate_hostname("example.com")?;

// Validate that ports differ
Validator::validate_ports_differ(8080, 8081, "HTTP", "WebSocket")?;
```

## Validation Categories

### 1. Port Validation

```rust
// Basic port validation (1-65535)
Validator::validate_port(8080)?;

// Ensure two ports are different
Validator::validate_ports_differ(
    8080, 
    8081, 
    "HTTP", 
    "WebSocket"
)?;
```

**Use Cases**:
- Validating HTTP, WebSocket, and monitoring ports
- Ensuring service ports don't conflict

### 2. Timeout Validation

```rust
// Basic timeout validation (> 0)
Validator::validate_timeout_secs(30, "connection_timeout")?;

// Timeout with maximum value
Validator::validate_timeout_with_max(
    10, 
    30, 
    "health_check_timeout"
)?;

// Ensure timeout ordering (A < B)
Validator::validate_timeout_ordering(
    10, 
    30, 
    "connect_timeout", 
    "request_timeout"
)?;
```

**Use Cases**:
- Connection, request, and session timeouts
- Health check intervals
- Retry delays

### 3. Network Validation

```rust
// Validate IP address
let ip = Validator::validate_ip_address("127.0.0.1")?;

// Validate hostname (RFC 1123)
Validator::validate_hostname("example.com")?;

// Validate URL scheme
Validator::validate_url_scheme(
    "https://api.example.com", 
    &["http", "https"]
)?;
```

**Use Cases**:
- Bind addresses and public addresses
- API endpoints and service URLs
- DNS configuration

### 4. File System Validation

```rust
use std::path::Path;

// Validate file exists
Validator::validate_file_exists(Path::new("cert.pem"), "certificate")?;

// Validate directory exists
Validator::validate_dir_exists(Path::new("/var/log"), "log_directory")?;

// Validate parent directory exists (for new files)
Validator::validate_parent_dir_exists(
    Path::new("/var/log/app.log"), 
    "log_file"
)?;
```

**Use Cases**:
- TLS certificates and keys
- Configuration file paths
- Log and data directories

### 5. String Validation

```rust
// Validate non-empty string
Validator::validate_not_empty("my-service", "service_name")?;

// Validate alphanumeric with allowed symbols
Validator::validate_alphanumeric_with(
    "my-service-name", 
    "service_name", 
    &['-', '_']
)?;

// Validate semantic version
Validator::validate_semver("1.2.3")?;
```

**Use Cases**:
- Service names and identifiers
- Version strings
- User input sanitization

### 6. Numeric Validation

```rust
// Validate greater than minimum
let value = Validator::validate_greater_than(10, 0, "max_connections")?;

// Validate within range
let cpu = Validator::validate_range(
    75.0, 
    0.0, 
    100.0, 
    "cpu_percent"
)?;
```

**Use Cases**:
- Resource limits (memory, CPU, connections)
- Percentage values
- Rate limits

### 7. Security Validation

```rust
// Validate API key length
Validator::validate_api_key("sk-1234567890...", 10, "openai_api_key")?;

// Validate JWT secret length
Validator::validate_jwt_secret("my-very-long-secret-key-here")?;
```

**Use Cases**:
- API key configuration
- JWT and encryption secrets
- Credential validation

## Error Handling

All validation functions return `ValidationResult<T>`, which is a `Result<T, ValidationError>`:

```rust
use squirrel_mcp_config::unified::validation::{Validator, ValidationError};

match Validator::validate_port(port) {
    Ok(valid_port) => {
        println!("Port {} is valid", valid_port);
    }
    Err(ValidationError::Constraint { field, constraint }) => {
        eprintln!("Invalid {}: must be {}", field, constraint);
    }
    Err(e) => {
        eprintln!("Validation error: {}", e);
    }
}
```

### Validation Error Types

```rust
pub enum ValidationError {
    Invalid { field: String, reason: String },
    Missing { field: String },
    Constraint { field: String, constraint: String },
    Conflict { description: String },
    FileNotFound { path: String },
}
```

## Integration Examples

### Example 1: Validating a Complete Config Struct

```rust
use squirrel_mcp_config::unified::validation::{Validator, ValidationResult};

#[derive(Debug)]
pub struct ServerConfig {
    pub port: u16,
    pub timeout_secs: u64,
    pub hostname: String,
}

impl ServerConfig {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate port
        if let Err(e) = Validator::validate_port(self.port) {
            errors.push(format!("Port: {}", e));
        }

        // Validate timeout
        if let Err(e) = Validator::validate_timeout_secs(self.timeout_secs, "timeout") {
            errors.push(format!("Timeout: {}", e));
        }

        // Validate hostname
        if let Err(e) = Validator::validate_hostname(&self.hostname) {
            errors.push(format!("Hostname: {}", e));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

### Example 2: Builder Pattern with Validation

```rust
pub struct ConfigBuilder {
    port: Option<u16>,
    timeout: Option<u64>,
}

impl ConfigBuilder {
    pub fn port(mut self, port: u16) -> Result<Self, ValidationError> {
        Validator::validate_port(port)?;
        self.port = Some(port);
        Ok(self)
    }

    pub fn timeout(mut self, timeout: u64) -> Result<Self, ValidationError> {
        Validator::validate_timeout_secs(timeout, "timeout")?;
        self.timeout = Some(timeout);
        Ok(self)
    }

    pub fn build(self) -> ServerConfig {
        ServerConfig {
            port: self.port.unwrap_or(8080),
            timeout: self.timeout.unwrap_or(30),
        }
    }
}
```

### Example 3: Cross-Field Validation

```rust
pub struct NetworkConfig {
    pub http_port: u16,
    pub https_port: u16,
    pub admin_port: u16,
}

impl NetworkConfig {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate individual ports
        for (port, name) in [
            (self.http_port, "HTTP"),
            (self.https_port, "HTTPS"),
            (self.admin_port, "Admin"),
        ] {
            if let Err(e) = Validator::validate_port(port) {
                errors.push(format!("{} port: {}", name, e));
            }
        }

        // Validate ports differ
        if let Err(e) = Validator::validate_ports_differ(
            self.http_port, 
            self.https_port, 
            "HTTP", 
            "HTTPS"
        ) {
            errors.push(e.to_string());
        }

        if let Err(e) = Validator::validate_ports_differ(
            self.http_port, 
            self.admin_port, 
            "HTTP", 
            "Admin"
        ) {
            errors.push(e.to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

## Best Practices

### 1. **Validate Early**
Validate configuration as soon as it's loaded, before starting services:

```rust
let config = ConfigLoader::load()?;
// Validation happens automatically in ConfigLoader::load()
// but you can also call it explicitly:
config.validate()?;
```

### 2. **Provide Context**
Always provide descriptive field names in validation calls:

```rust
// Good
Validator::validate_timeout_secs(timeout, "database_connection_timeout")?;

// Not as helpful
Validator::validate_timeout_secs(timeout, "timeout")?;
```

### 3. **Collect All Errors**
Use `Vec<String>` to collect multiple validation errors:

```rust
pub fn validate(&self) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Validate multiple fields
    if let Err(e) = Validator::validate_port(self.port) {
        errors.push(format!("Port: {}", e));
    }
    if let Err(e) = Validator::validate_hostname(&self.host) {
        errors.push(format!("Hostname: {}", e));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

### 4. **Use Type System**
Combine validation with Rust's type system:

```rust
// Create validated types
pub struct ValidatedPort(u16);

impl ValidatedPort {
    pub fn new(port: u16) -> ValidationResult<Self> {
        Validator::validate_port(port)?;
        Ok(Self(port))
    }

    pub fn get(&self) -> u16 {
        self.0
    }
}
```

### 5. **Document Validation Rules**
Document what validation is performed in your config structs:

```rust
/// Server configuration
///
/// # Validation
/// - `port` must be 1-65535
/// - `timeout_secs` must be > 0
/// - `max_connections` must be > 0
pub struct ServerConfig {
    pub port: u16,
    pub timeout_secs: u64,
    pub max_connections: usize,
}
```

## Testing

The validation module includes comprehensive tests. To run them:

```bash
cargo test -p squirrel-mcp-config --lib validation
```

To test your own validation logic:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use squirrel_mcp_config::unified::validation::Validator;

    #[test]
    fn test_valid_config() {
        let config = ServerConfig {
            port: 8080,
            timeout_secs: 30,
            hostname: "example.com".to_string(),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_port() {
        let config = ServerConfig {
            port: 0, // Invalid
            timeout_secs: 30,
            hostname: "example.com".to_string(),
        };
        assert!(config.validate().is_err());
    }
}
```

## Migration from Legacy Validators

If you're migrating from legacy validation code:

### Before:
```rust
if port == 0 {
    return Err("Port must be > 0".to_string());
}
```

### After:
```rust
Validator::validate_port(port)
    .map_err(|e| e.to_string())?;
```

### Benefits:
- Consistent error messages
- Reusable validation logic
- Comprehensive test coverage
- Clear error types

## Further Reading

- [Configuration Module API Docs](src/unified/validation.rs)
- [SquirrelUnifiedConfig](src/unified/types/definitions.rs)
- [ConfigLoader](src/unified/loader.rs)

---

**Questions or issues?** Check the inline documentation in `src/unified/validation.rs` or see the comprehensive test suite for examples.

