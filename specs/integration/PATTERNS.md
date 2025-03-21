---
version: 1.0.0
last_updated: 2024-03-23
status: draft
priority: high
---

# Integration Patterns

## Overview
This document defines standard integration patterns for component interactions within the Squirrel platform. These patterns provide consistent approaches for implementing communication, state management, and error handling across different parts of the system.

## Pattern Categories

### 1. Component Communication Patterns

#### A. Service Interface Pattern
Components expose functionality through well-defined trait interfaces:

```rust
#[async_trait]
pub trait MyService: Send + Sync {
    async fn operation_one(&self, param: ParamType) -> Result<ResponseType>;
    async fn operation_two(&self, param: OtherParamType) -> Result<OtherResponseType>;
}

// Implementation
pub struct MyServiceImpl {
    // Dependencies
    dependency: Arc<dyn DependencyService>,
    // State
    state: Arc<RwLock<ServiceState>>,
}

#[async_trait]
impl MyService for MyServiceImpl {
    async fn operation_one(&self, param: ParamType) -> Result<ResponseType> {
        // Implementation
    }
    
    async fn operation_two(&self, param: OtherParamType) -> Result<OtherResponseType> {
        // Implementation
    }
}
```

**When to Use:**
- For core service capabilities that may have multiple implementations
- When the component's functionality needs to be accessed by multiple consumers
- For services that may be mocked in tests

**Benefits:**
- Clear API contract
- Implementation flexibility
- Testability through mocking
- Better separation of concerns

#### B. Event-Based Communication Pattern
Components communicate through an event bus without direct coupling:

```rust
// Event definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    StateChanged(StateChangeEvent),
    ResourceUpdated(ResourceUpdateEvent),
    UserAction(UserActionEvent),
    Error(ErrorEvent),
}

// Event bus interface
#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, event: SystemEvent) -> Result<()>;
    async fn subscribe(&self, topic: EventTopic) -> Result<mpsc::Receiver<SystemEvent>>;
}

// Component using event bus
pub struct ComponentWithEvents {
    event_bus: Arc<dyn EventBus>,
    // Other fields
}

impl ComponentWithEvents {
    pub async fn handle_event(&self, event: SystemEvent) -> Result<()> {
        match event {
            SystemEvent::StateChanged(e) => self.on_state_changed(e).await,
            SystemEvent::ResourceUpdated(e) => self.on_resource_updated(e).await,
            _ => Ok(()), // Ignore other events
        }
    }
    
    pub async fn trigger_action(&self) -> Result<()> {
        // Perform action
        let result = self.perform_action().await?;
        
        // Publish event
        self.event_bus.publish(SystemEvent::ResourceUpdated(
            ResourceUpdateEvent::new(result)
        )).await?;
        
        Ok(())
    }
}
```

**When to Use:**
- For loosely coupled components
- When multiple components need to react to the same events
- For asynchronous workflows where components operate independently

**Benefits:**
- Reduced coupling between components
- Easier to add new components without changing existing ones
- Simplified scaling and distribution
- Better handling of asynchronous operations

### 2. State Management Patterns

#### A. Shared State Pattern
Multiple components access shared state with proper synchronization:

```rust
// State definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedState {
    pub data: HashMap<String, Value>,
    pub version: u64,
    pub last_updated: DateTime<Utc>,
}

// State manager
pub struct StateManager {
    state: Arc<RwLock<SharedState>>,
    listeners: Arc<RwLock<Vec<StateChangeListener>>>,
}

impl StateManager {
    pub async fn update<F, T>(&self, updater: F) -> Result<T>
    where
        F: FnOnce(&mut SharedState) -> Result<T>,
    {
        let mut state = self.state.write().await;
        let result = updater(&mut state)?;
        state.version += 1;
        state.last_updated = Utc::now();
        
        // Notify listeners
        self.notify_state_change(&state).await?;
        
        Ok(result)
    }
    
    pub async fn get(&self) -> Result<SharedState> {
        Ok(self.state.read().await.clone())
    }
}

// Component using shared state
pub struct ComponentWithSharedState {
    state_manager: Arc<StateManager>,
}

impl ComponentWithSharedState {
    pub async fn perform_operation(&self) -> Result<()> {
        self.state_manager.update(|state| {
            state.data.insert("key".to_string(), Value::String("value".to_string()));
            Ok(())
        }).await?;
        
        Ok(())
    }
}
```

**When to Use:**
- When multiple components need access to the same state
- For centralized state management
- When state changes need to trigger reactions in multiple components

**Benefits:**
- Single source of truth
- Consistent state access
- Proper synchronization
- Change notification

#### B. State Synchronization Pattern
Components maintain local state that synchronizes with a central source:

```rust
// State synchronization interface
#[async_trait]
pub trait StateSynchronizer {
    async fn push_state(&self, state: LocalState) -> Result<()>;
    async fn pull_state(&self) -> Result<RemoteState>;
    async fn sync(&self) -> Result<SyncResult>;
}

// Component with local state
pub struct ComponentWithLocalState {
    local_state: RwLock<LocalState>,
    synchronizer: Arc<dyn StateSynchronizer>,
    last_sync: AtomicU64,
}

impl ComponentWithLocalState {
    pub async fn sync_state(&self) -> Result<()> {
        // Pull remote state
        let remote_state = self.synchronizer.pull_state().await?;
        
        // Merge with local state
        let mut local = self.local_state.write().await;
        *local = self.merge_states(&local, &remote_state)?;
        
        // Update last sync time
        self.last_sync.store(Utc::now().timestamp() as u64, Ordering::SeqCst);
        
        Ok(())
    }
    
    pub async fn update_local_state(&self) -> Result<()> {
        // Update local state
        let mut local = self.local_state.write().await;
        // Modify local state...
        
        // Push to remote if enough time has passed since last sync
        let now = Utc::now().timestamp() as u64;
        let last = self.last_sync.load(Ordering::SeqCst);
        
        if now - last > self.sync_interval {
            self.synchronizer.push_state(local.clone()).await?;
            self.last_sync.store(now, Ordering::SeqCst);
        }
        
        Ok(())
    }
}
```

**When to Use:**
- For distributed components that need their own state copy
- When network connectivity might be unreliable
- For performance-critical components that need local state access

**Benefits:**
- Better performance for local operations
- Resilience to network issues
- Reduced contention on central state
- Control over synchronization frequency

### 3. Error Handling Patterns

#### A. Error Propagation Pattern
Components propagate errors with appropriate context:

```rust
// Error types
#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Dependency error: {0}")]
    DependencyError(#[from] DependencyError),
    
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

// Component with proper error handling
impl MyComponent {
    pub async fn operation(&self) -> Result<(), ComponentError> {
        // Try dependency operation
        let result = self.dependency
            .perform_operation()
            .await
            .map_err(|e| {
                // Add context to error
                ComponentError::DependencyError(e)
            })?;
        
        // Check result
        if !self.validate_result(&result) {
            return Err(ComponentError::OperationFailed(
                format!("Validation failed for result: {:?}", result)
            ));
        }
        
        Ok(())
    }
}
```

**When to Use:**
- For all component boundaries
- When detailed error information is needed for debugging
- When errors need to be handled at different levels of the system

**Benefits:**
- Clear error context
- Proper error categorization
- Easy to track error source
- Better debugging experience

#### B. Circuit Breaker Pattern
Prevents cascading failures when a dependency fails repeatedly:

```rust
pub struct CircuitBreaker {
    state: AtomicU8,
    failure_count: AtomicUsize,
    last_failure: AtomicU64,
    settings: CircuitBreakerSettings,
}

impl CircuitBreaker {
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        match self.get_state() {
            CircuitState::Closed => {
                // Normal operation
                match operation.await {
                    Ok(result) => {
                        // Reset failure count on success
                        self.failure_count.store(0, Ordering::SeqCst);
                        Ok(result)
                    }
                    Err(e) => {
                        // Increment failure count
                        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                        self.last_failure.store(Utc::now().timestamp() as u64, Ordering::SeqCst);
                        
                        // Trip circuit if threshold reached
                        if failures >= self.settings.failure_threshold {
                            self.trip_circuit();
                        }
                        
                        Err(e)
                    }
                }
            }
            CircuitState::Open => {
                // Circuit is open, fast fail
                Err(E::from_error(CircuitBreakerError::CircuitOpen))
            }
            CircuitState::HalfOpen => {
                // Test if the circuit can be closed again
                match operation.await {
                    Ok(result) => {
                        // Success, close the circuit
                        self.close_circuit();
                        Ok(result)
                    }
                    Err(e) => {
                        // Failed again, keep circuit open
                        self.trip_circuit();
                        Err(e)
                    }
                }
            }
        }
    }
}
```

**When to Use:**
- For operations that depend on external systems
- When failures in one component should not cascade to others
- For resilient system design

**Benefits:**
- Prevents cascading failures
- Allows systems to recover
- Reduces load on failing systems
- Improves overall system stability

### 4. Resource Management Patterns

#### A. Connection Pool Pattern
Manages shared connections to external resources:

```rust
pub struct ConnectionPool<T> {
    available: Mutex<Vec<T>>,
    max_size: usize,
    factory: Box<dyn Fn() -> Result<T> + Send + Sync>,
}

impl<T: Send + Sync + 'static> ConnectionPool<T> {
    pub async fn get(&self) -> Result<PooledConnection<T>> {
        // Try to get an available connection
        let mut available = self.available.lock().await;
        
        let conn = if let Some(conn) = available.pop() {
            conn
        } else if available.len() < self.max_size {
            // Create new connection if under max size
            (self.factory)()?
        } else {
            // Wait for a connection to become available
            return Err(PoolError::NoAvailableConnections);
        };
        
        // Return pooled connection
        Ok(PooledConnection {
            conn: Some(conn),
            pool: self,
        })
    }
    
    fn return_connection(&self, conn: T) {
        let mut available = self.available.lock().await;
        available.push(conn);
    }
}

// Pooled connection that returns to pool on drop
pub struct PooledConnection<'a, T> {
    conn: Option<T>,
    pool: &'a ConnectionPool<T>,
}

impl<'a, T> Drop for PooledConnection<'a, T> {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            self.pool.return_connection(conn);
        }
    }
}

// Usage
let pool = ConnectionPool::new(
    10, // max_size
    Box::new(|| {
        // Connection factory
        DatabaseConnection::connect(&config)
    })
);

let conn = pool.get().await?;
// Use connection
conn.execute_query("SELECT * FROM table").await?;
// Connection automatically returns to pool when dropped
```

**When to Use:**
- For managing expensive resources like database connections
- When resources are limited and need to be shared
- For improving performance by reusing resources

**Benefits:**
- Efficient resource utilization
- Controlled access to limited resources
- Automatic resource cleanup
- Improved performance

#### B. Resource Lifecycle Pattern
Manages resource creation, initialization, and cleanup:

```rust
#[async_trait]
pub trait ManagedResource: Send + Sync {
    async fn initialize(&self) -> Result<()>;
    async fn is_healthy(&self) -> bool;
    async fn cleanup(&self) -> Result<()>;
}

pub struct ResourceManager {
    resources: RwLock<HashMap<ResourceId, Arc<dyn ManagedResource>>>,
}

impl ResourceManager {
    pub async fn register<R>(&self, id: ResourceId, resource: R) -> Result<()>
    where
        R: ManagedResource + 'static,
    {
        let resource = Arc::new(resource);
        
        // Initialize the resource
        resource.initialize().await?;
        
        // Store in resources map
        let mut resources = self.resources.write().await;
        resources.insert(id, resource);
        
        Ok(())
    }
    
    pub async fn get(&self, id: &ResourceId) -> Result<Arc<dyn ManagedResource>> {
        let resources = self.resources.read().await;
        
        resources.get(id)
            .cloned()
            .ok_or_else(|| ResourceError::NotFound(id.clone()))
    }
    
    pub async fn cleanup_all(&self) -> Result<()> {
        let resources = self.resources.read().await;
        
        for (id, resource) in resources.iter() {
            if let Err(e) = resource.cleanup().await {
                log::warn!("Failed to clean up resource {}: {}", id, e);
            }
        }
        
        Ok(())
    }
}
```

**When to Use:**
- For managing resources with complex lifecycles
- When resources need proper initialization and cleanup
- For centralized resource management

**Benefits:**
- Proper resource initialization
- Controlled resource lifecycle
- Centralized management
- Guaranteed cleanup

## Implementation Examples

### 1. Service Interface Example

```rust
// MCP protocol service interface
#[async_trait]
pub trait McpService: Send + Sync {
    async fn send_message(&self, message: McpMessage) -> Result<McpResponse>;
    async fn receive_messages(&self) -> Result<mpsc::Receiver<McpMessage>>;
    async fn register_handler(&self, handler: Box<dyn McpMessageHandler>) -> Result<HandlerId>;
}

// Core component using MCP service
pub struct CoreComponent {
    mcp_service: Arc<dyn McpService>,
    state: Arc<RwLock<CoreState>>,
}

impl CoreComponent {
    pub fn new(mcp_service: Arc<dyn McpService>) -> Self {
        let component = Self {
            mcp_service,
            state: Arc::new(RwLock::new(CoreState::default())),
        };
        
        // Register message handler
        let handler = Box::new(CoreMessageHandler::new(component.state.clone()));
        let _ = component.mcp_service.register_handler(handler);
        
        component
    }
    
    pub async fn send_command(&self, command: Command) -> Result<CommandResponse> {
        // Convert to MCP message
        let message = McpMessage::new_command(command);
        
        // Send via MCP service
        let response = self.mcp_service.send_message(message).await?;
        
        // Convert response
        Ok(CommandResponse::from_mcp(response))
    }
}
```

### 2. Event-Based Communication Example

```rust
// Event definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    ContextChanged(ContextChangedEvent),
    ToolExecuted(ToolExecutedEvent),
    UserAction(UserActionEvent),
}

// UI component using events
pub struct UiComponent {
    event_bus: Arc<dyn EventBus>,
    state: Arc<RwLock<UiState>>,
}

impl UiComponent {
    pub fn new(event_bus: Arc<dyn EventBus>) -> Self {
        let component = Self {
            event_bus: event_bus.clone(),
            state: Arc::new(RwLock::new(UiState::default())),
        };
        
        // Subscribe to relevant events
        let event_bus_clone = event_bus.clone();
        let state_clone = component.state.clone();
        
        tokio::spawn(async move {
            let mut receiver = event_bus_clone.subscribe(
                EventTopic::Context | EventTopic::Tool
            ).await.unwrap();
            
            while let Some(event) = receiver.recv().await {
                match event {
                    SystemEvent::ContextChanged(e) => {
                        let mut state = state_clone.write().await;
                        state.update_from_context(e.context);
                    },
                    SystemEvent::ToolExecuted(e) => {
                        let mut state = state_clone.write().await;
                        state.update_tool_status(e.tool_id, e.status);
                    },
                    _ => {}
                }
            }
        });
        
        component
    }
    
    pub async fn user_action(&self, action: UserAction) -> Result<()> {
        // Perform local updates
        {
            let mut state = self.state.write().await;
            state.record_user_action(&action);
        }
        
        // Publish event
        self.event_bus.publish(SystemEvent::UserAction(
            UserActionEvent::new(action)
        )).await?;
        
        Ok(())
    }
}
```

## Best Practices

### 1. Interface Design
- Define clear trait interfaces for component boundaries
- Use async functions for potentially blocking operations
- Return Result types for operations that can fail
- Keep interfaces focused on a single responsibility

### 2. State Management
- Use appropriate synchronization primitives (RwLock, Mutex)
- Consider state ownership and access patterns
- Document state invariants and access requirements
- Implement proper change notification mechanisms

### 3. Error Handling
- Define component-specific error types
- Include context in error messages
- Implement proper error conversion between components
- Use circuit breakers for external dependencies

### 4. Resource Management
- Implement proper lifecycle management for resources
- Use connection pooling for expensive resources
- Ensure cleanup happens even in error cases
- Monitor resource usage and health

### 5. Testing
- Mock dependencies for unit testing
- Test integration points explicitly
- Simulate error conditions
- Verify state consistency across components

## Migration Strategies

When migrating components to use these patterns:

1. **Interface Abstraction**:
   - Extract interfaces from concrete implementations
   - Update consumers to use interface types
   - Implement adapters for legacy components

2. **Event Conversion**:
   - Introduce event types alongside direct calls
   - Gradually migrate to event-based communication
   - Use adapters to bridge direct and event-based approaches

3. **State Management**:
   - Encapsulate state behind proper abstractions
   - Introduce synchronization gradually
   - Use feature flags to toggle between approaches

4. **Error Handling**:
   - Define new error types
   - Add context to existing errors
   - Implement conversion between error types

## Conclusion

These integration patterns provide a foundation for building robust, maintainable component interactions within the Squirrel platform. By consistently applying these patterns, teams can ensure that components interact properly, handle errors gracefully, and manage state effectively.

<version>1.0.0</version> 