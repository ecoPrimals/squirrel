# Plugin Architecture Implementation Patterns

## Overview

This document outlines the key design patterns and implementation approaches used in the Squirrel plugin architecture. Following these patterns ensures consistency across the codebase and makes extending the plugin system easier.

## Core Design Patterns

### 1. Builder Pattern

The plugin system uses the Builder pattern to simplify the creation of plugins:

```rust
let plugin = CommandPluginBuilder::new(metadata)
    .with_command("hello", "Say hello")
    .with_command("echo", "Echo text")
    .build();
```

**Benefits:**
- Simplified plugin creation
- Clear, fluent API
- Type-safe configuration

**Implementation:**
- Builder structs for each plugin type (CommandPluginBuilder, ToolPluginBuilder, etc.)
- Fluent interface with chainable methods
- Final build() method to create the plugin

### 2. Trait-Based Polymorphism

The plugin system uses trait-based polymorphism to define different plugin types:

```rust
trait Plugin: Send + Sync + Any + std::fmt::Debug {
    fn metadata(&self) -> &PluginMetadata;
    fn initialize(&self) -> BoxFuture<'_, Result<()>>;
    // ...
}

trait CommandPlugin: Plugin {
    async fn execute_command(&self, command: &str, args: Value) -> Result<Value>;
    // ...
}
```

**Benefits:**
- Flexibility in plugin implementations
- Dynamic dispatch for plugin operations
- Clear separation of concerns

**Implementation:**
- Base Plugin trait for common functionality
- Specialized traits for specific capabilities
- Dynamic casting via Any trait

### 3. Async/Await for Concurrency

The plugin system uses Rust's async/await pattern for concurrency:

```rust
async fn load_plugin(&self, id: Uuid) -> Result<()> {
    // Async operations...
    self.load_plugin_inner(id).await
}
```

**Benefits:**
- Non-blocking operations
- Efficient resource usage
- Simplified concurrency

**Implementation:**
- Async trait methods using async_trait
- BoxFuture for returning futures
- Tokio runtime for async execution

### 4. Resource Management with Arc and RwLock

The plugin system uses Arc and RwLock for thread-safe resource management:

```rust
pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<Uuid, Box<dyn Plugin>>>>,
    // ...
}
```

**Benefits:**
- Thread-safe access to shared resources
- Fine-grained read/write locking
- Memory safety without garbage collection

**Implementation:**
- Arc for shared ownership
- RwLock for read/write access control
- Careful lock management to prevent deadlocks

### 5. Strategy Pattern for Plugin Discovery

The plugin system uses the Strategy pattern for plugin discovery:

```rust
pub trait PluginDiscovery {
    fn discover(&self) -> BoxFuture<'_, Result<Vec<PluginMetadata>>>;
    // ...
}

pub struct FileSystemDiscovery {
    // ...
}

impl PluginDiscovery for FileSystemDiscovery {
    // ...
}
```

**Benefits:**
- Flexibility in discovery mechanisms
- Testability through abstraction
- Easy to extend with new discovery methods

**Implementation:**
- PluginDiscovery trait for the interface
- Concrete implementations for different discovery strategies
- Factory methods for creating discovery instances

### 6. Dependency Injection

The plugin system uses dependency injection for configuration:

```rust
pub fn with_storage(&mut self, storage: PluginStorageEnum) -> &mut Self {
    // ...
    self
}

pub fn with_security(&mut self) -> &mut Self {
    // ...
    self
}
```

**Benefits:**
- Flexible configuration
- Testability
- Separation of concerns

**Implementation:**
- Method chaining for configuration
- Default implementations for common scenarios
- Factory methods for creating pre-configured instances

### 7. Error Handling with thiserror and anyhow

The plugin system uses thiserror for defining errors and anyhow for error context:

```rust
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(Uuid),
    // ...
}
```

**Benefits:**
- Clear error definitions
- Context-rich error messages
- Simplified error handling

**Implementation:**
- Dedicated error types for each subsystem
- Error context with anyhow
- Use of ? operator for error propagation

## Implementation Patterns by Component

### Plugin Registration and Loading

```
1. Create plugin metadata
2. Create plugin implementation
3. Register plugin with PluginManager
4. Resolve dependencies
5. Load plugins in dependency order
6. Initialize plugins
7. Handle errors with proper recovery
```

### Plugin State Management

```
1. Define plugin state schema
2. Implement get_state/set_state methods
3. Use state in initialize/shutdown hooks
4. Persist state using PluginManager
5. Handle state loading errors
6. Use RwLock for thread-safe state access
```

### Plugin Discovery

```
1. Define discovery strategy
2. Implement scanning mechanism
3. Parse plugin metadata
4. Cache discovery results
5. Handle discovery errors
6. Provide filtering mechanisms
7. Implement periodic rescanning
```

### Security Model

```
1. Define permission levels
2. Implement resource limits
3. Create sandbox environment
4. Validate operations before execution
5. Track resource usage
6. Implement security validator
7. Apply least privilege principle
```

## Anti-Patterns to Avoid

1. **Direct Plugin Casting**: Avoid `unwrap()` when casting plugins to specific types. Use proper error handling.

2. **Synchronous Blocking**: Don't block the async runtime with synchronous operations. Use spawn_blocking for CPU-intensive tasks.

3. **Nested Locks**: Avoid nested locks that can cause deadlocks. Use scoped locks and drop them as soon as possible.

4. **Global State**: Avoid global state and singletons. Use dependency injection instead.

5. **Tight Coupling**: Don't couple plugin implementations to specific systems. Use traits for abstraction.

6. **Inefficient State Management**: Don't save state too frequently. Batch state updates and save at strategic points.

7. **Security Bypasses**: Never bypass security checks. Always use proper validation.

## Example Implementation Walkthrough

### Command Plugin Implementation

```rust
// 1. Define metadata
let metadata = PluginMetadata {
    id: Uuid::new_v4(),
    name: "example-commands".to_string(),
    version: "0.1.0".to_string(),
    description: "Example command plugin".to_string(),
    author: "Example Author".to_string(),
    dependencies: vec![],
    capabilities: vec!["command".to_string()],
};

// 2. Create plugin using builder pattern
let plugin = CommandPluginBuilder::new(metadata)
    .with_command("hello", "Say hello to someone")
    .with_command("echo", "Echo back the input")
    .build();

// 3. Register with manager
manager.register_plugin(plugin).await?;

// 4. Load plugin and dependencies
manager.load_plugin_with_recovery(metadata.id).await?;

// 5. Use the plugin
let command_plugin = manager.get_command_plugin(metadata.id).await?;
let result = command_plugin.execute_command("hello", json!({"name": "User"})).await?;
```

### Implementing a Custom Plugin

```rust
// 1. Define struct
struct MyPlugin {
    metadata: PluginMetadata,
    state: RwLock<Option<PluginState>>,
    // Custom fields...
}

// 2. Implement base Plugin trait
impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    // Initialize, shutdown, state management...
}

// 3. Implement specialized trait
#[async_trait]
impl CommandPlugin for MyPlugin {
    async fn execute_command(&self, command: &str, args: Value) -> Result<Value> {
        // Command implementation...
    }
    
    // Other methods...
}

// 4. Create and register
let plugin = MyPlugin::new();
manager.register_plugin(Box::new(plugin)).await?;
```

## Best Practices

1. **Test Plugins Thoroughly**: Create unit and integration tests for plugins.
2. **Document Plugin Capabilities**: Clearly document what commands and features your plugin provides.
3. **Handle Errors Gracefully**: Use proper error handling and recovery mechanisms.
4. **Be Resource Conscious**: Monitor and limit resource usage in plugins.
5. **Follow Async Best Practices**: Avoid blocking the async runtime.
6. **Use Builder Patterns**: Simplify plugin creation with builders when possible.
7. **Implement Clean Shutdown**: Always clean up resources in the shutdown method.
8. **Respect Security Boundaries**: Don't try to bypass security restrictions.
9. **Optimize State Management**: Don't save state too frequently.
10. **Follow Naming Conventions**: Use consistent naming for commands and features. 