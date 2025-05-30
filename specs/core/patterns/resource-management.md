# Resource Management Pattern

## Context

Effective resource management is critical for building reliable applications, especially in systems like Squirrel that may handle many concurrent operations with limited resources. This pattern describes the recommended approach to managing resources in the Squirrel codebase, such as file handles, network connections, memory allocations, threads, and external services.

### When to Use This Pattern

- When working with limited or costly resources
- When resources require explicit cleanup or release
- When implementing concurrent or parallel operations
- When resource contention might become a bottleneck
- When implementing components that require graceful shutdown

## Implementation

The Resource Management Pattern in Squirrel follows several core principles:

### 1. RAII (Resource Acquisition Is Initialization)

```rust
// Example: File resource management using RAII
fn process_file(path: &str) -> Result<String, Error> {
    // Resource acquisition tied to object initialization
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    
    // Process the file...
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    
    // No explicit cleanup needed - Drop trait handles cleanup
    Ok(contents)
}
```

### 2. Explicit Resource Pools

```rust
// Example: Connection pool implementation
pub struct ConnectionPool {
    available: Arc<Mutex<Vec<Connection>>>,
    max_size: usize,
    connection_semaphore: Arc<Semaphore>,
}

impl ConnectionPool {
    pub fn new(max_size: usize, connection_string: &str) -> Self {
        let available = Arc::new(Mutex::new(Vec::with_capacity(max_size)));
        let connection_semaphore = Arc::new(Semaphore::new(max_size));
        
        // Pre-initialize connections
        let mut connections = Vec::with_capacity(max_size);
        for _ in 0..max_size {
            connections.push(Connection::new(connection_string));
        }
        
        *available.lock().unwrap() = connections;
        
        Self {
            available,
            max_size,
            connection_semaphore,
        }
    }
    
    pub async fn get(&self) -> Result<PooledConnection, Error> {
        // Wait for an available connection
        let permit = self.connection_semaphore.acquire().await?;
        
        // Get a connection from the pool
        let connection = {
            let mut available = self.available.lock().unwrap();
            available.pop().ok_or(Error::NoAvailableConnections)?
        };
        
        // Return a pooled connection that will return itself when dropped
        Ok(PooledConnection::new(connection, self.available.clone(), permit))
    }
}

pub struct PooledConnection {
    connection: Option<Connection>,
    pool: Arc<Mutex<Vec<Connection>>>,
    _permit: OwnedSemaphorePermit,
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            let mut pool = self.pool.lock().unwrap();
            pool.push(conn);
        }
    }
}
```

### 3. Timeouts and Deadlines

```rust
// Example: Implementing timeouts for resource acquisition
pub async fn acquire_with_timeout<T>(
    resource_future: impl Future<Output = Result<T, Error>>,
    timeout: Duration,
) -> Result<T, Error> {
    tokio::time::timeout(timeout, resource_future)
        .await
        .map_err(|_| Error::AcquisitionTimeout)?
}

// Usage example
async fn process_with_timeout() -> Result<(), Error> {
    let connection = acquire_with_timeout(pool.get(), Duration::from_secs(5)).await?;
    // Use connection...
    Ok(())
}
```

### 4. Graceful Resource Cleanup

```rust
// Example: Implementing graceful shutdown
pub struct ResourceManager {
    resources: Arc<Mutex<Vec<Box<dyn Resource>>>>,
    shutdown_signal: Arc<Notify>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            resources: Arc::new(Mutex::new(Vec::new())),
            shutdown_signal: Arc::new(Notify::new()),
        }
    }
    
    pub fn register<R: Resource + 'static>(&self, resource: R) {
        let mut resources = self.resources.lock().unwrap();
        resources.push(Box::new(resource));
    }
    
    pub async fn shutdown(&self) -> Result<(), Error> {
        // Signal shutdown
        self.shutdown_signal.notify_waiters();
        
        // Release resources in reverse order of acquisition
        let mut resources = self.resources.lock().unwrap();
        while let Some(resource) = resources.pop() {
            resource.release().await?;
        }
        
        Ok(())
    }
    
    pub fn get_shutdown_signal(&self) -> Arc<Notify> {
        self.shutdown_signal.clone()
    }
}

// Usage with tokio tasks
async fn run_with_graceful_shutdown(manager: ResourceManager) {
    let shutdown_signal = manager.get_shutdown_signal();
    
    // Create task
    let task_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = shutdown_signal.notified() => {
                    // Graceful shutdown requested
                    break;
                }
                _ = async_work() => {
                    // Do regular work
                }
            }
        }
        
        // Cleanup before exiting
        println!("Task shutting down gracefully");
    });
    
    // Wait for task to complete
    task_handle.await.unwrap();
}
```

### 5. Resource Limiting

```rust
// Example: Using semaphores for resource limiting
pub struct RateLimitedClient {
    client: Client,
    rate_limiter: Arc<Semaphore>,
}

impl RateLimitedClient {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            client: Client::new(),
            rate_limiter: Arc::new(Semaphore::new(max_concurrent)),
        }
    }
    
    pub async fn request(&self, url: &str) -> Result<Response, Error> {
        // Acquire a permit or wait
        let permit = self.rate_limiter.acquire().await?;
        
        // Make the request
        let response = self.client.get(url).send().await?;
        
        // Permit is automatically released when dropped
        drop(permit);
        
        Ok(response)
    }
}
```

## Benefits

1. **Predictable Resource Cleanup**: Resources are properly released even during errors
2. **Prevention of Resource Leaks**: Structured approach prevents forgetting to release resources
3. **Efficient Resource Utilization**: Pooling and reuse maximize efficiency
4. **Graceful Degradation**: System continues functioning under resource constraints
5. **Improved Reliability**: Proper timeouts and error handling increase system resilience
6. **Better Performance**: Efficient resource management reduces contention and improves throughput

## Tradeoffs

1. **Increased Complexity**: Resource management adds overhead and complexity
2. **Performance Overhead**: Some techniques like locks can introduce performance costs
3. **Learning Curve**: Requires understanding Rust's ownership model and async patterns
4. **Potential for Deadlocks**: Improper resource management can lead to deadlocks

## When to Use

- When working with external resources (files, network, databases)
- When implementing services with strict reliability requirements
- When building components that need to scale with concurrent requests
- When implementing long-running operations that need graceful cancellation

## When to Avoid

- For simple, short-lived applications with minimal resource usage
- When performance is more critical than resource safety
- For prototype code where resource safety is not yet a concern

## Related Patterns

- **Error Handling Pattern**: Combines with resource management for proper cleanup during errors
- **Async Programming Pattern**: Works together for async resource management
- **Factory Pattern**: Often used to create and initialize resources
- **Pool Pattern**: Specifically for managing reusable resources
- **Circuit Breaker Pattern**: For managing external service dependencies

## Examples in Codebase

- `crates/commands/src/resources.rs` - Command resource management
- `crates/context/src/manager.rs` - Context resource handling
- `crates/mcp/src/connection_manager.rs` - Connection pool implementation
- `crates/transport/src/pool.rs` - Transport resource pooling

## Testing Approach

Testing resource management requires verifying proper acquisition, usage, and release cycles:

1. **Resource Acquisition Tests**:
   - Test successful acquisition
   - Test error handling during acquisition
   - Test timeout behavior

2. **Concurrent Usage Tests**:
   - Test multiple concurrent acquisitions
   - Test resource contention scenarios
   - Test fairness in resource distribution

3. **Resource Release Tests**:
   - Verify resources are properly released
   - Test release during normal operation
   - Test release during errors and panics

Example test:

```rust
#[tokio::test]
async fn test_connection_pool() {
    let pool = ConnectionPool::new(5, "connection_string");
    
    // Test successful acquisition
    let conn1 = pool.get().await.unwrap();
    let conn2 = pool.get().await.unwrap();
    assert_eq!(pool.available_connections(), 3);
    
    // Test returning to pool
    drop(conn1);
    assert_eq!(pool.available_connections(), 4);
    
    // Test timeout
    let conns: Vec<_> = (0..4).map(|_| pool.get().await.unwrap()).collect();
    let timeout_result = tokio::time::timeout(
        Duration::from_millis(100),
        pool.get()
    ).await;
    assert!(timeout_result.is_err()); // Should timeout
    
    // Test resource release
    drop(conns);
    assert_eq!(pool.available_connections(), 5);
}
```

## Security Considerations

1. **Resource Exhaustion Attacks**: Implement proper limits to prevent DoS attacks
2. **Sensitive Data in Resources**: Ensure proper cleanup of sensitive information
3. **Privilege Escalation**: Manage resource access based on user permissions
4. **Resource Authentication**: Securely manage credentials for resource access
5. **Logging Resource Usage**: Monitor for abnormal patterns that might indicate attacks

## Performance Characteristics

- **Time Complexity**: O(1) for acquiring pre-initialized resources, O(log n) for pooled resources
- **Space Complexity**: Memory overhead for tracking and pooling resources
- **Latency Impact**: Minimal when properly implemented, most overhead during initialization
- **Throughput**: Can significantly improve under high load with proper pooling
- **Resource Utilization**: Efficient use of system resources through pooling and limiting

## Migration Guide

When migrating from ad-hoc resource management to this pattern:

1. **Identify Resources**: Catalog all resources that require explicit management
2. **Apply RAII**: Refactor code to use RAII principles where possible
3. **Implement Pooling**: Add resource pooling for frequently used resources
4. **Add Timeouts**: Implement timeouts for all resource acquisition operations
5. **Add Graceful Shutdown**: Implement shutdown procedures for all components
6. **Test Thoroughly**: Verify resource management under normal and error conditions

## Version History

- **1.0.0 (2024-03-21)**: Initial version of the Resource Management Pattern

<version>1.0.0</version> 