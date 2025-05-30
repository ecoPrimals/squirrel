---
version: 1.0.0
last_updated: 2024-04-04
status: active
priority: high
---

# Thread-Safety Pattern Implementation Guide

## Overview

This guide outlines the callback-based thread-safety pattern we've implemented across the Squirrel codebase to address critical thread-safety issues. This pattern replaces direct adapter references with callback functions that enforce proper thread-safety bounds, making concurrency requirements explicit and improving testability.

## The Problem: Thread Safety with Trait Objects

When designing traits in Rust, especially those that will be used as trait objects (e.g., `&dyn MyTrait`), thread safety is a critical concern. Without explicit `Send + Sync` bounds, trait objects might not be thread-safe, leading to subtle concurrency bugs that are difficult to diagnose.

Common thread-safety issues include:

1. **Implicit Thread Safety Assumptions**: Assuming a trait is thread-safe without explicit bounds.
2. **Non-Send/Sync Types in Trait Methods**: Methods that return or work with types that don't implement `Send` or `Sync`.
3. **Shared References Across Threads**: Passing references between threads without proper synchronization.
4. **Interior Mutability Without Thread Safety**: Using `RefCell` or similar types that aren't thread-safe in a concurrent context.

## The Solution: Callback-Based Thread Safety Pattern

Our solution involves:

1. **Explicit Thread Safety Bounds**: New traits explicitly require `Send + Sync`.
2. **Callbacks Instead of Direct References**: Replace direct adapter references with function pointers stored in a callback struct.
3. **Adapter Pattern for Backward Compatibility**: Use wrappers to adapt new trait implementations to old traits.
4. **Default Implementations**: Provide sensible defaults for callback methods where appropriate.

### Key Components of the Pattern

Each implementation of this pattern includes:

1. **Callbacks Struct**: Contains function pointers for operations that would normally require adapter references.
2. **V2 Trait**: Extends `Send + Sync` and defines core functionality.
3. **Wrapper Struct**: Adapts V2 implementations to original traits for backward compatibility.
4. **Helper Functions**: Make it easy to adapt between V1 and V2 implementations.

## Implementation Guide

### Step 1: Define the Callbacks Struct

```rust
pub struct ExampleCallbacks {
    /// Log a message
    pub log: Option<Box<dyn Fn(&str, &str) -> Result<()> + Send + Sync>>,
    
    /// Access a resource
    pub get_resource: Option<Box<dyn Fn(Uuid) -> Result<Arc<dyn Resource>> + Send + Sync>>,
    
    /// Perform an operation
    pub perform_operation: Option<Box<dyn Fn(&str, Value) -> Result<()> + Send + Sync>>,
}

impl Default for ExampleCallbacks {
    fn default() -> Self {
        Self {
            log: None,
            get_resource: None,
            perform_operation: None,
        }
    }
}
```

### Step 2: Define the V2 Trait with Thread Safety Bounds

```rust
#[async_trait]
pub trait ExampleV2: Send + Sync + std::fmt::Debug {
    /// Get metadata about this example
    fn metadata(&self) -> &Metadata;
    
    /// Initialize the example
    async fn initialize(&self) -> Result<()>;
    
    /// Shutdown the example
    async fn shutdown(&self) -> Result<()>;
    
    /// Register callbacks for interaction
    fn register_callbacks(&mut self, callbacks: ExampleCallbacks) {
        // Default empty implementation
        let _ = callbacks; // Suppress unused variable warning
    }
}
```

### Step 3: Create a Wrapper for Backward Compatibility

```rust
#[derive(Debug)]
pub struct ExampleWrapper<T: ExampleV2> {
    inner: T,
}

impl<T: ExampleV2> ExampleWrapper<T> {
    /// Create a new ExampleWrapper with the given ExampleV2 implementation
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<T: ExampleV2 + 'static> Example for ExampleWrapper<T> {
    fn metadata(&self) -> &Metadata {
        self.inner.metadata()
    }
    
    async fn initialize(&self) -> Result<()> {
        self.inner.initialize().await
    }
    
    async fn shutdown(&self) -> Result<()> {
        self.inner.shutdown().await
    }
}
```

### Step 4: Create a Helper Function for Adaptation

```rust
/// Helper function to adapt an ExampleV2 to Example
pub fn adapt_example_v2<T: ExampleV2 + 'static>(example: T) -> Arc<dyn Example> {
    Arc::new(ExampleWrapper::new(example))
}
```

## Example Implementation

Here's a complete example of implementing the V2 trait with thread-safety:

```rust
struct MyExampleV2 {
    metadata: Metadata,
    state: Arc<Mutex<HashMap<String, String>>>,
    callbacks: Option<ExampleCallbacks>,
}

impl std::fmt::Debug for MyExampleV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyExampleV2")
            .field("metadata", &self.metadata)
            .field("state", &self.state)
            .field("callbacks", &"<callbacks>")
            .finish()
    }
}

impl MyExampleV2 {
    fn new(name: &str) -> Self {
        Self {
            metadata: Metadata::new(name, "1.0.0", "Example implementation", "Example Author"),
            state: Arc::new(Mutex::new(HashMap::new())),
            callbacks: None,
        }
    }
    
    fn log(&self, level: &str, message: &str) {
        if let Some(callbacks) = &self.callbacks {
            if let Some(log_fn) = &callbacks.log {
                let _ = log_fn(level, message);
            }
        }
    }
}

#[async_trait]
impl ExampleV2 for MyExampleV2 {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> Result<()> {
        self.log("info", "Initializing example");
        let mut state = self.state.lock().unwrap();
        state.insert("initialized".to_string(), "true".to_string());
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<()> {
        self.log("info", "Shutting down example");
        Ok(())
    }
    
    fn register_callbacks(&mut self, callbacks: ExampleCallbacks) {
        self.callbacks = Some(callbacks);
    }
}
```

## Using V2 Traits with Callbacks

```rust
#[tokio::test]
async fn test_example_v2() {
    // Create a V2 implementation
    let mut example_v2 = MyExampleV2::new("test-example");
    
    // Set up callbacks
    let callbacks = ExampleCallbacks {
        log: Some(Box::new(|level, message| {
            println!("[{}] {}", level, message);
            Ok(())
        })),
        ..Default::default()
    };
    
    // Register callbacks
    example_v2.register_callbacks(callbacks);
    
    // Use the implementation
    example_v2.initialize().await.unwrap();
    
    // Adapt to V1 trait if needed
    let adapted: Arc<dyn Example> = adapt_example_v2(example_v2);
    
    // Use the adapted version
    adapted.shutdown().await.unwrap();
}
```

## Best Practices

1. **Always Include Send + Sync Bounds**: Make thread-safety requirements explicit in trait definitions.
2. **Provide Default Implementations**: Use default implementations for callback registration to simplify implementations.
3. **Use Arc for Shared State**: When state needs to be shared, use `Arc<Mutex<T>>` or `Arc<RwLock<T>>`.
4. **Implement Debug Manually**: Since callbacks can't automatically implement `Debug`, implement it manually for your types.
5. **Test Thread Safety**: Write tests that verify your implementations work correctly in concurrent contexts.
6. **Document Thread Safety**: Clearly document thread-safety guarantees in your API documentation.

## Migrating Existing Code

To migrate existing code to the new pattern:

1. **Create V2 Trait**: Define a new trait with the `Send + Sync` bounds.
2. **Define Callbacks**: Identify operations that need adapter references and convert them to callbacks.
3. **Create Wrapper**: Implement a wrapper to adapt V2 to V1 traits.
4. **Update Implementations**: Create new implementations of the V2 trait.
5. **Adapt When Needed**: Use helper functions to adapt between V1 and V2 traits as needed.

## Common Issues and Solutions

### 1. Debug Implementation for Types with Callbacks

Since function pointers don't implement `Debug`, you'll need to manually implement it:

```rust
impl std::fmt::Debug for MyTypeWithCallbacks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyTypeWithCallbacks")
            .field("data", &self.data)
            .field("callbacks", &"<callbacks>") // Placeholder for callbacks
            .finish()
    }
}
```

### 2. Clone for Types with Callbacks

Function pointers with `Send + Sync` bounds don't automatically implement `Clone`. Either:
- Don't derive `Clone` for callback structs
- Manually implement `Clone` if needed, handling the function pointers appropriately

### 3. Optional Callbacks

Make callbacks optional (`Option<Box<dyn Fn...>>`) to allow for partial implementation and testing.

### 4. Thread-Safety in Tests

When testing, remember that even test code must respect thread-safety bounds:

```rust
// Thread-safe mock for testing
let callbacks = ExampleCallbacks {
    log: Some(Box::new(|level, message| {
        // Thread-safe logging in tests
        Ok(())
    })),
    ..Default::default()
};
```

## Implemented Examples in Codebase

We have successfully implemented this pattern in:

1. **PluginV2**: Plugin system with thread-safe callbacks.
2. **ToolHandlerV2**: Tool handling with thread-safety.
3. **ContextManagerV2**: Context management with proper thread-safety.
4. **AIClientV2**: AI client interfaces with thread-safety.

Refer to these implementations for concrete examples of the pattern in action.

## Conclusion

The callback-based thread-safety pattern provides a robust approach to handling concurrency in trait objects. By making thread-safety requirements explicit and replacing direct adapter references with callback functions, we improve code safety, testability, and maintainability.

This pattern should be applied consistently across the codebase, especially for traits that will be used as trait objects in concurrent contexts.

<version>1.0.0</version> 