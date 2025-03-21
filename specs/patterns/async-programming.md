---
description: Standard Async Programming pattern for the Squirrel codebase
version: 1.0.0
last_updated: 2024-03-21
status: active
---

# Async Programming Pattern

## Context

Asynchronous programming is essential for creating highly concurrent and efficient applications that can handle many operations simultaneously without blocking. This pattern outlines the standard approach to async programming in the Squirrel codebase, ensuring consistent and safe use of Rust's async/await features.

This pattern should be used when:
- Implementing I/O-bound operations (file, network, database access)
- Working with operations that involve waiting for external resources
- Building services that need to handle multiple requests concurrently
- Managing complex workflows with parallel operations
- Optimizing resource usage for long-running operations

## Implementation

### Async Function Definition

```rust
use tokio::fs;
use std::path::Path;
use squirrel_context::Context;

/// Loads a context from the filesystem asynchronously
pub async fn load_context(path: impl AsRef<Path>) -> Result<Context, ContextError> {
    let content = fs::read_to_string(path).await?;
    let context: Context = serde_json::from_str(&content)?;
    Ok(context)
}
```

### Async Trait Implementation

Using `async-trait` for async methods in traits:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait ContextManager: Send + Sync + 'static {
    /// Creates a new context
    async fn create_context(&self, request: ContextRequest) -> Result<Context, ContextError>;
    
    /// Updates an existing context
    async fn update_context(&self, context_id: &str, update: ContextUpdate) -> Result<(), ContextError>;
    
    /// Retrieves a context by ID
    async fn get_context(&self, context_id: &str) -> Result<Context, ContextError>;
}
```

### Concurrent Operations

Using `tokio::spawn` for concurrent operations:

```rust
pub async fn process_contexts(context_ids: &[String]) -> Result<Vec<ProcessedContext>, ProcessingError> {
    let mut handles = Vec::with_capacity(context_ids.len());
    
    for id in context_ids {
        let id = id.clone();
        let handle = tokio::spawn(async move {
            process_single_context(&id).await
        });
        handles.push(handle);
    }
    
    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        match handle.await {
            Ok(Ok(result)) => results.push(result),
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(ProcessingError::TaskJoinError(e.to_string())),
        }
    }
    
    Ok(results)
}
```

### Timeouts

Adding timeouts to async operations:

```rust
use tokio::time::{timeout, Duration};

pub async fn get_context_with_timeout(
    context_id: &str,
    timeout_duration: Duration,
) -> Result<Context, ContextError> {
    match timeout(timeout_duration, get_context(context_id)).await {
        Ok(result) => result,
        Err(_) => Err(ContextError::Timeout(format!("Operation timed out after {:?}", timeout_duration))),
    }
}
```

### Cancellation

Supporting cancellation with `tokio::select!`:

```rust
use tokio::select;

pub async fn process_with_cancellation(
    context_id: &str,
    cancel_token: CancellationToken,
) -> Result<ProcessedContext, ProcessingError> {
    select! {
        result = process_context(context_id) => result,
        _ = cancel_token.cancelled() => {
            Err(ProcessingError::Cancelled("Operation was cancelled".to_string()))
        }
    }
}
```

### Resource Management

Using semaphores for resource limiting:

```rust
use tokio::sync::Semaphore;

/// Process contexts with a limit on concurrent operations
pub async fn process_with_limit(
    context_ids: &[String],
    max_concurrent: usize,
) -> Result<Vec<ProcessedContext>, ProcessingError> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut handles = Vec::with_capacity(context_ids.len());
    
    for id in context_ids {
        let id = id.clone();
        let permit_semaphore = semaphore.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = permit_semaphore.acquire().await.unwrap();
            process_single_context(&id).await
        });
        
        handles.push(handle);
    }
    
    // Collect results as before
    // ...
}
```

## Benefits

- **Concurrency**: Efficiently handle many operations simultaneously
- **Scalability**: Serve more requests with the same resources
- **Responsiveness**: Avoid blocking the main thread
- **Resource Utilization**: Maximize CPU and I/O utilization
- **Clarity**: Async/await syntax maintains code readability
- **Composition**: Easily compose async operations

## Tradeoffs

- **Complexity**: Async code can be more complex to reason about
- **Error Handling**: Error propagation requires careful design
- **Stack Traces**: Less helpful stack traces for debugging
- **Learning Curve**: Requires understanding of async concepts
- **Ecosystem Compatibility**: Not all libraries support async

## When to Use

- For I/O-bound operations (file, network, database)
- When handling many concurrent operations
- For long-running operations that shouldn't block
- When implementing high-throughput services
- When composing multiple asynchronous operations

## When to Avoid

- For CPU-bound operations (use threads instead)
- For simple, fast operations where async overhead isn't justified
- When synchronous alternatives provide better simplicity
- When working with libraries that don't support async well

## Related Patterns

- [Error Handling Pattern](./error-handling.md)
- [Resource Management Pattern](./resource-management.md)
- [Cancellation Pattern](./cancellation.md)

## Examples in Codebase

- `crates/context/src/manager.rs`: Async context management
- `crates/mcp/src/protocol/handler.rs`: Async protocol handling
- `crates/app/src/service.rs`: Async service implementation

## Testing Approach

Testing async code requires special consideration:

```rust
#[tokio::test]
async fn test_load_context() {
    // Setup test context
    let temp_dir = tempfile::tempdir().unwrap();
    let context_path = temp_dir.path().join("test-context.json");
    
    // Write test data
    let test_context = Context::new("test");
    let json = serde_json::to_string(&test_context).unwrap();
    tokio::fs::write(&context_path, json).await.unwrap();
    
    // Test the function
    let result = load_context(&context_path).await;
    
    // Assertions
    assert!(result.is_ok());
    let loaded_context = result.unwrap();
    assert_eq!(loaded_context.id(), "test");
}
```

## Security Considerations

- Ensure proper resource limiting to prevent DoS attacks
- Implement timeouts for all external service calls
- Handle cancellation gracefully to prevent resource leaks
- Consider using dedicated thread pools for security-sensitive operations
- Ensure async operations maintain proper security context

## Performance Characteristics

- **Scalability**: Can handle thousands of concurrent operations
- **Overhead**: Small overhead for task creation and management
- **Memory Usage**: Reduced memory compared to thread-per-request
- **Context Switching**: Minimal compared to OS thread switching
- **Resource Contention**: Need careful design for shared resources

## Migration Guide

When migrating from synchronous code:

1. Identify I/O-bound operations that would benefit from async
2. Convert functions to async using `async` keyword
3. Update function calls to use `.await`
4. Update trait implementations to use `#[async_trait]`
5. Ensure proper error propagation with `?` operator
6. Add appropriate timeouts and resource limits
7. Update tests to use async test frameworks

## Version History

- 1.0.0 (2024-03-21): Initial version 