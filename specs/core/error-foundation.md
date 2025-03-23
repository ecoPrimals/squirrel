---
title: Error Foundation Specification
version: 1.0.0
date: 2024-05-15
status: active
priority: high
---

# Error Foundation Specification

## Overview

This specification defines the foundational error system used throughout the Squirrel ecosystem. It outlines the core error types, patterns, and principles that all components should follow when handling errors. The error foundation is intentionally kept minimal but comprehensive enough to support the entire system.

## Core Error Types

### SquirrelError

The root error type for the entire ecosystem:

```rust
pub enum SquirrelError {
    /// App initialization errors
    AppInitialization(AppInitializationError),
    /// App operation errors
    AppOperation(AppOperationError),
    /// Generic error with message
    Generic(String),
    /// IO errors
    IO(std::io::Error),
    /// Security-related errors
    Security(String),
    /// MCP module errors
    MCP(String),
    /// Other errors
    Other(String),
    /// Health monitoring errors
    Health(String),
    /// Metric collection errors
    Metric(String),
    /// Dashboard errors
    Dashboard(String),
    /// Serialization errors
    Serialization(String),
    /// Network monitoring errors
    Network(String),
    /// Alert errors
    Alert(String),
    /// Session-related errors
    Session(String),
    /// Persistence errors
    Persistence(PersistenceError),
    /// Protocol version errors
    ProtocolVersion(String),
    /// Context-related errors
    Context(String),
}
```

### Specific Error Types

The error system includes specialized error types for common scenarios:

#### AppInitializationError

```rust
pub enum AppInitializationError {
    /// The application has already been initialized
    AlreadyInitialized,
    /// Invalid configuration
    InvalidConfiguration(String),
    /// Failed to load resources
    ResourceLoadFailure(String),
}
```

#### AppOperationError

```rust
pub enum AppOperationError {
    /// The application has not been initialized
    NotInitialized,
    /// Operation is not supported
    UnsupportedOperation(String),
    /// Failed to complete operation
    OperationFailure(String),
    /// The application is already started
    AlreadyStarted,
    /// The application is already stopped
    AlreadyStopped,
    /// The application is not started
    NotStarted,
}
```

#### PersistenceError

```rust
pub enum PersistenceError {
    /// IO error
    IO(String),
    /// Configuration error
    Config(String),
    /// Storage error
    Storage(String),
    /// Format error
    Format(String),
}
```

#### AlertError

```rust
pub enum AlertError {
    /// Configuration-related errors in the alert system
    Configuration(String),
    /// Errors related to sending notifications
    Notification(String),
    /// Internal errors within the alert system
    Internal(String),
}
```

### Error Type Categories

Each major subsystem has its own error category in the `SquirrelError` enum. This allows for:

1. Clear identification of error sources
2. Systematic error handling based on category
3. Consistent error reporting across the system

### Implementation Requirements

Core error types must:

1. Implement the `std::error::Error` trait
2. Provide clear, human-readable error messages via `Display`
3. Support error cause chaining when appropriate
4. Be `Send` and `Sync` to work in concurrent contexts

## Error Creation Patterns

### Factory Methods

The error system provides factory methods to ensure consistent error creation:

```rust
impl SquirrelError {
    /// Create a new security error
    pub fn security(msg: impl Into<String>) -> Self {
        Self::Security(msg.into())
    }

    /// Create a new MCP error
    pub fn mcp(msg: impl Into<String>) -> Self {
        Self::MCP(msg.into())
    }

    /// Create a new generic error
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    /// Create a new other error
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }

    /// Create a new health error
    pub fn health(msg: impl Into<String>) -> Self {
        Self::Health(msg.into())
    }

    /// Create a new metric error
    pub fn metric(msg: impl Into<String>) -> Self {
        Self::Metric(msg.into())
    }

    /// Create a new dashboard error
    pub fn dashboard(msg: impl Into<String>) -> Self {
        Self::Dashboard(msg.into())
    }

    /// Create a new serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// Create a new network error
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    /// Create a new alert error
    pub fn alert(msg: impl Into<String>) -> Self {
        Self::Alert(msg.into())
    }

    /// Create a new session error
    pub fn session(msg: impl Into<String>) -> Self {
        Self::Session(msg.into())
    }

    /// Create a new protocol version error
    pub fn protocol_version(msg: impl Into<String>) -> Self {
        Self::ProtocolVersion(msg.into())
    }

    /// Create a new context error
    pub fn context(msg: impl Into<String>) -> Self {
        Self::Context(msg.into())
    }

    /// Create a new monitoring error
    pub fn monitoring(msg: impl Into<String>) -> Self {
        Self::Metric(msg.into())
    }
}
```

### From Trait Implementations

The error system implements `From` traits for common conversion scenarios:

```rust
impl From<std::io::Error> for SquirrelError { /* ... */ }
impl From<AppInitializationError> for SquirrelError { /* ... */ }
impl From<AppOperationError> for SquirrelError { /* ... */ }
impl From<String> for SquirrelError { /* ... */ }
impl From<&str> for SquirrelError { /* ... */ }
impl From<serde_json::Error> for SquirrelError { /* ... */ }
impl From<PersistenceError> for SquirrelError { /* ... */ }
impl From<AlertError> for SquirrelError { /* ... */ }
```

## Error Result Type

A common `Result` type is used throughout the ecosystem:

```rust
/// Result type alias for `SquirrelError`
pub type Result<T> = std::result::Result<T, SquirrelError>;
```

## Error Recovery and Handling

### Error Recovery Detection

The error system provides utilities to determine if errors are recoverable:

```rust
impl SquirrelError {
    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            SquirrelError::IO(_) | SquirrelError::Generic(_) | SquirrelError::AppInitialization(_) => false,
            SquirrelError::AppOperation(e) => {
                !matches!(e, AppOperationError::NotInitialized)
            },
            _ => true,
        }
    }
}
```

### Error Handling Principles

1. **Public Boundaries**: All public functions should return `Result` unless failure is impossible
2. **Internal Operations**: Internal functions can use `Result` or other error handling as appropriate
3. **Performance-Critical Paths**: Consider alternatives like `Option` for performance-critical code

### Error Translation

When crossing module boundaries:

1. Map errors to the appropriate SquirrelError variant
2. Preserve original error context when possible
3. Add boundary-specific context when helpful

```rust
// Example: Mapping errors at boundary
pub fn read_config_file(path: &Path) -> Result<Config> {
    std::fs::read_to_string(path)
        .map_err(|e| SquirrelError::IO(e))
        .and_then(|s| parse_config(&s))
}
```

### Error Logging

1. Log errors at the point of handling, not at the point of creation
2. Include all relevant context in the log message
3. Use appropriate log levels based on error severity

## Plugin-Specific Errors

While the core error system defines the foundation, plugins should:

1. Define domain-specific error types for their own internal use
2. Map to `SquirrelError` at plugin boundaries
3. Provide clear context when mapping errors

```rust
// Example: Plugin-specific error mapping
impl From<MyPluginError> for SquirrelError {
    fn from(err: MyPluginError) -> Self {
        match err {
            MyPluginError::Configuration(msg) => 
                SquirrelError::AppOperation(AppOperationError::OperationFailure(format!("Plugin configuration error: {}", msg))),
            MyPluginError::Resource(msg) => 
                SquirrelError::Generic(format!("Plugin resource error: {}", msg)),
            // ... other mappings
        }
    }
}
```

## Backwards Compatibility

The error system should maintain backwards compatibility:

1. New error variants can be added, but existing ones should not be removed
2. Error factory methods should remain stable
3. Error translation patterns should be consistent

## Implementation Status

- Core error types: **Implemented**
- Factory methods: **Implemented**
- Error recovery detection: **Implemented**
- Result type alias: **Implemented**
- From trait implementations: **Implemented**
- Plugin error mappings: **Partially Implemented**

## Future Enhancements

While maintaining minimalism, future enhancements may include:

1. Structured error metadata for improved diagnostics
2. Error code system for machine-readable error identification
3. Enhanced context chain for better debugging
4. Error categorization for improved handling strategies

## Conclusion

The error foundation provides a minimal but essential framework for consistent error handling across the Squirrel ecosystem. By following these patterns and principles, components can achieve robust error handling with clear error communication while keeping the core crate focused on its essential responsibilities. 