---
description: Standard Error Handling pattern for the Squirrel codebase
version: 1.0.0
last_updated: 2024-03-21
status: active
---

# Error Handling Pattern

## Context

Consistent error handling is crucial for building reliable and maintainable software. This pattern outlines the standard approach to error handling in the Squirrel codebase, ensuring errors are properly defined, propagated, and handled throughout the application.

This pattern should be used when:
- Defining new error types for a module or crate
- Propagating errors across function boundaries
- Converting between different error types
- Handling errors at appropriate boundaries
- Providing meaningful error context to users and logs

## Implementation

### Error Type Definition

```rust
use thiserror::Error;
use std::io;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("Failed to create context: {0}")]
    Creation(String),
    
    #[error("Context not found: {id}")]
    NotFound { id: String },
    
    #[error("Invalid context structure: {0}")]
    InvalidStructure(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}
```

### Error Propagation

Using the `?` operator for clean error propagation:

```rust
pub async fn load_context(id: &str) -> Result<Context, ContextError> {
    let path = context_path(id)?;
    let content = tokio::fs::read_to_string(path).await?;
    let context: Context = serde_json::from_str(&content)?;
    
    if !context.is_valid() {
        return Err(ContextError::InvalidStructure(
            "Context fails validation rules".to_string()
        ));
    }
    
    Ok(context)
}
```

### Error Context and Conversion

Using `map_err` for adding context or converting between error types:

```rust
pub async fn process_context(id: &str) -> Result<ProcessedContext, ProcessingError> {
    let context = load_context(id)
        .await
        .map_err(|e| ProcessingError::LoadFailed(e.to_string()))?;
        
    // Process the context...
    
    Ok(ProcessedContext::new(/* ... */))
}
```

### Error Handling at Boundaries

```rust
async fn handle_request(req: Request) -> Response {
    match process_request(req).await {
        Ok(result) => Response::success(result),
        Err(e) => {
            // Log the error
            tracing::error!("Request processing failed: {}", e);
            
            // Convert to appropriate response
            match e {
                ProcessingError::NotFound(_) => Response::not_found(&e.to_string()),
                ProcessingError::InvalidInput(_) => Response::bad_request(&e.to_string()),
                _ => Response::internal_error("An unexpected error occurred"),
            }
        }
    }
}
```

### Error Context Propagation

For more complex scenarios, use `anyhow` with context information:

```rust
use anyhow::{Context, Result};

fn complex_operation() -> Result<Output> {
    let config = load_config()
        .context("Failed to load configuration")?;
        
    let data = process_input(&config)
        .context("Failed to process input data")?;
        
    let result = generate_output(data)
        .context("Failed to generate output")?;
        
    Ok(result)
}
```

## Benefits

- **Clarity**: Error types clearly document what can go wrong
- **Type Safety**: Compiler ensures errors are properly handled
- **Context**: Errors include helpful context information
- **Propagation**: Easy to propagate and transform errors
- **Consistency**: Uniform error handling across the codebase
- **Testability**: Error conditions can be reliably tested

## Tradeoffs

- **Verbosity**: Defining error types requires more code
- **Complexity**: Error conversion can add complexity
- **Overhead**: Some performance overhead for rich error types
- **Learning Curve**: Requires understanding Rust's error handling patterns
- **Balance**: Need to balance between too generic and too specific errors

## When to Use

- When implementing public API functions
- When creating a new module with its own error types
- When errors need to cross module boundaries
- When error information needs to be preserved during propagation
- When specific error types are needed for proper handling

## When to Avoid

- For simple internal utilities with limited error cases
- When performance is absolutely critical (consider simpler error types)
- When errors are immediately handled and never propagated

## Related Patterns

- [Result Type Pattern](./result-type.md)
- [Logging Pattern](./logging.md)
- [API Response Pattern](./api-response.md)

## Examples in Codebase

- `crates/context/src/error.rs`: Context subsystem error types
- `crates/mcp/src/protocol/error.rs`: MCP protocol error handling
- `crates/commands/src/error.rs`: Command system error types

## Testing Approach

Error handling should be thoroughly tested:

```rust
#[test]
fn test_load_context_not_found() {
    let result = block_on(load_context("nonexistent"));
    
    assert!(matches!(
        result, 
        Err(ContextError::NotFound { id }) if id == "nonexistent"
    ));
}

#[test]
fn test_load_context_invalid() {
    // Setup invalid context file
    let context_path = setup_invalid_context();
    
    let result = block_on(load_context("invalid"));
    
    assert!(matches!(result, Err(ContextError::InvalidStructure(_))));
}
```

## Security Considerations

- Avoid exposing sensitive information in error messages
- Consider using different error details for internal logging vs. external responses
- Implement proper error sanitization at API boundaries
- Be cautious about error information that might reveal system details

## Performance Characteristics

- Time complexity: O(1) for error creation and propagation
- Space complexity: O(1) per error type
- Memory usage: Low to moderate depending on error context
- CPU usage: Minimal for most error operations

## Migration Guide

When migrating from ad-hoc error handling:

1. Define domain-specific error enums using `thiserror`
2. Update functions to return appropriate `Result` types
3. Implement error conversion between different modules
4. Add context information to errors where helpful
5. Update error handling at boundaries to use new error types

## Version History

- 1.0.0 (2024-03-21): Initial version 